use std::collections::HashMap;

use tak::{game::Game, turn::Turn};
use tch::Tensor;

use crate::{repr::moves_dims, turn_map::LUT};

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
{
    #[must_use]
    pub fn complete(self, result: f32) -> Example<N> {
        let mut pi = vec![0.; moves_dims(N)];
        for (turn, value) in self.policy {
            pi[turn.turn_map()] = value;
        }

        Example {
            game: self.game,
            pi: Tensor::of_slice(&pi),
            v: Tensor::of_slice(&[result]),
        }
    }
}

pub struct Example<const N: usize> {
    pub game: Game<N>,
    pub pi: Tensor,
    pub v: Tensor,
}
