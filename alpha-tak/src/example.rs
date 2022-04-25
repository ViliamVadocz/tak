use tak::*;
use tch::Tensor;

use crate::{
    repr::{game_repr, possible_moves},
    search::move_index,
};

#[derive(Debug)]
pub struct IncompleteExample<const N: usize> {
    pub game: Game<N>,
    pub policy: Vec<(Move, u32)>,
}

impl<const N: usize> IncompleteExample<N> {
    #[must_use]
    pub fn complete(self, result: f32) -> Example<N> {
        Example {
            game: self.game,
            policy: self.policy,
            result,
        }
    }
}

#[derive(Debug)]
pub struct Example<const N: usize> {
    pub game: Game<N>,
    pub policy: Vec<(Move, u32)>,
    pub result: f32,
}

impl<const N: usize> Example<N> {
    pub fn to_tensors(&self) -> Vec<(Tensor, Tensor, f32)> {
        let mut pi = [
            vec![0.; possible_moves(N)],
            vec![0.; possible_moves(N)],
            vec![0.; possible_moves(N)],
            vec![0.; possible_moves(N)],
            vec![0.; possible_moves(N)],
            vec![0.; possible_moves(N)],
            vec![0.; possible_moves(N)],
            vec![0.; possible_moves(N)],
        ];
        let total = self.policy.iter().map(|(_, c)| c).sum::<u32>() as f32;
        for (m, value) in &self.policy {
            for (i, symm) in Symmetry::<N>::symmetries(*m).into_iter().enumerate() {
                pi[i][move_index(&symm, N)] = *value as f32 / total;
            }
        }

        self.game
            .clone()
            .symmetries()
            .into_iter()
            .enumerate()
            .map(|(i, game)| (game_repr(&game), Tensor::of_slice(&pi[i]), self.result))
            .collect()
    }
}
