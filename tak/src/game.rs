use std::{cmp::Ordering, ops::Not};

use arrayvec::ArrayVec;
use takparse::{Color, Direction, Move, MoveKind, Pattern, Piece, Square};

use crate::{board::Board, game_result::GameResult, tile::Tile, StrResult};

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

const PLY_LIMIT: u16 = 240;

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
    pub result: GameResult,
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
            result: GameResult::default(),
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

    pub fn from_ptn_moves(moves: &[&str]) -> StrResult<Game<N>> {
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
    // TODO try to do all checks before applying the move.
    pub fn play(&mut self, my_move: Move) -> StrResult<GameResult> {
        match my_move.kind() {
            MoveKind::Place(piece) => self.execute_place(my_move.square(), piece),
            MoveKind::Spread(direction, pattern) => self.execute_spread(my_move.square(), direction, pattern),
        }?;
        self.ply += 1;
        self.result = self.game_result();
        self.to_move = self.to_move.not();
        Ok(self.result)
    }

    fn execute_place(&mut self, square: Square, piece: Piece) -> StrResult<()> {
        let (stones, caps) = self.get_counts();
        if !self.board[square].is_empty() {
            Err(format!(
                "cannot place a piece in that position because it is already occupied, square={square}"
            ))
        } else if matches!(piece, Piece::Cap) && (caps == 0) {
            Err(format!(
                "there is no capstone to play, white=({}, {}), black=({}, {})",
                self.white_stones, self.white_caps, self.black_stones, self.black_caps
            ))
        } else if matches!(piece, Piece::Flat | Piece::Wall) && (stones == 0) {
            Err(format!(
                "cannot play a stone without stones, white=({}, {}), black=({}, {})",
                self.white_stones, self.white_caps, self.black_stones, self.black_caps
            ))
        } else if self.is_swapped() && matches!(piece, Piece::Wall | Piece::Cap) {
            Err(format!(
                "cannot play a wall or capstone on the first two plies, ply={}",
                self.ply
            ))
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

    fn execute_spread(&mut self, square: Square, direction: Direction, pattern: Pattern) -> StrResult<()> {
        if self.board[square]
            .top()
            .ok_or("cannot move from an empty square")?
            .1
            != self.color()
        {
            return Err("cannot move a stack that you do not own".to_string());
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
                .ok_or("spread would leave the board")?;
            for _ in 0..drop_count {
                self.board[pos].stack(pieces.pop().unwrap(), carry.pop().unwrap())?;
            }
        }
        assert!(pieces.is_empty());
        assert!(carry.is_empty());
        Ok(())
    }

    fn game_result(&mut self) -> GameResult {
        if self.board.find_paths(self.to_move) {
            GameResult::Winner {
                color: self.to_move,
                road: true,
            }
        } else if self.board.find_paths(self.to_move.not()) {
            GameResult::Winner {
                color: self.to_move.not(),
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
                        GameResult::Draw { turn_limit: false }
                    } else {
                        GameResult::Winner {
                            color: Color::Black,
                            road: false,
                        }
                    }
                }
            }
        } else if self.ply >= PLY_LIMIT {
            GameResult::Draw { turn_limit: true }
        } else {
            GameResult::Ongoing
        }
    }
}
