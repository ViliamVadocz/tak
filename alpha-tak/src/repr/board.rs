use tak::*;
use tch::{kind::FLOAT_CPU, Tensor};

pub const STACK_DEPTH_BEYOND_CARRY: usize = 6;

pub const fn board_channels(n: usize) -> usize {
    (n + 2 + STACK_DEPTH_BEYOND_CARRY) * 2
}

/// Creates a tensor which represents the board
/// from the perspective of the current player.
pub fn board_repr<const N: usize>(board: &Board<N>, to_move: Color) -> Tensor {
    // track the positions of all ones in the input tensor
    let mut positions = Vec::new();

    for y in 0..N {
        for x in 0..N {
            let board_offset = N * y + x;
            let square = Square::new(x as u8, y as u8);
            let tile = &board[square];

            // top layer of stack has 6 channels in total
            // 2 for flats (1 per player)
            // 2 for walls (1 per player)
            // 2 for caps (1 per player)
            if let Some((piece, color)) = tile.top() {
                let channel = match piece {
                    Piece::Flat => 0,
                    Piece::Wall => 2,
                    Piece::Cap => 4,
                } + if color == to_move { 0 } else { 1 };
                positions.push((board_offset + N * N * channel) as i64);
            }

            // the rest of the layers alternate mine and opponent's tiles
            for (i, &color) in tile
                .stack
                .iter()
                .take(N + STACK_DEPTH_BEYOND_CARRY)
                .skip(1)
                .enumerate()
            {
                let channel = 6 + i + if color == to_move { 0 } else { 1 };
                positions.push((board_offset + N * N * channel) as i64);
            }
        }
    }

    let index = Tensor::of_slice(&positions);
    let ones = Tensor::ones(&[positions.len() as i64], FLOAT_CPU);
    let mut zeros = Tensor::zeros(&[board_channels(N) as i64, N as i64, N as i64], FLOAT_CPU);
    zeros.put_(&index, &ones, false)
}
