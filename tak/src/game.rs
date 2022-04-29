use std::{cmp::Ordering, ops::Not};

use arrayvec::ArrayVec;
use takparse::{Color, Direction, Move, MoveKind, Pattern, Piece, Square};

use crate::{board::Board, error::PlayError, game_result::GameResult, tile::Tile};

type Stones = u8;
type Capstones = u8;
pub const fn default_starting_stones(width: usize) -> (Stones, Capstones) {
    match width {
        3 => (10, 0),
        4 => (15, 0),
        5 => (21, 1),
        6 => (30, 1),
        7 => (40, 2),
        8 => (50, 2),
        _ => panic!("missing starting stones for non-standard board size"),
    }
}

const REVERSIBLE_PLIES: u8 = 20; // playtak uses 50

#[derive(Clone, Debug)]
pub struct Game<const N: usize> {
    pub board: Board<N>,
    pub to_move: Color,
    pub ply: u16,
    pub white_stones: u8,
    pub white_caps: u8,
    pub black_stones: u8,
    pub black_caps: u8,
    pub half_komi: i8,
    pub reversible_plies: u8,
}

impl<const N: usize> Default for Game<N> {
    /// Create a new game with the default amount of starting stones for the
    /// board size and no komi.
    fn default() -> Self {
        let (stones, caps) = default_starting_stones(N);
        Game {
            board: Board::default(),
            to_move: Color::White,
            ply: 0,
            white_stones: stones,
            white_caps: caps,
            black_stones: stones,
            black_caps: caps,
            half_komi: 0,
            reversible_plies: 0,
        }
    }
}

impl<const N: usize> Game<N> {
    /// Create a game with komi.
    pub fn with_komi(komi: i8) -> Self {
        Game {
            half_komi: komi * 2,
            ..Default::default()
        }
    }

    /// Create a game with half komi.
    /// This a 0 flat count difference a win instead of a draw.
    pub fn with_half_komi(half_komi: i8) -> Self {
        Game {
            half_komi,
            ..Default::default()
        }
    }

    /// Create a game from a list of PTN moves.
    /// Assumes the moves are correct PTN notation.
    pub fn from_ptn_moves(moves: &[&str]) -> Result<Game<N>, PlayError> {
        let mut game = Game::default();
        for m in moves {
            game.play(m.parse().unwrap())?;
        }
        Ok(game)
    }

    pub(crate) fn is_swapped(&self) -> bool {
        self.ply < 2
    }

    pub(crate) fn color(&self) -> Color {
        if self.is_swapped() {
            self.to_move.not()
        } else {
            self.to_move
        }
    }

    pub(crate) fn get_counts(&self) -> (Stones, Capstones) {
        match self.to_move {
            Color::White => (self.white_stones, self.white_caps),
            Color::Black => (self.black_stones, self.black_caps),
        }
    }

    fn dec_stones(&mut self) {
        match self.to_move {
            Color::White => self.white_stones -= 1,
            Color::Black => self.black_stones -= 1,
        }
    }

    fn dec_caps(&mut self) {
        match self.to_move {
            Color::White => self.white_caps -= 1,
            Color::Black => self.black_caps -= 1,
        }
    }

    /// Play a move on the board. Returns the updated game result.
    /// In case the move is invalid an error is returned and the game
    /// might be in an invalid state.
    pub fn play(&mut self, my_move: Move) -> Result<(), PlayError> {
        match my_move.kind() {
            MoveKind::Place(piece) => self.execute_place(my_move.square(), piece),
            MoveKind::Spread(direction, pattern) => self.execute_spread(my_move.square(), direction, pattern),
        }?;
        self.update_reversible(my_move);
        self.ply += 1;
        self.to_move = self.to_move.not();
        Ok(())
    }

    /// Play a move, except if an error occurs, revert to the game
    /// state before the move was played. This should be used
    /// when the move passed in cannot be trusted (such as user input).
    /// Returns the backed up game-state if the move worked.
    pub fn safe_play(&mut self, my_move: Move) -> Result<Self, PlayError> {
        let backup = self.clone();
        let result = self.play(my_move);
        if let Err(err) = result {
            *self = backup;
            Err(err)
        } else {
            Ok(backup)
        }
    }

    fn execute_place(&mut self, square: Square, piece: Piece) -> Result<(), PlayError> {
        let (stones, caps) = self.get_counts();
        if !self.board.get(square).ok_or(PlayError::OutOfBounds)?.is_empty() {
            Err(PlayError::AlreadyOccupied)
        } else if matches!(piece, Piece::Cap) && (caps == 0) {
            Err(PlayError::NoCapstone)
        } else if matches!(piece, Piece::Flat | Piece::Wall) && (stones == 0) {
            Err(PlayError::NoStones)
        } else if self.is_swapped() && matches!(piece, Piece::Wall | Piece::Cap) {
            Err(PlayError::OpeningNonFlat)
        } else {
            self.board[square] = Tile {
                piece,
                stack: vec![self.color()],
            };
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
        if self
            .board
            .get(square)
            .ok_or(PlayError::OutOfBounds)?
            .top()
            .ok_or(PlayError::EmptySquare)?
            .1
            != self.color()
        {
            return Err(PlayError::StackNotOwned);
        }

        let (piece, mut carry) = self.board[square].take::<N>(pattern.count_pieces() as usize)?;

        let mut pieces: ArrayVec<Piece, N> = ArrayVec::new();
        pieces.push(piece);
        for _ in 0..(pattern.count_pieces() - 1) {
            pieces.push(Piece::Flat);
        }

        let mut pos = square;
        for drop_count in pattern.drop_counts() {
            pos = pos
                .checked_step(direction, N as u8)
                .ok_or(PlayError::SpreadOutOfBounds)?;
            for _ in 0..drop_count {
                self.board[pos].stack(pieces.pop().unwrap(), carry.pop().unwrap())?;
            }
        }
        assert!(pieces.is_empty());
        assert!(carry.is_empty());
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

    pub fn result(&self) -> GameResult {
        // We check the result after a move, so for the dragon clause
        // we look at the other player's path first (they just played).
        if self.board.find_paths(self.to_move.not()) {
            GameResult::Winner {
                color: self.to_move.not(),
                road: true,
            }
        } else if self.board.find_paths(self.to_move) {
            GameResult::Winner {
                color: self.to_move,
                road: true,
            }
        } else if self.white_caps == 0 && self.white_stones == 0
            || self.black_caps == 0 && self.black_stones == 0
            || self.board.full()
        {
            let flat_diff = self.board.flat_diff();
            match flat_diff.cmp(&(self.half_komi / 2)) {
                Ordering::Greater => GameResult::Winner {
                    color: Color::White,
                    road: false,
                },
                Ordering::Less => GameResult::Winner {
                    color: Color::Black,
                    road: false,
                },
                Ordering::Equal => {
                    if self.half_komi % 2 == 0 {
                        GameResult::Draw {
                            reversible_plies: false,
                        }
                    } else {
                        GameResult::Winner {
                            color: Color::Black,
                            road: false,
                        }
                    }
                }
            }
        } else if self.reversible_plies >= REVERSIBLE_PLIES {
            GameResult::Draw {
                reversible_plies: true,
            }
        } else {
            GameResult::Ongoing
        }
    }
}
