use takparse::{Color, Piece};

use crate::{
    colors::Colors,
    error::{StackError, TakeError},
};

#[derive(Clone, Copy, Debug, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stack {
    piece: Piece,
    colors: Colors,
}

impl Stack {
    /// Create a new stack with a single piece.
    pub const fn new(piece: Piece, color: Color) -> Self {
        Self {
            piece,
            colors: Colors::of_one(color),
        }
    }

    /// Create a new stack with the given colors.
    pub fn exact(piece: Piece, colors: Colors) -> Self {
        assert!(!colors.is_empty());
        Self { piece, colors }
    }

    /// Check if anything is in this stack. If false, it means the square is
    /// empty.
    pub const fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    /// Get the size of the stack.
    pub const fn size(&self) -> u32 {
        self.colors.len()
    }

    /// Get the top piece and color of this stack.
    pub fn top(&self) -> Option<(Piece, Color)> {
        self.colors.top().map(|color| (self.piece, color))
    }

    pub const fn colors(&self) -> Colors {
        self.colors
    }

    /// Check if this stack contributes to roads for the given color.
    pub fn road(&self, color: Color) -> bool {
        matches!(self.top(), Some((Piece::Flat | Piece::Cap, c)) if c == color)
    }

    /// Try to put a piece on top of this stack.
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
        self.colors.push(color);
        Ok(())
    }

    /// Try taking the top `amount` pieces from this tile.
    pub fn take<const N: usize>(&mut self, amount: u32) -> Result<(Piece, Colors), TakeError> {
        if amount == 0 {
            return Err(TakeError::Zero);
        } else if amount as usize > N {
            return Err(TakeError::CarryLimit);
        } else if amount > self.size() {
            return Err(TakeError::StackSize);
        }

        let piece = self.piece;
        self.piece = Piece::Flat;
        Ok((piece, self.colors.take(amount).unwrap()))
    }
}
