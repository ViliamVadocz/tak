use std::ops::{Index, IndexMut};

use takparse::{Color, Piece, Square};

use crate::tile::Tile;

#[derive(Clone, Debug)]
pub struct Board<const N: usize> {
    data: [[Tile; N]; N],
}

impl<const N: usize> Default for Board<N> {
    fn default() -> Self {
        let data = vec![vec![Tile::default(); N].try_into().unwrap(); N]
            .try_into()
            .unwrap();
        Board { data }
    }
}

impl<const N: usize> Index<Square> for Board<N> {
    type Output = Tile;

    fn index(&self, index: Square) -> &Self::Output {
        let x = index.column() as usize;
        let y = index.row() as usize;
        self.data.index(y).index(x)
    }
}

impl<const N: usize> IndexMut<Square> for Board<N> {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        let x = index.column() as usize;
        let y = index.row() as usize;
        self.data.index_mut(y).index_mut(x)
    }
}

impl<const N: usize> Board<N> {
    fn has(square: Square) -> bool {
        let n = N as u8;
        square.column() < n && square.row() < n
    }

    pub fn get(&self, index: Square) -> Option<&Tile> {
        if Board::<N>::has(index) {
            Some(self.index(index))
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: Square) -> Option<&mut Tile> {
        if Board::<N>::has(index) {
            Some(self.index_mut(index))
        } else {
            None
        }
    }

    pub fn full(&self) -> bool {
        !self.data.iter().any(|row| row.iter().any(Tile::is_empty))
    }

    pub fn flat_diff(&self) -> i8 {
        self.data
            .iter()
            .flat_map(|row| row.iter())
            .map(|tile| match tile.top() {
                Some((Piece::Flat, Color::White)) => 1,
                Some((Piece::Flat, Color::Black)) => -1,
                _ => 0,
            })
            .sum()
    }

    pub fn find_paths(&self, color: Color) -> bool {
        // check vertical paths.
        let mut seen = [[false; N]; N];
        for x in 0..N {
            self.find_paths_recursive(x, 0, color, &mut seen);
        }
        if seen[N - 1].iter().any(|&x| x) {
            return true;
        }
        // check horizontal paths.
        let mut seen = [[false; N]; N];
        for y in 0..N {
            self.find_paths_recursive(0, y, color, &mut seen);
        }
        seen.iter().any(|row| row[N - 1])
    }

    fn find_paths_recursive(&self, x: usize, y: usize, color: Color, seen: &mut [[bool; N]; N]) {
        if y >= N || x >= N || seen[y][x] {
            return;
        }

        if let Some((top_piece, top_color)) = self.data[y][x].top() {
            if top_color == color && matches!(top_piece, Piece::Flat | Piece::Cap) {
                seen[y][x] = true;
                // Recursive board fill.
                self.find_paths_recursive(x + 1, y, color, seen);
                self.find_paths_recursive(x, y + 1, color, seen);
                if let Some(x) = x.checked_sub(1) {
                    self.find_paths_recursive(x, y, color, seen);
                }
                if let Some(y) = y.checked_sub(1) {
                    self.find_paths_recursive(x, y, color, seen)
                }
            }
        }
    }
}
