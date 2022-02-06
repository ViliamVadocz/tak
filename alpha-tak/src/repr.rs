use tak::{board::Board, colour::Colour, game::Game, pos::Pos, tile::Shape};
use tch::Tensor;

const STACK_DEPTH_BEYOND_CARRY: usize = 1;
const COLOUR_CHANNEL: usize = 1;
const FCD_CHANNEL: usize = 1;

pub const fn input_dims(n: usize) -> [usize; 3] {
    // channels first
    [
        n + 2 + STACK_DEPTH_BEYOND_CARRY + COLOUR_CHANNEL + FCD_CHANNEL,
        n,
        n,
    ]
}

pub const fn board_dims(n: usize) -> [usize; 3] {
    let [d1, d2, d3] = input_dims(n);
    [(d1 - FCD_CHANNEL), d2, d3]
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

fn colour_repr(colour: &Colour, to_move: &Colour) -> f32 {
    if colour == to_move {
        1.
    } else {
        -1.
    }
}

/// Creates a tensor which represents the board
/// from the perspective of the current player.
fn board_repr<const N: usize>(board: &Board<N>, to_move: Colour) -> Tensor {
    let [d1, d2, d3] = board_dims(N);
    let board_shape = [d2 as i64, d3 as i64];

    let mut flats = [[0.; N]; N];
    let mut walls = [[0.; N]; N];
    let mut caps = [[0.; N]; N];
    // top layer
    for y in 0..N {
        for x in 0..N {
            let pos = Pos { x, y };
            if let Some(tile) = &board[pos] {
                let colour = colour_repr(&tile.top.colour, &to_move);
                match tile.top.shape {
                    Shape::Flat => flats[y][x] = colour,
                    Shape::Wall => walls[y][x] = colour,
                    Shape::Capstone => caps[y][x] = colour,
                }
            }
        }
    }
    let mut layers = vec![
        Tensor::of_slice(flats.into_iter().flatten().collect::<Vec<_>>().as_slice()).view(board_shape),
        Tensor::of_slice(walls.into_iter().flatten().collect::<Vec<_>>().as_slice()).view(board_shape),
        Tensor::of_slice(caps.into_iter().flatten().collect::<Vec<_>>().as_slice()).view(board_shape),
    ];

    // other layers
    for n in 0..(d1 - 2 - COLOUR_CHANNEL - 1) {
        let mut layer = [[0.; N]; N];
        #[allow(clippy::needless_range_loop)]
        for y in 0..N {
            for x in 0..N {
                let pos = Pos { x, y };
                if let Some(tile) = &board[pos] {
                    if let Some(colour) = tile.stack.iter().rev().nth(n) {
                        layer[y][x] = colour_repr(colour, &to_move);
                    }
                }
            }
        }
        layers.push(
            Tensor::of_slice(layer.into_iter().flatten().collect::<Vec<_>>().as_slice()).view(board_shape),
        );
    }
    // last layer for whose turn it is
    let colour_layer: Vec<f32> = vec![if to_move == Colour::White { 1. } else { -1. }; N * N];
    layers.push(Tensor::of_slice(&colour_layer).view(board_shape));

    Tensor::stack(&layers, 0)
}

// TODO add other info such as reserves
pub fn game_repr<const N: usize>(game: &Game<N>) -> Tensor {
    let board = board_repr(&game.board, game.to_move);

    let fcd = game.board.flat_diff() - game.komi;
    let relative_fcd = fcd as f32 / (N * N) as f32;
    Tensor::cat(
        &[
            board,
            Tensor::of_slice(&vec![relative_fcd; N * N]).view([FCD_CHANNEL as i64, N as i64, N as i64]),
        ],
        0,
    )
}

#[cfg(test)]
mod test {
    use tak::{board::Board, colour::Colour, game::Game, ptn::FromPTN};
    use tch::{Kind, Tensor};

    use super::{board_dims, board_repr};

    fn dims(n: usize) -> [i64; 3] {
        board_dims(n).map(|x| x as i64)
    }

    fn eq(a: &Tensor, b: &Tensor) -> bool {
        let diff: f32 = (a - b).square().sum(Kind::Float).into();
        diff < 1e-9
    }

    #[test]
    fn empty_board() {
        let repr = board_repr(&Board::<3>::default(), Colour::White);
        #[rustfmt::skip]
        let tensor = Tensor::of_slice(&[
            // flats
            0., 0., 0.,
            0., 0., 0.,
            0., 0., 0.,
            // walls
            0., 0., 0.,
            0., 0., 0.,
            0., 0., 0.,
            // caps
            0., 0., 0.,
            0., 0., 0.,
            0., 0., 0.,
            // layer 2
            0., 0., 0.,
            0., 0., 0.,
            0., 0., 0.,
            // layer 3
            0., 0., 0.,
            0., 0., 0.,
            0., 0., 0.,
            // layer 4 (since STACK_DEPTH_BEYOND_CARRY = 1)
            0., 0., 0.,
            0., 0., 0.,
            0., 0., 0.,
            // white to move
            1., 1., 1.,
            1., 1., 1.,
            1., 1., 1.,
            ]
        ).reshape(&dims(3));
        assert!(eq(&repr, &tensor));
    }

    #[test]
    fn no_stacks() {
        let game = Game::<3>::from_ptn(
            "
            1. a1 b3
            2. c3 Sa2
            3. b2 c2",
        )
        .unwrap();
        let board = game.board;
        #[rustfmt::skip]
        let white_perspective = Tensor::of_slice(&[
            // flats
            -1., 0., 0.,
            0., 1., -1.,
            0., 1., 1.,
            // walls
            0., 0., 0.,
            -1., 0., 0.,
            0., 0., 0.,
            // caps
            0., 0., 0.,
            0., 0., 0.,
            0., 0., 0.,
            // layer 2
            0., 0., 0.,
            0., 0., 0.,
            0., 0., 0.,
            // layer 3
            0., 0., 0.,
            0., 0., 0.,
            0., 0., 0.,
            // layer 4 (since STACK_DEPTH_BEYOND_CARRY = 1)
            0., 0., 0.,
            0., 0., 0.,
            0., 0., 0.,
            // white to move
            1., 1., 1.,
            1., 1., 1.,
            1., 1., 1.0f32,
        ]).reshape(&dims(3));
        assert!(eq(&board_repr(&board, Colour::White), &white_perspective));
        assert!(eq(&board_repr(&board, Colour::Black), &-white_perspective));
    }

    #[test]
    fn with_stacks() {
        let game = Game::<3>::from_ptn(
            "
        1. a1 b1
        2. b1< Sa2
        3. b1 a2-
        4. a2 a3
        5. a2+ Sb3
        6. a2 b2
        7. a2> c2
        8. b1+ a2",
        )
        .unwrap();
        let board = game.board;
        #[rustfmt::skip]
        let white_perspective = Tensor::of_slice(&[
            // flats
            0., 0., 0.,
            -1., 1., -1.,
            1., 0., 0.,
            // walls
            -1., 0., 0.,
            0., 0., 0.,
            0., -1., 0.,
            // caps
            0., 0., 0.,
            0., 0., 0.,
            0., 0., 0.,
            // layer 2
            1., 0., 0.,
            0., 1., 0.,
            -1., 0., 0.,
            // layer 3
            -1., 0., 0.,
            0., -1., 0.,
            0., 0., 0.,
            // layer 4 (since STACK_DEPTH_BEYOND_CARRY = 1)
            0., 0., 0.,
            0., 0., 0.,
            0., 0., 0.,
            // white to move
            1., 1., 1.,
            1., 1., 1.,
            1., 1., 1.0f32,
        ]).reshape(&dims(3));
        assert!(eq(&board_repr(&board, Colour::White), &white_perspective));
        assert!(eq(&board_repr(&board, Colour::Black), &-white_perspective));
    }
}
