// use std::ops::{Index, IndexMut};

use takparse::{Color, Piece, Square, Stack as TpsStack};

use crate::stack::Stack;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Board<const N: usize> {
    data: [[Stack; N]; N],
}

impl<const N: usize> Default for Board<N> {
    fn default() -> Self {
        Self {
            data: [[Stack::default(); N]; N],
        }
    }
}

impl<const N: usize> Board<N> {
    pub fn iter(&self) -> impl Iterator<Item = impl Iterator<Item = &Stack>> {
        self.data.iter().map(|row| row.iter())
    }

    pub fn get(&self, square: Square) -> Option<&Stack> {
        self.data
            .get(square.row() as usize)
            .and_then(|r| r.get(square.column() as usize))
    }

    pub fn get_mut(&mut self, square: Square) -> Option<&mut Stack> {
        self.data
            .get_mut(square.row() as usize)
            .and_then(|r| r.get_mut(square.column() as usize))
    }

    pub fn full(&self) -> bool {
        !self.data.iter().any(|row| row.iter().any(Stack::is_empty))
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

    pub fn has_road(&self, color: Color) -> bool {
        let road = self.data.map(|row| row.map(|s| s.road(color)));
        let mut seen = [[false; N]; N];
        let mut reached = [[false; N]; N];
        let mut stack = Vec::with_capacity(N * N);

        // Horizontal roads
        for row in 0..N {
            stack.push((row, 0));
        }
        Self::flood_fill(&road, &mut stack, &mut seen, &mut reached);
        if reached.iter().any(|row| row[N - 1]) {
            return true;
        }

        // Vertical roads
        stack.clear();
        reached = [[false; N]; N];
        seen = [[false; N]; N];
        for col in 0..N {
            stack.push((0, col));
        }
        Self::flood_fill(&road, &mut stack, &mut seen, &mut reached);
        reached[N - 1].iter().any(|b| *b)
    }

    fn flood_fill(
        road: &[[bool; N]; N],
        stack: &mut Vec<(usize, usize)>,
        seen: &mut [[bool; N]; N],
        reached: &mut [[bool; N]; N],
    ) {
        while let Some((row, col)) = stack.pop() {
            seen[row][col] = true;
            if road[row][col] {
                reached[row][col] = true;

                if let Some(r) = row.checked_sub(1) {
                    if !seen[r][col] {
                        stack.push((r, col));
                    }
                }
                if let Some(c) = col.checked_sub(1) {
                    if !seen[row][c] {
                        stack.push((row, c));
                    }
                }
                if row < N - 1 {
                    let r = row + 1;
                    if !seen[r][col] {
                        stack.push((r, col));
                    }
                }
                if col < N - 1 {
                    let c = col + 1;
                    if !seen[row][c] {
                        stack.push((row, c));
                    }
                }
            }
        }
    }
}

impl<'a, const N: usize> FromIterator<Option<&'a TpsStack>> for Board<N> {
    fn from_iter<T: IntoIterator<Item = Option<&'a TpsStack>>>(iter: T) -> Self {
        let mut iter = iter.into_iter().map(|square| {
            square.map_or_else(Stack::default, |stack| {
                Stack::exact(stack.top(), stack.colors().collect())
            })
        });
        let mut data = [(); N].map(|()| [(); N].map(|()| iter.next().unwrap_or_default()));
        data.reverse();
        Self { data }
    }
}
