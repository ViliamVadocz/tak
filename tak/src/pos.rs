use std::{cmp::Ordering, ops::Sub};

use arrayvec::ArrayVec;

use crate::StrResult;

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
    type Output = StrResult<Direction>;

    fn sub(self, rhs: Self) -> Self::Output {
        let diagonal_err = Err("cannot have a diagonal direction");
        match self.x.cmp(&rhs.x) {
            Ordering::Greater => match self.y.cmp(&rhs.y) {
                Ordering::Equal => Ok(Direction::NegX),
                Ordering::Less | Ordering::Greater => diagonal_err,
            },
            Ordering::Less => match self.y.cmp(&rhs.y) {
                Ordering::Equal => Ok(Direction::PosX),
                Ordering::Less | Ordering::Greater => diagonal_err,
            },
            Ordering::Equal => match self.y.cmp(&rhs.y) {
                Ordering::Greater => Ok(Direction::PosY),
                Ordering::Less => Ok(Direction::NegY),
                Ordering::Equal => Err("cannot decide when positions are the same"),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    PosX,
    PosY,
    NegX,
    NegY,
}
