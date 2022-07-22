use takparse::{Direction, Move, MoveKind, Square};

use crate::{board::Board, game::Game};

pub trait Symmetry<const N: usize>: Sized {
    fn symmetries(self) -> [Self; 8];
}

impl<const N: usize> Symmetry<N> for Square {
    fn symmetries(self) -> [Self; 8] {
        let n = N as u8;
        [
            self,
            self.rotate(n),
            self.rotate(n).rotate(n),
            self.rotate(n).rotate(n).rotate(n),
            self.mirror(n),
            self.mirror(n).rotate(n),
            self.mirror(n).rotate(n).rotate(n),
            self.mirror(n).rotate(n).rotate(n).rotate(n),
        ]
    }
}

impl<const N: usize> Symmetry<N> for Direction {
    fn symmetries(self) -> [Self; 8] {
        [
            self,
            self.rotate(),
            self.rotate().rotate(),
            self.rotate().rotate().rotate(),
            self.mirror(),
            self.mirror().rotate(),
            self.mirror().rotate().rotate(),
            self.mirror().rotate().rotate().rotate(),
        ]
    }
}

impl<const N: usize> Symmetry<N> for Move {
    fn symmetries(self) -> [Self; 8] {
        let square = self.square();
        let kind = self.kind();
        match kind {
            MoveKind::Place(_) => Symmetry::<N>::symmetries(square).map(|square| Move::new(square, kind)),
            MoveKind::Spread(direction, pattern) => {
                let mut directions = Symmetry::<N>::symmetries(direction).into_iter();
                Symmetry::<N>::symmetries(square)
                    .map(|square| Move::new(square, MoveKind::Spread(directions.next().unwrap(), pattern)))
            }
        }
    }
}

impl<const N: usize> Symmetry<N> for Board<N> {
    fn symmetries(self) -> [Self; 8] {
        let mut boards = [
            Board::default(),
            Board::default(),
            Board::default(),
            Board::default(),
            Board::default(),
            Board::default(),
            Board::default(),
            Board::default(),
        ];
        for x in 0..N {
            for y in 0..N {
                let square = Square::new(x as u8, y as u8);
                for (i, sym) in Symmetry::<N>::symmetries(square).into_iter().enumerate() {
                    boards[i][sym] = self[square].clone();
                }
            }
        }
        boards
    }
}

impl<const N: usize> Symmetry<N> for Game<N> {
    fn symmetries(self) -> [Self; 8] {
        let games = [
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
        ];
        let mut boards = self.board.symmetries().into_iter();
        games.map(|mut game| {
            game.board = boards.next().unwrap();
            game
        })
    }
}
