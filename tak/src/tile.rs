use std::iter::once;

use arrayvec::ArrayVec;

use crate::{colour::Colour, StrResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Shape {
    Flat,
    Wall,
    Capstone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    pub colour: Colour,
    pub shape: Shape,
}

impl Shape {
    pub fn from_ptn(s: &str) -> Self {
        match s {
            "C" => Shape::Capstone,
            "S" => Shape::Wall,
            "" => Shape::Flat,
            _ => unreachable!(),
        }
    }

    pub fn to_ptn(&self) -> &str {
        match self {
            Shape::Flat => "",
            Shape::Wall => "S",
            Shape::Capstone => "C",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub top: Piece,
    pub stack: Vec<Colour>,
}

impl Tile {
    pub fn new(top: Piece) -> Self {
        Tile {
            top,
            stack: Vec::new(),
        }
    }

    pub fn size(&self) -> usize {
        1 + self.stack.len()
    }

    pub fn stack(mut self, piece: Piece) -> StrResult<Self> {
        // Only allow stacking on top of flats, or flattening walls.

        match self.top.shape {
            Shape::Flat => Ok(()),
            Shape::Wall => {
                if matches!(piece.shape, Shape::Capstone) {
                    Ok(())
                } else {
                    Err("can only flatten a wall with a capstone")
                }
            }
            Shape::Capstone => Err("cannot create a stack on top of a capstone"),
        }?;

        self.stack.push(self.top.colour);
        Ok(Tile {
            top: piece,
            stack: self.stack,
        })
    }

    pub fn take<const N: usize>(self, amount: usize) -> StrResult<(Option<Tile>, ArrayVec<Piece, N>)> {
        let count = self.size();
        if amount == 0 {
            return Err("cannot take 0 from a tile");
        } else if amount > N {
            return Err("cannot take more than the carry limit");
        } else if amount > count {
            return Err("cannot take more pieces than there are on the tile");
        }

        let mut stack = self
            .stack
            .into_iter()
            .map(|colour| Piece {
                colour,
                shape: Shape::Flat,
            })
            .chain(once(self.top))
            .rev();

        // carry is ordered from top to bottom, so if you want to drop one-by-one, use
        // pop or reverse it
        let carry = stack.by_ref().take(amount).collect();

        let left = stack.next().map(|top| Tile {
            top,
            stack: stack.rev().map(|p| p.colour).collect(),
        });
        Ok((left, carry))
    }
}
