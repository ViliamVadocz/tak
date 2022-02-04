use std::collections::HashMap;

use tak::{game::Game, symm::Symmetry, tile::Tile, turn::Turn};
use tch::Tensor;

use crate::{
    repr::{game_repr, moves_dims},
    turn_map::Lut,
};

#[derive(Debug)]
pub struct IncompleteExample<const N: usize> {
    pub game: Game<N>,
    pub policy: HashMap<Turn<N>, f32>,
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
    pub policy: HashMap<Turn<N>, f32>,
    pub result: f32,
}

impl<const N: usize> Example<N>
where
    Turn<N>: Lut,
    [[Option<Tile>; N]; N]: Default,
{
    pub fn to_tensors(&self) -> Vec<(Tensor, Tensor, Tensor)> {
        let mut pi = [
            vec![0.; moves_dims(N)],
            vec![0.; moves_dims(N)],
            vec![0.; moves_dims(N)],
            vec![0.; moves_dims(N)],
            vec![0.; moves_dims(N)],
            vec![0.; moves_dims(N)],
            vec![0.; moves_dims(N)],
            vec![0.; moves_dims(N)],
        ];
        for (turn, &value) in self.policy.iter() {
            for (i, symm) in turn.clone().symmetries().into_iter().enumerate() {
                pi[i][symm.turn_map()] = value;
            }
        }

        self.game
            .clone()
            .symmetries()
            .into_iter()
            .enumerate()
            .map(|(i, game)| {
                (
                    game_repr(&game),
                    Tensor::of_slice(&pi[i]),
                    Tensor::of_slice(&[self.result]),
                )
            })
            .collect()
    }
}
