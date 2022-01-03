use std::{
    ops::{Index, IndexMut},
};

use crate::{tile::Tile, turn::Pos};

#[derive(Clone, Debug)]
pub struct Board<const N: usize> {
    data: [[Option<Tile>; N]; N],
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
