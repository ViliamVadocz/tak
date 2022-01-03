use std::ops::Sub;

use arrayvec::ArrayVec;

use crate::tile::Piece;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

impl Pos {
    pub fn neighbors<const N: usize>(self) -> ArrayVec<Pos, 4> {
        let Pos { x, y } = self;
        let mut neighbors = ArrayVec::new();
        if x > 0 {
            neighbors.push(Pos { x: x - 1, y });
        }
        if y > 0 {
            neighbors.push(Pos { x, y: y - 1 });
        }
        if x < N - 1 {
            neighbors.push(Pos { x: x + 1, y });
        }
        if y < N - 1 {
            neighbors.push(Pos { x, y: y + 1 });
        }
        neighbors
    }
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
