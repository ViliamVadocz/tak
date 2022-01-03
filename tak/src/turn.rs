use arrayvec::ArrayVec;

use crate::board::{Piece, Tile};

#[derive(Clone, Copy, Debug)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Debug)]
pub enum Turn<const N: usize> {
    Place {
        pos: Pos,
        piece: Piece,
    },
    Move {
        pos: Pos,
        // at most N drops because of carry limit and you have to drop at least one
        drops: ArrayVec<(Pos, Tile), N>,
    },
}
