use arrayvec::ArrayVec;
use takparse::{Color, Piece};

use crate::StrResult;

#[derive(Clone, Debug, Default)]
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
    pub fn stack(&mut self, piece: Piece, color: Color) -> StrResult<()> {
        // Only allow stacking on top of flats, or flattening walls.
        match self.piece {
            Piece::Flat => Ok(()),
            Piece::Wall => {
                if matches!(piece, Piece::Cap) {
                    Ok(())
                } else {
                    Err("can only flatten a wall with a capstone")
                }
            }
            Piece::Cap => Err("cannot create a stack on top of a capstone"),
        }?;

        self.piece = piece;
        self.stack.push(color);
        Ok(())
    }

    /// Try taking the top `amount` pieces from this tile.
    /// Returned ArrayVec is ordered top to bottom.
    pub fn take<const N: usize>(&mut self, amount: usize) -> StrResult<(Piece, ArrayVec<Color, N>)> {
        if amount == 0 {
            return Err("cannot take 0 from a tile".to_string());
        } else if amount > N {
            return Err(format!("cannot take more than the carry limit, amount={amount}"));
        } else if amount > self.size() {
            return Err(format!(
                "cannot take more pieces than there are on the tile, amount={amount}, count={}",
                self.size()
            ));
        }

        let mut stack = std::mem::take(&mut self.stack).into_iter().rev();
        let carry = stack.by_ref().take(amount).collect();
        let piece = std::mem::take(&mut self.piece);
        self.stack = stack.rev().collect();
        Ok((piece, carry))
    }
}
