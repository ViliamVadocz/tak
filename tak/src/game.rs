use std::cmp::Ordering;

use arrayvec::ArrayVec;

use crate::{
    board::Board,
    colour::Colour,
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
    Winner(Colour),
    Draw,
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
            return Err("openings should be played on an empty board with no previous plies");
        }
        let i = opening_index % (N * N * (N * N - 1));
        self.play(self.move_gen().into_iter().nth(i / (N * N - 1)).unwrap())?;
        self.play(self.move_gen().into_iter().nth(i % (N * N - 1)).unwrap())
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

    fn execute_place(&mut self, pos: Pos<N>, piece: Piece) -> StrResult<()> {
        let (stones, caps) = self.get_counts();
        if self.board[pos].is_some() {
            Err("cannot place a piece in that position because it is already occupied")
        } else if matches!(piece.shape, Shape::Capstone) && (caps == 0) {
            Err("there is no capstone to play")
        } else if matches!(piece.shape, Shape::Flat | Shape::Wall) && stones == 0 {
            Err("cannot play a stone without stones")
        } else if self.ply < 2 && matches!(piece.shape, Shape::Wall | Shape::Capstone) {
            Err("cannot play a wall or capstone on the first two plies")
        } else if piece.colour != self.colour() {
            Err("cannot play the other players colour outside the first two plies")
        } else {
            self.board[pos] = Some(Tile::new(piece));
            if matches!(piece.shape, Shape::Flat | Shape::Wall) {
                self.dec_stones();
            } else {
                self.dec_caps();
            }
            Ok(())
        }
    }

    fn execute_move(&mut self, mut pos: Pos<N>, drops: ArrayVec<(Pos<N>, Piece), N>) -> StrResult<()> {
        if drops.is_empty() {
            return Err("moves cannot be empty");
        }
        // take the pieces
        let on_square = self.board[pos].take().ok_or("cannot move from an empty square")?;
        if on_square.top.colour != self.to_move {
            return Err("cannot move a stack that you do not own");
        }
        let (left, carry) = on_square.take::<N>(drops.len())?;
        self.board[pos] = left;

        // try to move them
        let mut direction = None;
        for (carried, (next, dropped)) in carry.into_iter().rev().zip(drops) {
            // make sure move direction is correct
            if let Some(dir) = direction {
                if !(next == pos || (next - pos) == dir) {
                    return Err("cannot switch directions during a move");
                }
            } else {
                direction = Some(next - pos);
            }
            pos = next;
            // check that the dropped piece is the same as the one that was picked up
            if carried != dropped {
                return Err("tried dropping a different piece than what was picked up");
            }
            // stack the dropped piece on top
            if let Some(t) = self.board[pos].take() {
                self.board[pos] = Some(t.stack(carried)?);
            } else {
                self.board[pos] = Some(Tile::new(carried));
            }
        }
        Ok(())
    }

    pub fn play(&mut self, my_move: Turn<N>) -> StrResult<()> {
        match my_move {
            Turn::Place { pos, piece } => self.execute_place(pos, piece),
            Turn::Move { pos, drops } => self.execute_move(pos, drops),
        }?;
        self.ply += 1;
        self.to_move = self.to_move.next();
        Ok(())
    }

    pub fn winner(&self) -> GameResult {
        if self.white_caps == 0 && self.white_stones == 0
            || self.black_caps == 0 && self.black_stones == 0
            || self.board.full()
        {
            let flat_diff = self.board.flat_diff();
            match flat_diff.cmp(&self.komi) {
                Ordering::Greater => GameResult::Winner(Colour::White),
                Ordering::Less => GameResult::Winner(Colour::Black),
                Ordering::Equal => GameResult::Draw,
            }
        } else if self.board.find_paths(self.to_move.next()) {
            GameResult::Winner(self.to_move.next())
        } else if self.board.find_paths(self.to_move) {
            GameResult::Winner(self.to_move)
        } else if self.ply >= TURN_LIMIT {
            GameResult::Draw
        } else {
            GameResult::Ongoing
        }
    }
}
