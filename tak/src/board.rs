use std::{
    collections::HashSet,
    fmt::Display,
    ops::{Index, IndexMut},
};

use crate::{
    colour::Colour,
    pos::Pos,
    tile::{Piece, Shape, Tile},
};

#[derive(Clone, Debug)]
pub struct Board<const N: usize> {
    data: [[Option<Tile>; N]; N],
}

impl<const N: usize> Board<N> {
    pub fn full(&self) -> bool {
        self.data.iter().all(|row| row.iter().all(|x| x.is_some()))
    }

    pub fn flat_diff(&self) -> i32 {
        let mut diff = 0;
        for row in &self.data {
            row.iter().flatten().for_each(|tile| {
                if matches!(tile.top.shape, Shape::Flat) {
                    match tile.top.colour {
                        Colour::White => diff += 1,
                        Colour::Black => diff -= 1,
                    }
                }
            });
        }
        diff
    }

    pub fn find_paths(&self, colour: Colour) -> bool {
        // check vertical paths
        let mut seen = HashSet::new();
        for x in 0..N {
            let pos = Pos { x, y: 0 };
            self.find_paths_recursive(pos, colour, &mut seen);
        }
        if (0..N).any(|x| seen.contains(&Pos { x, y: N - 1 })) {
            return true;
        }
        // check horizontal paths
        let mut seen = HashSet::new();
        for y in 1..N {
            let pos = Pos { x: 0, y };
            self.find_paths_recursive(pos, colour, &mut seen);
        }
        (0..N).any(|y| seen.contains(&Pos { x: N - 1, y }))
    }

    fn find_paths_recursive(&self, pos: Pos, colour: Colour, seen: &mut HashSet<Pos>) {
        if seen.contains(&pos) {
            return;
        }
        if let Some(Tile {
            top: Piece {
                colour: piece_colour,
                shape,
            },
            stack: _,
        }) = self[pos]
        {
            if piece_colour == colour && matches!(shape, Shape::Flat | Shape::Capstone) {
                seen.insert(pos);
                for neighbor in pos.neighbors::<N>() {
                    self.find_paths_recursive(neighbor, colour, seen)
                }
            }
        }
    }
}

impl<const N: usize> Default for Board<N>
where
    [[Option<Tile>; N]; N]: Default,
{
    fn default() -> Self {
        Self {
            data: <[[Option<Tile>; N]; N]>::default(),
        }
    }
}

impl<const N: usize> Index<Pos> for Board<N> {
    type Output = Option<Tile>;

    fn index(&self, index: Pos) -> &Self::Output {
        self.data.index(index.y).index(index.x)
    }
}

impl<const N: usize> IndexMut<Pos> for Board<N> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        self.data.index_mut(index.y).index_mut(index.x)
    }
}

impl<const N: usize> Display for Board<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        for y in 0..N {
            for x in 0..N {
                let pos = Pos { x, y };
                if let Some(tile) = &self[pos] {
                    output.push(match tile.top.shape {
                        Shape::Flat => '_',
                        Shape::Wall => '/',
                        Shape::Capstone => 'o',
                    });
                    output.push(match tile.top.colour {
                        Colour::White => 'w',
                        Colour::Black => 'b',
                    });
                    output.push_str(&tile.size().to_string());
                } else {
                    output.push('.');
                    output.push('.');
                    output.push('.');
                }
                output.push(' ');
            }
            output.push('\n');
        }
        write!(f, "{}", output)
    }
}
