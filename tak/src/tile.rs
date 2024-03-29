use arrayvec::ArrayVec;
use takparse::{Color, Piece};

use crate::error::{StackError, TakeError};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tile {
    pub piece: Piece,
    pub stack: Vec<Color>,
}

impl Tile {
    /// Get whether there is a stack on this tile.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Get the number of pieces on this tile.
    pub fn size(&self) -> usize {
        self.stack.len()
    }

    pub fn top(&self) -> Option<(Piece, Color)> {
        self.stack.last().map(|&color| (self.piece, color))
    }

    /// Try to stack the piece on this tile.
    pub fn stack(&mut self, piece: Piece, color: Color) -> Result<(), StackError> {
        // Only allow stacking on top of flats, or flattening walls.
        match self.piece {
            Piece::Flat => Ok(()),
            Piece::Wall => {
                if matches!(piece, Piece::Cap) {
                    Ok(())
                } else {
                    Err(StackError::Wall)
                }
            }
            Piece::Cap => Err(StackError::Cap),
        }?;

        self.piece = piece;
        self.stack.push(color);
        Ok(())
    }

    /// Try taking the top `amount` pieces from this tile.
    /// Returned ArrayVec is ordered top to bottom.
    pub fn take<const N: usize>(&mut self, amount: usize) -> Result<(Piece, ArrayVec<Color, N>), TakeError> {
        if amount == 0 {
            return Err(TakeError::Zero);
        } else if amount > N {
            return Err(TakeError::CarryLimit);
        } else if amount > self.size() {
            return Err(TakeError::StackSize(self.size()));
        }

        let mut stack = std::mem::take(&mut self.stack).into_iter().rev();
        let carry = stack.by_ref().take(amount).collect();
        let piece = std::mem::take(&mut self.piece);
        self.stack = stack.rev().collect();
        Ok((piece, carry))
    }
}
