use tak::{
    board::Board,
    colour::Colour,
    game::{default_starting_stones, Game},
    pos::Pos,
    tile::Shape,
};
use tch::{Kind, Tensor};

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
    let board_shape = [N as i64, N as i64];

    // handle top layer
    // each shape type has its own channel, different colours also go on seperate
    // channels
    let mut my_flats = vec![false; N * N];
    let mut en_flats = vec![false; N * N];
    let mut my_walls = vec![false; N * N];
    let mut en_walls = vec![false; N * N];
    let mut my_caps = vec![false; N * N];
    let mut en_caps = vec![false; N * N];
    for y in 0..N {
        for x in 0..N {
            let pos = Pos { x, y };
            if let Some(tile) = &board[pos] {
                let i = N * y + x;
                if tile.top.colour == to_move {
                    match tile.top.shape {
                        Shape::Flat => my_flats[i] = true,
                        Shape::Wall => my_walls[i] = true,
                        Shape::Capstone => my_caps[i] = true,
                    }
                } else {
                    match tile.top.shape {
                        Shape::Flat => en_flats[i] = true,
                        Shape::Wall => en_walls[i] = true,
                        Shape::Capstone => en_caps[i] = true,
                    }
                }
            }
        }
    }

    let mut layers = vec![
        Tensor::of_slice(&my_flats).view(board_shape),
        Tensor::of_slice(&en_flats).view(board_shape),
        Tensor::of_slice(&my_walls).view(board_shape),
        Tensor::of_slice(&en_walls).view(board_shape),
        Tensor::of_slice(&my_caps).view(board_shape),
        Tensor::of_slice(&en_caps).view(board_shape),
    ];

    // other layers
    for n in 0..(N + STACK_DEPTH_BEYOND_CARRY - 1) {
        let mut my_layer = vec![false; N * N];
        let mut en_layer = vec![false; N * N];
        for y in 0..N {
            for x in 0..N {
                let pos = Pos { x, y };
                if let Some(tile) = &board[pos] {
                    let i = N * y + x;
                    if let Some(&colour) = tile.stack.iter().rev().nth(n) {
                        if to_move == colour {
                            my_layer[i] = true;
                        } else {
                            en_layer[i] = true;
                        }
                    }
                }
            }
        }
        layers.push(Tensor::of_slice(&my_layer).view(board_shape));
        layers.push(Tensor::of_slice(&en_layer).view(board_shape));
    }

    Tensor::stack(&layers, 0)
}

fn create_reserves_tensor<const N: usize>(stones: u8, max: u8) -> Tensor {
    let mut reserves = vec![vec![false; N * N]; max as usize];
    if stones > 0 {
        reserves[(stones - 1) as usize] = vec![true; N * N];
    }
    Tensor::stack(
        &reserves
            .into_iter()
            .map(|v| Tensor::of_slice(&v).view([N as i64, N as i64]))
            .collect::<Vec<_>>(),
        0,
    )
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
    let colour_layer = Tensor::of_slice(&vec![game.to_move == Colour::White; N * N]).view(layer_shape);

    // layer for fcd (+ komi)
    let fcd = game.board.flat_diff() - game.komi;
    let fcd_layer = Tensor::of_slice(&vec![fcd as f32 / (N * N) as f32; N * N]).view(layer_shape);

    Tensor::cat(
        &[
            board.to_kind(Kind::Float),
            my_stones.to_kind(Kind::Float),
            en_stones.to_kind(Kind::Float),
            my_caps.to_kind(Kind::Float),
            en_caps.to_kind(Kind::Float),
            colour_layer.to_kind(Kind::Float),
            fcd_layer,
        ],
        0,
    )
}

#[cfg(test)]
mod test {
    use tak::{board::Board, colour::Colour, ptn::FromPTN};
    use tch::{Device, Kind, Tensor};

    use super::board_repr;
    use crate::repr::board_channels;
    // fn eq(a: &Tensor, b: &Tensor) -> bool {
    //     let diff: f32 = (a - b).square().sum(Kind::Float).into();
    //     diff < 1e-9
    // }

    #[test]
    fn empty_board() {
        let board = Board::<5>::default();
        assert_eq!(
            board_repr(&board, Colour::White),
            Tensor::zeros(&[board_channels(5) as i64, 5, 5], (Kind::Bool, Device::Cpu))
        );
    }

    #[test]
    fn complicated_board() {
        let board =
            Board::<5>::from_ptn("x2,1221,x,1S/2,2C,2,1,x/x,212,21C,2S,2/2211S,2,21,1,1/x2,221S,2,x 2 21")
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
            Tensor::zeros(&[board_channels(5) as  i64 - 12, 5, 5], (Kind::Bool, Device::Cpu))
        ], 0);

        let a: Vec<bool> = board_repr(&board, Colour::White).into();
        let b: Vec<bool> = handmade.into();
        assert_eq!(a, b);
        // assert_eq!(board_repr(&board, Colour::Black), handmade) // idk
        // doesn't work
    }
}
