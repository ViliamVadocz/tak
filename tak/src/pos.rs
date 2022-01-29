use std::{cmp::Ordering, ops::Sub};

use arrayvec::ArrayVec;

use crate::StrResult;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pos<const N: usize> {
    pub x: usize,
    pub y: usize,
}

impl<const N: usize> Pos<N> {
    pub fn neighbors(self) -> ArrayVec<Pos<N>, 4> {
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

    pub fn step(self, direction: Direction) -> Option<Pos<N>> {
        self.neighbors()
            .into_iter()
            .find(|&n| (n - self).unwrap() == direction)
    }

    /// rotates a position 1 quarter turn counterclockwise
    #[must_use]
    pub const fn rotate(&self) -> Self {
        Pos {
            x: self.y,
            y: N - 1 - self.x,
        }
    }

    /// mirror along the x axis
    #[must_use]
    pub const fn mirror(&self) -> Self {
        Pos {
            x: self.x,
            y: N - 1 - self.y,
        }
    }
}

impl<const N: usize> Sub for Pos<N> {
    type Output = StrResult<Direction>;

    fn sub(self, rhs: Self) -> Self::Output {
        let diagonal_err = Err("cannot have a diagonal direction");
        match self.x.cmp(&rhs.x) {
            Ordering::Greater => match self.y.cmp(&rhs.y) {
                Ordering::Equal => Ok(Direction::PosX),
                Ordering::Less | Ordering::Greater => diagonal_err,
            },
            Ordering::Less => match self.y.cmp(&rhs.y) {
                Ordering::Equal => Ok(Direction::NegX),
                Ordering::Less | Ordering::Greater => diagonal_err,
            },
            Ordering::Equal => match self.y.cmp(&rhs.y) {
                Ordering::Greater => Ok(Direction::PosY),
                Ordering::Less => Ok(Direction::NegY),
                Ordering::Equal => Err("cannot decide direction when positions are the same"),
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

impl Direction {
    /// rotates a direction 1 quarter turn counterclockwise
    #[must_use]
    pub const fn rotate(&self) -> Self {
        match self {
            Direction::PosX => Direction::NegY,
            Direction::PosY => Direction::PosX,
            Direction::NegX => Direction::PosY,
            Direction::NegY => Direction::NegX,
        }
    }

    /// mirror along the x axis
    #[must_use]
    pub const fn mirror(&self) -> Self {
        match self {
            Direction::PosX => Direction::PosX,
            Direction::PosY => Direction::NegY,
            Direction::NegX => Direction::NegX,
            Direction::NegY => Direction::PosY,
        }
    }
}
