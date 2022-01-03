use std::ops::Sub;

use arrayvec::ArrayVec;

use crate::tile::Piece;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

impl Sub for Pos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Pos {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
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
        drops: ArrayVec<(Pos, Piece), N>,
    },
}
