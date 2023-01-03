use takparse::{Move, ParseMoveError};

use crate::{reserves::Reserves, Game, GameResult, PlayError};

impl<const N: usize, const HALF_KOMI: i8> Game<N, HALF_KOMI>
where
    Reserves<N>: Default,
{
    /// Creates a default game and plays the supplied moves.
    ///
    /// # Errors
    ///
    /// If a move is invalid or the game is over when trying to
    /// play a move, a [`PlayError`] is returned.
    pub fn from_moves(moves: &[Move]) -> Result<Self, PlayError> {
        let mut game = Self::default();
        for mov in moves {
            if game.result() != GameResult::Ongoing {
                return Err(PlayError::GameOver);
            }
            game.play(*mov)?;
        }
        Ok(game)
    }

    /// Creates a default game and plays all the
    /// moves after parsing them.
    ///
    /// # Panics
    ///
    /// Panics if there is an error in parsing or if any
    /// move is invalid during play.
    #[must_use]
    pub fn from_ptn_moves(moves: &[&str]) -> Self {
        let moves = ptn_to_moves(moves).unwrap();
        Self::from_moves(&moves).unwrap()
    }
}

pub fn ptn_to_moves(moves: &[&str]) -> Result<Box<[Move]>, ParseMoveError> {
    moves.iter().map(|mov| mov.parse()).collect()
}

#[cfg(test)]
mod tests {
    use crate::{ptn::ptn_to_moves, Game, PlayError};

    #[test]
    fn game_ends() {
        let moves = ptn_to_moves(&[
            "a1", "e5", "a3", "b1", "c3", "c1", "e3", "d1", "d3", "e1", "b5",
        ])
        .unwrap();
        assert_eq!(Game::<5, 0>::from_moves(&moves), Err(PlayError::GameOver));
    }
}
