use crate::{
    board::{Board, Tile, Shape},
    colour::Colour,
    turn::Turn,
    StrResult,
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

impl<const N: usize> Game<N> {
    fn get_counts(&self) -> (Stones, Capstones) {
        match self.to_move {
            Colour::White => (self.white_stones, self.white_caps),
            Colour::Black => (self.black_stones, self.black_caps),
        }
    }

    pub fn play(&mut self, my_move: Turn<N>) -> StrResult<()> {
        match my_move {
            Turn::Place { pos, piece } => {
                let (stones, caps) = self.get_counts();
                if self.board[pos].is_some() {
                    Err("cannot place a piece in that position because it is already occupied")
                } else if matches!(piece.shape, Shape::Capstone) && caps == 0 {
                    Err("there is no capstone to play")
                } else if !matches!(piece.shape, Shape::Capstone) && stones == 0 {
                    Err("cannot play a stone without stones")
                } else {
                    self.board[pos] = Some(Tile {
                        top: piece,
                        stack: None,
                    });
                    Ok(())
                }
            }
            Turn::Move { mut pos, drops } => {
                // take the pieces
                let on_square = self.board[pos].take().ok_or("cannot move from an empty square")?;
                let (left, carry) = on_square.take::<N>(drops.len())?;
                self.board[pos] = left;

                // try to move them
                let mut direction = None;
                for (carried, (next, dropped)) in carry.into_iter().zip(drops) {
                    // make sure move direction is correct
                    let diff = next - pos;
                    if let Some(dir) = direction {
                        if !(next == pos || diff == dir) {
                            return Err("cannot switch directions during a move");
                        }
                    } else {
                        if diff.x * diff.x + diff.y * diff.y != 1 {
                            return Err("impossible move direction");
                        }
                        direction = Some(diff);
                    }
                    pos = next;
                    // check that the dropped piece is the same as the one that was picked up
                    if carried != dropped {
                        return Err("tried dropping a different piece than what was picked up");
                    }
                    // stack the dropped piece on top
                    self.board[next] = self.board[pos].take().map(|t| t.stack(carried)).transpose()?;
                }
                Ok(())
            }
        }
    }

    // TODO check win conditions
    // TODO movegen
}
