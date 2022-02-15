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

#[derive(Clone, Debug, PartialEq, Eq)]
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

    /// Get the number of pieces on this tile.
    pub fn size(&self) -> usize {
        1 + self.stack.len()
    }

    /// Try to stack the piece on this tile.
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

    /// Try taking the top `amount` pieces from this tile.
    /// Returned ArrayVec is ordered top to bottom.
    pub fn take<const N: usize>(self, amount: usize) -> StrResult<(Option<Tile>, ArrayVec<Piece, N>)> {
        let count = self.size();
        if amount == 0 {
            return Err("cannot take 0 from a tile".to_string());
        } else if amount > N {
            return Err(format!("cannot take more than the carry limit, amount={amount}"));
        } else if amount > count {
            return Err(format!(
                "cannot take more pieces than there are on the tile, amount={amount}, count={count}"
            ));
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

        let carry = stack.by_ref().take(amount).collect();

        let left = stack.next().map(|top| Tile {
            top,
            stack: stack.rev().map(|p| p.colour).collect(),
        });
        Ok((left, carry))
    }
}
