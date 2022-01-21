use crate::{board::Board, game::Game, pos::Pos, tile::Tile, turn::Turn};

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

impl<const N: usize> Symmetry for Turn<N> {
    fn symmetries(self) -> [Self; 8] {
        match self {
            Turn::Place { pos, piece } => pos.symmetries().map(|pos| Turn::Place { pos, piece }),
            Turn::Move { pos, drops } => {
                pos.symmetries()
                    .into_iter()
                    .enumerate()
                    .map(|(i, pos)| Turn::Move {
                        pos,
                        drops: drops
                            .clone()
                            .into_iter()
                            .map(|(pos, piece)| (pos.symmetries()[i], piece))
                            .collect(),
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap() // UGLY
            }
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

#[cfg(test)]
mod tests {
    use crate::pos::Pos;

    #[test]
    fn rotate_even() {
        // corner
        let pos: Pos<6> = Pos { x: 0, y: 0 };
        assert_eq!(pos.rotate(), Pos { x: 0, y: 5 });
        assert_eq!(pos.rotate().rotate(), Pos { x: 5, y: 5 });
        assert_eq!(pos.rotate().rotate().rotate(), Pos { x: 5, y: 0 });
        assert_eq!(pos.rotate().rotate().rotate().rotate(), Pos { x: 0, y: 0 });
        // centre
        let pos: Pos<6> = Pos { x: 2, y: 2 };
        assert_eq!(pos.rotate(), Pos { x: 2, y: 3 });
        assert_eq!(pos.rotate().rotate(), Pos { x: 3, y: 3 });
        assert_eq!(pos.rotate().rotate().rotate(), Pos { x: 3, y: 2 });
        assert_eq!(pos.rotate().rotate().rotate().rotate(), Pos { x: 2, y: 2 });
    }

    #[test]
    fn rotate_odd() {
        // corner
        let pos: Pos<7> = Pos { x: 0, y: 0 };
        assert_eq!(pos.rotate(), Pos { x: 0, y: 6 });
        assert_eq!(pos.rotate().rotate(), Pos { x: 6, y: 6 });
        assert_eq!(pos.rotate().rotate().rotate(), Pos { x: 6, y: 0 });
        assert_eq!(pos.rotate().rotate().rotate().rotate(), Pos { x: 0, y: 0 });
        // centre
        let pos: Pos<7> = Pos { x: 3, y: 3 };
        assert_eq!(pos.rotate(), Pos { x: 3, y: 3 });
        assert_eq!(pos.rotate().rotate(), Pos { x: 3, y: 3 });
        assert_eq!(pos.rotate().rotate().rotate(), Pos { x: 3, y: 3 });
        assert_eq!(pos.rotate().rotate().rotate().rotate(), Pos { x: 3, y: 3 });
    }

    #[test]
    fn mirror_even() {
        let pos: Pos<6> = Pos { x: 1, y: 2 };
        assert_eq!(pos.mirror(), Pos { x: 1, y: 3 });
        assert_eq!(pos.mirror().mirror(), Pos { x: 1, y: 2 });
    }

    #[test]
    fn mirror_odd() {
        let pos: Pos<7> = Pos { x: 4, y: 1 };
        assert_eq!(pos.mirror(), Pos { x: 4, y: 5 });
        assert_eq!(pos.mirror().mirror(), Pos { x: 4, y: 1 });

        // centre line
        let pos: Pos<7> = Pos { x: 2, y: 3 };
        assert_eq!(pos.mirror(), Pos { x: 2, y: 3 });
    }
}
