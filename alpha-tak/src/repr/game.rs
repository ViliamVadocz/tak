use tak::*;
use tch::{kind::FLOAT_CPU, Tensor};

use super::{
    board::{board_channels, board_repr},
    reserves::reserves_repr,
};

const COLOUR_CHANNEL: usize = 1;
const FCD_CHANNEL: usize = 1;

pub const fn input_channels(n: usize) -> usize {
    let (stones, capstones) = default_starting_stones(n);
    board_channels(n) + COLOUR_CHANNEL + FCD_CHANNEL + 2 * stones as usize + 2 * capstones as usize
}

/// Create a tensor which represents the game to be used
/// as input for the network.
pub fn game_repr<const N: usize>(game: &Game<N>) -> Tensor {
    let board = board_repr(&game.board, game.to_move);

    let layer_shape = [1, N as i64, N as i64];

    // one-hot encoded reserves
    let (my_stones, en_stones, my_caps, en_caps) = reserves_repr(game);

    // layer for whose turn it is
    let colour_layer = Tensor::full(
        &layer_shape,
        if game.to_move == Color::White { 1. } else { 0. },
        FLOAT_CPU,
    );

    // layer for fcd (+ komi)
    let fcd = game.board.flat_diff() - game.half_komi / 2;
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
