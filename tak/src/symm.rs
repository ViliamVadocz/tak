use crate::{board::Board, direction::Direction, game::Game, pos::Pos, tile::Tile, turn::Turn};

pub trait Symmetry: Sized {
    fn symmetries(self) -> [Self; 8];
}

impl<const N: usize> Symmetry for Pos<N> {
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

impl Symmetry for Direction {
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

impl<const N: usize> Symmetry for Turn<N> {
    fn symmetries(self) -> [Self; 8] {
        match self {
            Turn::Place { pos, shape } => pos.symmetries().map(|pos| Turn::Place { pos, shape }),
            Turn::Move {
                pos,
                direction,
                moves,
            } => pos
                .symmetries()
                .zip(direction.symmetries())
                .map(|(pos, direction)| Turn::Move {
                    pos,
                    direction,
                    moves: moves.clone(),
                }),
        }
    }
}

impl<const N: usize> Symmetry for Board<N>
where
    [[Option<Tile>; N]; N]: Default,
{
    fn symmetries(self) -> [Self; 8] {
        (0..8)
            .map(|i| {
                let mut board = Board::default();
                for y in 0..N {
                    for x in 0..N {
                        let pos = Pos { x, y };
                        board[pos.symmetries()[i]] = self[pos].clone();
                    }
                }
                board
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}

impl<const N: usize> Symmetry for Game<N>
where
    [[Option<Tile>; N]; N]: Default,
{
    fn symmetries(self) -> [Self; 8] {
        [
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
        ]
        .zip(self.board.symmetries())
        .map(|(mut game, board)| {
            game.board = board;
            game
        })
    }
}
