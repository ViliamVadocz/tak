use crate::{
    board::{Board, Shape, Tile},
    colour::Colour,
    turn::Turn,
};

type Stones = u8;
type Capstones = u8;
const fn starting_stones(width: usize) -> (Stones, Capstones) {
    match width {
        3 => (10, 0),
        4 => (15, 0),
        5 => (21, 1),
        6 => (30, 1),
        8 => (50, 2),
        _ => panic!("missing starting stones for non-standard board size"),
    }
}

#[derive(Clone, Debug)]
pub struct Game<const N: usize> {
    pub board: Board<N>,
    pub to_move: Colour,
    pub white_stones: Stones,
    pub black_stones: Stones,
    pub white_caps: Capstones,
    pub black_caps: Capstones,
}

impl<const N: usize> Default for Game<N>
where
    [[Option<Tile>; N]; N]: Default,
{
    fn default() -> Self {
        let (stones, capstones) = starting_stones(N);
        Self {
            board: Board::default(),
            to_move: Colour::White,
            white_stones: stones,
            black_stones: stones,
            white_caps: capstones,
            black_caps: capstones,
        }
    }
}
