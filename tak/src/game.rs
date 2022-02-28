use std::cmp::Ordering;

use arrayvec::ArrayVec;

use crate::{
    board::Board,
    colour::Colour,
    direction::Direction,
    pos::Pos,
    tile::{Piece, Shape, Tile},
    turn::Turn,
    StrResult,
};

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

const TURN_LIMIT: u64 = 400;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameResult {
    Winner { colour: Colour, road: bool },
    Draw { turn_limit: bool },
    Ongoing,
}

#[derive(Clone, Debug)]
pub struct Game<const N: usize> {
    pub board: Board<N>,
    pub to_move: Colour,
    pub ply: u64,
    pub white_stones: Stones,
    pub black_stones: Stones,
    pub white_caps: Capstones,
    pub black_caps: Capstones,
    pub komi: i32,
}

impl<const N: usize> Game<N>
where
    [[Option<Tile>; N]; N]: Default,
{
    pub fn with_komi(komi: i32) -> Self {
        Game {
            komi,
            ..Default::default()
        }
    }
}

impl<const N: usize> Default for Game<N>
where
    [[Option<Tile>; N]; N]: Default,
{
    fn default() -> Self {
        let (stones, capstones) = default_starting_stones(N);
        Self {
            board: Board::default(),
            to_move: Colour::White, // White picks the first move for Black
            ply: 0,
            white_stones: stones,
            black_stones: stones,
            white_caps: capstones,
            black_caps: capstones,
            komi: 0,
        }
    }
}

impl<const N: usize> Game<N> {
    pub fn swap(&self) -> bool {
        self.ply < 2
    }

    pub fn colour(&self) -> Colour {
        if self.swap() {
            self.to_move.next()
        } else {
            self.to_move
        }
    }

    pub fn opening(&mut self, opening_index: usize) -> StrResult<()> {
        if !self.board.empty() || self.ply != 0 {
            return Err("openings should be played on an empty board with no previous plies".to_string());
        }
        let i = opening_index % (N * N * (N * N - 1));
        self.play(self.possible_turns().into_iter().nth(i / (N * N - 1)).unwrap())?;
        self.play(self.possible_turns().into_iter().nth(i % (N * N - 1)).unwrap())
    }

    /// Play the nth possible turn. Useful for random openings.
    pub fn nth_move(&mut self, mut n: usize) -> StrResult<()> {
        let turns = self.possible_turns();
        n %= turns.len();
        self.play(turns.into_iter().nth(n).unwrap())
    }

    /// Like nth_move except limited to only placing flats.
    pub fn nth_place_flat(&mut self, mut n: usize) -> StrResult<()> {
        let turns: Vec<_> = self
            .possible_turns()
            .into_iter()
            .filter(|t| matches!(t, Turn::Place { shape: Shape::Flat, .. }))
            .collect();
        n %= turns.len();
        self.play(turns.into_iter().nth(n).unwrap())
    }

    pub fn get_counts(&self) -> (Stones, Capstones) {
        match self.to_move {
            Colour::White => (self.white_stones, self.white_caps),
            Colour::Black => (self.black_stones, self.black_caps),
        }
    }

    fn dec_stones(&mut self) {
        match self.to_move {
            Colour::White => self.white_stones -= 1,
            Colour::Black => self.black_stones -= 1,
        }
    }

    fn dec_caps(&mut self) {
        match self.to_move {
            Colour::White => self.white_caps -= 1,
            Colour::Black => self.black_caps -= 1,
        }
    }

    fn execute_place(&mut self, pos: Pos<N>, shape: Shape) -> StrResult<()> {
        let (stones, caps) = self.get_counts();
        if self.board[pos].is_some() {
            Err(format!(
                "cannot place a piece in that position because it is already occupied, pos={pos:?}, {}",
                self.board
            ))
        } else if matches!(shape, Shape::Capstone) && (caps == 0) {
            Err(format!(
                "there is no capstone to play, white=({}, {}), black=({}, {})",
                self.white_stones, self.white_caps, self.black_stones, self.black_caps
            ))
        } else if matches!(shape, Shape::Flat | Shape::Wall) && stones == 0 {
            Err(format!(
                "cannot play a stone without stones, white=({}, {}), black=({}, {})",
                self.white_stones, self.white_caps, self.black_stones, self.black_caps
            ))
        } else if self.ply < 2 && matches!(shape, Shape::Wall | Shape::Capstone) {
            Err(format!(
                "cannot play a wall or capstone on the first two plies, ply={}",
                self.ply
            ))
        } else {
            self.board[pos] = Some(Tile::new(Piece {
                colour: self.colour(),
                shape,
            }));
            if matches!(shape, Shape::Flat | Shape::Wall) {
                self.dec_stones();
            } else {
                self.dec_caps();
            }
            Ok(())
        }
    }

    fn execute_move(&mut self, pos: Pos<N>, direction: Direction, moves: ArrayVec<bool, N>) -> StrResult<()> {
        // take the pieces
        let on_square = self.board[pos].take().ok_or("cannot move from an empty square")?;
        if on_square.top.colour != self.to_move {
            return Err(format!(
                "cannot move a stack that you do not own, pos={pos:?}, {}",
                self.board
            ));
        }
        let (left, carry) = on_square.take::<N>(moves.len())?;
        self.board[pos] = left;

        let mut next = pos.step(direction);
        for (carry, &should_step) in carry.into_iter().rev().zip(&moves) {
            // only unwrap the position when it is needed
            let p = next.ok_or(format!(
                "cannot move out of board, pos={pos:?}, direction={direction:?}, moves={moves:?}"
            ))?;

            // stack the dropped piece on top
            if let Some(t) = self.board[p].take() {
                self.board[p] = Some(t.stack(carry)?);
            } else {
                self.board[p] = Some(Tile::new(carry));
            }
            if should_step {
                next = p.step(direction);
            }
        }

        Ok(())
    }

    pub fn play(&mut self, my_move: Turn<N>) -> StrResult<()> {
        match my_move {
            Turn::Place { pos, shape } => self.execute_place(pos, shape),
            Turn::Move {
                pos,
                direction,
                moves,
            } => self.execute_move(pos, direction, moves),
        }?;
        self.ply += 1;
        self.to_move = self.to_move.next();
        Ok(())
    }

    pub fn winner(&self) -> GameResult {
        if self.board.find_paths(self.to_move.next()) {
            GameResult::Winner {
                colour: self.to_move.next(),
                road: true,
            }
        } else if self.board.find_paths(self.to_move) {
            GameResult::Winner {
                colour: self.to_move,
                road: true,
            }
        } else if self.white_caps == 0 && self.white_stones == 0
            || self.black_caps == 0 && self.black_stones == 0
            || self.board.full()
        {
            let flat_diff = self.board.flat_diff();
            match flat_diff.cmp(&self.komi) {
                Ordering::Greater => GameResult::Winner {
                    colour: Colour::White,
                    road: false,
                },
                Ordering::Less => GameResult::Winner {
                    colour: Colour::Black,
                    road: false,
                },
                Ordering::Equal => GameResult::Draw { turn_limit: false },
            }
        } else if self.ply >= TURN_LIMIT {
            GameResult::Draw { turn_limit: true }
        } else {
            GameResult::Ongoing
        }
    }
}
