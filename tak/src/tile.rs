use std::iter::once;

use arrayvec::ArrayVec;

use crate::{colour::Colour, StrResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Flat,
    Wall,
    Capstone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub colour: Colour,
    pub shape: Shape,
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub top: Piece,
    pub stack: Option<Vec<Colour>>,
}

impl Tile {
    pub fn size(&self) -> usize {
        1 + self.stack.as_ref().map(|v| v.len()).unwrap_or_default()
    }

    pub fn stack(self, piece: Piece) -> StrResult<Self> {
        // Only allow stacking on top of flats, or flattening walls.

        match self.top.shape {
            Shape::Flat => Ok(()),
            Shape::Wall => {
                if matches!(piece.shape, Shape::Capstone) {
                    Ok(())
                } else {
                    Err("can only stack on top of a wall with a capstone")
                }
            }
            Shape::Capstone => Err("cannot create a stack on top of a capstone"),
        }?;

        let mut stack = self.stack.unwrap_or_default();
        stack.push(self.top.colour);
        Ok(Tile {
            top: piece,
            stack: Some(stack),
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
            .unwrap_or_default()
            .into_iter()
            .map(|colour| Piece {
                colour,
                shape: Shape::Flat,
            })
            .chain(once(self.top))
            .rev();

        let carry = stack.by_ref().take(amount).collect();

        let left = stack.next().map(|top| Tile {
            top,
            stack: if count - amount == 1 {
                None
            } else {
                Some(stack.map(|p| p.colour).collect())
            },
        });
        Ok((left, carry))
    }
}
