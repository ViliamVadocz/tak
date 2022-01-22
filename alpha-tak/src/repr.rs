use tak::{board::Board, colour::Colour, game::Game, pos::Pos, tile::Shape};
use tch::{Device, Tensor};

pub const fn input_dims(n: usize) -> [usize; 3] {
    // channels first
    [n + 3, n, n]
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
    let [d1, d2, d3] = input_dims(N);
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
    for n in 0..(d1 - 3) {
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

    Tensor::stack(&layers, 0).to_device(Device::cuda_if_available())
}

pub fn game_repr<const N: usize>(game: &Game<N>) -> Tensor {
    // TODO add other info such as komi, fcd, total stones, reserves
    board_repr(&game.board, game.to_move)
}
