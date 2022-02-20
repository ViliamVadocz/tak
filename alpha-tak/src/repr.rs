use tak::{
    board::Board,
    colour::Colour,
    game::{default_starting_stones, Game},
    pos::Pos,
    tile::Shape,
};
use tch::{kind::FLOAT_CPU, Tensor};

const STACK_DEPTH_BEYOND_CARRY: usize = 6;
const COLOUR_CHANNEL: usize = 1;
const FCD_CHANNEL: usize = 1;

pub const fn board_channels(n: usize) -> usize {
    (n + 2 + STACK_DEPTH_BEYOND_CARRY) * 2
}

pub const fn input_channels(n: usize) -> usize {
    let (stones, capstones) = default_starting_stones(n);
    board_channels(n) + COLOUR_CHANNEL + FCD_CHANNEL + 2 * stones as usize + 2 * capstones as usize
}

pub const fn moves_dims(n: usize) -> usize {
    match n {
        3 => 2 * 3 * 3 + 108,   // 126
        4 => 2 * 4 * 4 + 448,   // 480
        5 => 3 * 5 * 5 + 1500,  // 1575
        6 => 3 * 6 * 6 + 4464,  // 4572
        7 => 3 * 7 * 7 + 12348, // 12495
        8 => 3 * 8 * 8 + 32512, // 32704
        _ => unimplemented!(),
    }
}

/// Creates a tensor which represents the board
/// from the perspective of the current player.
fn board_repr<const N: usize>(board: &Board<N>, to_move: Colour) -> Tensor {
    // track the positions of all ones in the input tensor
    let mut positions = Vec::new();

    // top layer of stack has 6 channels in total
    // 2 for flats (1 per player)
    // 2 for walls (1 per player)
    // 2 for caps (1 per player)
    for y in 0..N {
        for x in 0..N {
            let pos = Pos { x, y };
            if let Some(tile) = &board[pos] {
                let board_offset = N * y + x;
                let channel = match tile.top.shape {
                    Shape::Flat => 0,
                    Shape::Wall => 2,
                    Shape::Capstone => 4,
                } + if tile.top.colour == to_move { 0 } else { 1 };
                positions.push((board_offset + N * N * channel) as i64);
            }
        }
    }

    // other layers of stacks
    // alternating mine and opponent's
    for n in 0..(N + STACK_DEPTH_BEYOND_CARRY - 1) {
        for y in 0..N {
            for x in 0..N {
                let pos = Pos { x, y };
                if let Some(tile) = &board[pos] {
                    let board_offset = N * y + x;
                    if let Some(&colour) = tile.stack.iter().rev().nth(n) {
                        positions.push(
                            (board_offset + N * N * (6 + 2 * n + if to_move == colour { 0 } else { 1 }))
                                as i64,
                        );
                    }
                }
            }
        }
    }

    let index = Tensor::of_slice(&positions);
    let ones = Tensor::ones(&[positions.len() as i64], FLOAT_CPU);
    let mut zeros = Tensor::zeros(&[board_channels(N) as i64, N as i64, N as i64], FLOAT_CPU);
    zeros.put_(&index, &ones, false)
}

fn create_reserves_tensor<const N: usize>(stones: u8, max: u8) -> Tensor {
    let mut reserves = Tensor::zeros(&[max as i64, N as i64, N as i64], FLOAT_CPU);
    if stones > 0 {
        reserves = reserves.index_put_(
            &[Some(Tensor::of_slice(&[(stones - 1) as i64])), None, None],
            &Tensor::ones(&[N as i64, N as i64], FLOAT_CPU),
            false,
        );
    }
    reserves
}

fn reserves_repr<const N: usize>(game: &Game<N>) -> (Tensor, Tensor, Tensor, Tensor) {
    let (stones, capstones) = default_starting_stones(N);
    let white_stones = create_reserves_tensor::<N>(game.white_stones, stones);
    let black_stones = create_reserves_tensor::<N>(game.black_stones, stones);
    let white_caps = create_reserves_tensor::<N>(game.white_caps, capstones);
    let black_caps = create_reserves_tensor::<N>(game.black_caps, capstones);

    if game.to_move == Colour::White {
        (white_stones, black_stones, white_caps, black_caps)
    } else {
        (black_stones, white_stones, black_caps, white_caps)
    }
}

pub fn game_repr<const N: usize>(game: &Game<N>) -> Tensor {
    let board = board_repr(&game.board, game.to_move);

    let layer_shape = [1, N as i64, N as i64];

    // one-hot encoded reserves
    let (my_stones, en_stones, my_caps, en_caps) = reserves_repr(game);

    // layer for whose turn it is
    let colour_layer = Tensor::full(
        &layer_shape,
        if game.to_move == Colour::White { 1. } else { 0. },
        FLOAT_CPU,
    );

    // layer for fcd (+ komi)
    let fcd = game.board.flat_diff() - game.komi;
    let relative_fcd = fcd as f64 / (N * N) as f64;
    let fcd_layer = Tensor::full(&layer_shape, relative_fcd, FLOAT_CPU);

    Tensor::cat(
        &[
            board,
            my_stones,
            en_stones,
            my_caps,
            en_caps,
            colour_layer,
            fcd_layer,
        ],
        0,
    )
}

#[cfg(test)]
mod test {
    use tak::{board::Board, colour::Colour, game::Game, ptn::FromPTN};
    use tch::{kind::FLOAT_CPU, Tensor};
    use test::Bencher;

    use super::{board_repr, game_repr};
    use crate::repr::board_channels;

    #[test]
    fn empty_board() {
        let board = Board::<5>::default();
        assert_eq!(
            board_repr(&board, Colour::White),
            Tensor::zeros(&[board_channels(5) as i64, 5, 5], FLOAT_CPU)
        );
    }

    #[test]
    fn complicated_board() {
        let board = Board::<5>::from_ptn("x2,1221,x,1S/2,2C,2,1,x/x,212,21C,2S,2/2211S,2,21,1,1/x2,221S,2,x")
            .unwrap();
        let (x, o) = (true, false);
        #[rustfmt::skip]
        let handmade = Tensor::cat(&[
            Tensor::of_slice(&[
                // my flats
                o, o, o, o, o,
                o, o, x, x, x,
                o, o, o, o, o,
                o, o, o, x, o,
                o, o, x, o, o,
                // en flats
                o, o, o, x, o,
                o, x, o, o, o,
                o, x, o, o, x,
                x, o, x, o, o,
                o, o, o, o, o,
                // my walls
                o, o, x, o, o,
                x, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, x,
                // en walls
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, x, o,
                o, o, o, o, o,
                o, o, o, o, o,
                // my caps
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, x, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                // en caps
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                o, x, o, o, o,
                o, o, o, o, o,
                // my second layer
                o, o, o, o, o,
                x, o, o, o, o,
                o, x, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                // en second layer
                o, o, x, o, o,
                o, o, x, o, o,
                o, o, x, o, o,
                o, o, o, o, o,
                o, o, x, o, o,
                // my third layer
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                // en third layer
                o, o, x, o, o,
                x, o, o, o, o,
                o, x, o, o, o,
                o, o, o, o, o,
                o, o, x, o, o,
                // my fourth layer
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, x, o, o,
                // en fourth layer
                o, o, o, o, o,
                x, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
            ]).view([12, 5, 5]),
            Tensor::zeros(&[board_channels(5) as  i64 - 12, 5, 5], FLOAT_CPU)
        ], 0);

        let a: Vec<f32> = handmade.into();
        let b: Vec<f32> = board_repr(&board, Colour::White).into();
        assert_eq!(a, b);
    }

    #[bench]
    fn game_repr_bench(b: &mut Bencher) {
        let game = Game::<5>::from_ptn(
            "1. b3 c3
            2. Cc4 Sd3
            3. c3< c3
            4. c4- Cb4
            5. b2 c2
            6. d2 a2
            7. Sb1 a1
            8. b1+ a1+
            9. 2b2< b2
            10. a5 b5
            11. c5 d5
            12. Se5 a4
            13. a5> d5<
            14. 2b5> c4
            15. c1 d1
            16. c1+ c1
            17. d4 b1
            18. Sa1 a3
            19. a1> a3>
            20. 2b1> e3
            21. e2",
        )
        .unwrap();

        b.iter(|| game_repr(&game));
    }
}
