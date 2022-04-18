use regex::Regex;

use crate::{
    board::Board,
    colour::Colour,
    game::Game,
    pos::Pos,
    ptn::{FromPTN, ToPTN},
    tile::{Piece, Shape, Tile},
    StrResult,
};

lazy_static! {
    static ref EMPTY_TILE_RE: Regex = Regex::new("x([0-9]?)").unwrap();
    static ref STACK_TILE_RE: Regex = Regex::new("([12]*)([12])([CS]?)").unwrap();
}

pub trait FromTPS: Sized {
    fn from_tps(s: &str) -> StrResult<Self>;
}

pub trait ToTPS {
    fn to_tps(&self) -> String;
}

impl<const N: usize> ToTPS for Game<N> {
    /// Technically this is modified TPS with extra info
    fn to_tps(&self) -> String {
        // TPS to_move move_num (white_reserves) (black_reserves)
        format!(
            "{} {} {} ({}/{}) ({}/{}) {}",
            self.board.to_tps(),
            self.to_move.to_ptn(),
            (self.ply / 2) + 1,
            self.white_stones,
            self.white_caps,
            self.black_stones,
            self.black_caps,
            self.komi
        )
    }
}

impl<const N: usize> ToTPS for Board<N> {
    /// Get board TPS
    fn to_tps(&self) -> String {
        let mut out = String::new();

        // combine empty squares
        let add_empty = |out: &mut String, empty: usize| {
            if empty > 0 {
                out.push('x');
                if empty > 1 {
                    out.push_str(&empty.to_string());
                }
                out.push(',');
            }
            0
        };

        // for each row
        for y in (0..N).rev() {
            let mut empty = 0;
            // for each tile
            for x in 0..N {
                let pos = Pos { x, y };
                if let Some(tile) = &self[pos] {
                    empty = add_empty(&mut out, empty);
                    for colour in &tile.stack {
                        out.push_str(&colour.to_ptn());
                    }
                    out.push_str(&tile.top.colour.to_ptn());
                    out.push_str(&tile.top.shape.to_ptn());
                    out.push(',');
                } else {
                    empty += 1;
                }
            }
            add_empty(&mut out, empty);
            out.pop().unwrap(); // remove last comma
            out.push('/');
        }
        out.pop().unwrap(); // remove last slash
        out
    }
}

impl<const N: usize> FromTPS for Board<N>
where
    [[Option<Tile>; N]; N]: Default,
{
    fn from_tps(s: &str) -> StrResult<Self> {
        let mut board = Board::default();
        let row_count = s.split('/').count();
        if row_count != N {
            return Err(format!("expected {N} rows, got {row_count}"));
        }
        for (i, row) in s.split('/').enumerate() {
            let y = N - i - 1;
            let mut x = 0;
            for tile in row.split(',') {
                if let Some(cap) = EMPTY_TILE_RE.captures(tile) {
                    x += cap[1].parse::<usize>().unwrap_or(1);
                } else {
                    let pos = Pos { x, y };
                    let cap = STACK_TILE_RE
                        .captures(tile)
                        .ok_or_else(|| format!("didn't recognize stack {tile}"))?;
                    let stack = cap[1]
                        .chars()
                        .map(|c| Colour::from_ptn(&c.to_string()))
                        .collect::<StrResult<Vec<_>>>()?;
                    let piece = Piece {
                        shape: Shape::from_ptn(&cap[3])?,
                        colour: Colour::from_ptn(&cap[2])?,
                    };
                    board[pos] = Some(Tile { top: piece, stack });
                    x += 1;
                }
            }
            if x != N {
                return Err(format!("only got {x} tiles in row number {y}, expected {N}"));
            }
        }
        Ok(board)
    }
}
