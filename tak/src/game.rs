use std::ops::Not;

use takparse::{Color, Direction, Move, MoveKind, Pattern, Piece, Square};

use crate::{board::Board, error::PlayError, reserves::Reserves, stack::Stack};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Game<const N: usize, const HALF_KOMI: i8> {
    pub(crate) board: Board<N>,
    pub(crate) to_move: Color,
    pub(crate) white_reserves: Reserves<N>,
    pub(crate) black_reserves: Reserves<N>,
    pub(crate) ply: u16,
    pub(crate) reversible_plies: u16,
}

impl<const N: usize, const HALF_KOMI: i8> Default for Game<N, HALF_KOMI>
where
    Reserves<N>: Default,
{
    fn default() -> Self {
        Self {
            board: Board::default(),
            to_move: Color::White,
            white_reserves: Reserves::default(),
            black_reserves: Reserves::default(),
            ply: u16::default(),
            reversible_plies: u16::default(),
        }
    }
}

impl<const N: usize, const HALF_KOMI: i8> Game<N, HALF_KOMI> {
    pub const fn board(&self) -> Board<N> {
        self.board
    }

    pub const fn to_move(&self) -> Color {
        self.to_move
    }

    pub(crate) const fn is_swapped(&self) -> bool {
        self.ply < 2
    }

    pub(crate) fn color_to_place(&self) -> Color {
        if self.is_swapped() {
            self.to_move.not()
        } else {
            self.to_move
        }
    }

    pub(crate) fn get_reserves(&self) -> Reserves<N> {
        match self.color_to_place() {
            Color::White => self.white_reserves,
            Color::Black => self.black_reserves,
        }
    }

    fn dec_stones(&mut self) {
        if (self.to_move == Color::White) ^ self.is_swapped() {
            self.white_reserves.stones -= 1;
        } else {
            self.black_reserves.stones -= 1;
        }
    }

    fn dec_caps(&mut self) {
        match self.to_move {
            Color::White => self.white_reserves.caps -= 1,
            Color::Black => self.black_reserves.caps -= 1,
        }
    }

    /// Play a move on the board.
    ///
    /// # Errors
    ///
    /// In case the move is invalid an error is returned and the game
    /// might be in an invalid state.
    pub fn play(&mut self, my_move: Move) -> Result<(), PlayError> {
        match my_move.kind() {
            MoveKind::Place(piece) => self.execute_place(my_move.square(), piece),
            MoveKind::Spread(direction, pattern) => {
                self.execute_spread(my_move.square(), direction, pattern)
            }
        }?;
        self.update_reversible(my_move);
        self.ply += 1;
        self.to_move = self.to_move.not();
        Ok(())
    }

    fn execute_place(&mut self, square: Square, piece: Piece) -> Result<(), PlayError> {
        let Reserves { stones, caps } = self.get_reserves();
        let is_swapped = self.is_swapped();
        let color_to_place = self.color_to_place();
        let stack = self.board.get_mut(square).ok_or(PlayError::OutOfBounds)?;
        if !stack.is_empty() {
            Err(PlayError::AlreadyOccupied)
        } else if matches!(piece, Piece::Cap) && (caps == 0) {
            Err(PlayError::NoCapstone)
        } else if matches!(piece, Piece::Flat | Piece::Wall) && (stones == 0) {
            Err(PlayError::NoStones)
        } else if is_swapped && matches!(piece, Piece::Wall | Piece::Cap) {
            Err(PlayError::OpeningNonFlat)
        } else {
            *stack = Stack::new(piece, color_to_place);
            if matches!(piece, Piece::Flat | Piece::Wall) {
                self.dec_stones();
            } else {
                self.dec_caps();
            }
            Ok(())
        }
    }

    fn execute_spread(
        &mut self,
        square: Square,
        direction: Direction,
        pattern: Pattern,
    ) -> Result<(), PlayError> {
        let n = N as u8;

        let stack = self.board.get_mut(square).ok_or(PlayError::OutOfBounds)?;
        let (_, top_color) = stack.top().ok_or(PlayError::EmptySquare)?;
        if top_color != self.to_move {
            return Err(PlayError::StackNotOwned);
        }
        let mut amount = pattern.count_pieces();
        let (piece, colors) = stack.take::<N>(amount)?;
        let mut carry = colors.into_iter();

        // For each square in the spread, stack dropped pieces.
        let mut pos = square;
        for drop_count in pattern.drop_counts() {
            pos = pos
                .checked_step(direction, n)
                .ok_or(PlayError::SpreadOutOfBounds)?;

            for color in carry.by_ref().take(drop_count as usize) {
                amount -= 1;
                // unwrap is sound since we checked whether it is on the board earlier
                let Some(stack) = self.board.get_mut(pos) else { continue; };
                stack.stack(if amount > 0 { Piece::Flat } else { piece }, color)?;
            }
        }

        assert_eq!(amount, 0);
        assert_eq!(carry.next(), None);
        Ok(())
    }

    fn update_reversible(&mut self, my_move: Move) {
        // TODO detect smashes
        if matches!(my_move.kind(), MoveKind::Place(_)) {
            self.reversible_plies = 0;
        } else {
            self.reversible_plies += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Game, PlayError, StackError, TakeError};

    #[test]
    fn square_out_of_bounds() {
        let mut game = Game::<5, 0>::default();
        let my_move = "a8".parse().unwrap();
        assert_eq!(game.play(my_move), Err(PlayError::OutOfBounds));
    }

    #[test]
    fn already_occupied() {
        let mut game = Game::<5, 0>::from_ptn_moves(&["a2"]);
        let my_move = "a2".parse().unwrap();
        assert_eq!(game.play(my_move), Err(PlayError::AlreadyOccupied));
    }

    #[test]
    fn no_capstone() {
        let mut game = Game::<6, 0>::from_ptn_moves(&["a1", "b1", "Ca2", "b2"]);
        let my_move = "Ca3".parse().unwrap();
        assert_eq!(game.play(my_move), Err(PlayError::NoCapstone));
    }

    #[test]
    fn no_stones() {
        let mut game = Game::<3, 0>::from_ptn_moves(&[
            "a1", "b1", "a2", "b2", "a2-", "a2", "b1<", "b1", "3a1>", "a1", "b1<", "c1", "3b1>",
            "b1", "2a1>", "a1", "3b1<", "b1", "c2", "c3",
        ]);
        // technically the game is over so maybe this should be an error?
        assert_eq!(game.play("c2<".parse().unwrap()), Ok(()));
        let my_move = "c2".parse().unwrap();
        assert_eq!(game.play(my_move), Err(PlayError::NoStones));
    }

    #[test]
    fn opening_wall() {
        let mut game = Game::<7, 0>::default();
        let my_move = "Sa1".parse().unwrap();
        assert_eq!(game.play(my_move), Err(PlayError::OpeningNonFlat));
    }

    #[test]
    fn opening_capstone() {
        let mut game = Game::<8, 0>::default();
        let my_move = "Ca1".parse().unwrap();
        assert_eq!(game.play(my_move), Err(PlayError::OpeningNonFlat));
    }

    #[test]
    fn empty_square() {
        let mut game = Game::<4, 0>::from_ptn_moves(&["a1", "b1", "a2"]);
        let my_move = "b2<".parse().unwrap();
        assert_eq!(game.play(my_move), Err(PlayError::EmptySquare));
    }

    #[test]
    fn stack_not_owned() {
        let mut game = Game::<5, 0>::from_ptn_moves(&[
            "a1", "e5", "c3", "b2", "c2", "c1", "d1", "Cd2", "b1", "c1+", "Cc1", "e2", "c1+",
        ]);
        let my_move = "3c2<12".parse().unwrap();
        assert_eq!(game.play(my_move), Err(PlayError::StackNotOwned));
    }

    #[test]
    fn spread_out_of_bounds() {
        let mut game = Game::<5, 0>::from_ptn_moves(&[
            "a1", "e5", "c3", "b2", "c2", "c1", "d1", "Cd2", "b1", "c1+", "Cc1", "e2", "c1+", "a2",
        ]);
        let my_move = "3c2-111".parse().unwrap();
        assert_eq!(game.play(my_move), Err(PlayError::SpreadOutOfBounds));
    }

    #[test]
    fn stack_on_wall() {
        let mut game = Game::<4, 0>::from_ptn_moves(&["a1", "a2", "Sb1"]);
        let my_move = "a1>".parse().unwrap();
        assert_eq!(
            game.play(my_move),
            Err(PlayError::StackError(StackError::Wall))
        );
    }

    #[test]
    fn stack_on_cap() {
        let mut game = Game::<7, 0>::from_ptn_moves(&["a1", "a2", "Cb1"]);
        let my_move = "a1>".parse().unwrap();
        assert_eq!(
            game.play(my_move),
            Err(PlayError::StackError(StackError::Cap))
        );
    }

    #[test]
    fn carry_limit() {
        let mut game =
            Game::<3, 0>::from_ptn_moves(&["a1", "b1", "b1<", "b1", "2a1>", "a1", "3b1<", "b1"]);
        let my_move = "4a1>".parse().unwrap();
        assert_eq!(
            game.play(my_move),
            Err(PlayError::TakeError(TakeError::CarryLimit))
        );
    }

    #[test]
    fn take_more_than_stack() {
        let mut game = Game::<3, 0>::from_ptn_moves(&["a1", "b1", "b1<", "b1"]);
        let my_move = "3a1>".parse().unwrap();
        assert_eq!(
            game.play(my_move),
            Err(PlayError::TakeError(TakeError::StackSize))
        );
    }
}
