use std::collections::HashMap;

use tak::{game::Game, symm::Symmetry, tile::Tile, turn::Turn};
use tch::Tensor;

use crate::{repr::moves_dims, turn_map::LUT};

#[derive(Debug)]
pub struct IncompleteExample<const N: usize>
where
    Turn<N>: LUT,
{
    pub game: Game<N>,
    pub policy: HashMap<Turn<N>, f32>,
}

impl<const N: usize> IncompleteExample<N>
where
    Turn<N>: LUT,
    [[Option<Tile>; N]; N]: Default,
{
    #[must_use]
    pub fn complete(self, result: f32) -> Vec<Example<N>> {
        let mut pi = [vec![0.; moves_dims(N)]; 8];
        for (turn, value) in self.policy {
            for (i, symm) in turn.symmetries().into_iter().enumerate() {
                pi[i][symm.turn_map()] = value;
            }
        }

        self.game
            .symmetries()
            .into_iter()
            .enumerate()
            .map(|(i, game)| Example {
                game,
                pi: Tensor::of_slice(&pi[i]),
                v: Tensor::of_slice(&[result]),
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Example<const N: usize> {
    pub game: Game<N>,
    pub pi: Tensor,
    pub v: Tensor,
}
