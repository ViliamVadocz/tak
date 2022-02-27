use tak::*;

use crate::{
    agent::Agent,
    example::{Example, IncompleteExample},
    search::{node::Node, turn_map::Lut},
};

struct Player<'a, const N: usize, A: Agent<N>> {
    node: Node<N>,
    agent: &'a A,
    examples: Vec<IncompleteExample<N>>,
    turns: Vec<Turn<N>>,
}

impl<'a, const N: usize, A: Agent<N>> Player<'a, N, A>
where
    Turn<N>: Lut,
{
    pub fn rollout(&mut self, game: &Game<N>, amount: usize) {
        for _ in 0..amount {
            self.node.rollout(game.clone(), self.agent);
        }

        self.examples.push(IncompleteExample {
            game: game.clone(),
            policy: self.node.improved_policy(),
        })
    }

    pub fn play_turn(&mut self, exploitation: bool) -> Turn<N> {
        let turn = self.node.pick_move(exploitation);
        let node = std::mem::take(&mut self.node);
        self.node = node.play(&turn);
        turn
    }

    pub fn get_examples(self, result: f32, opening_plies: usize) -> Vec<Example<N>> {
        self.examples
            .into_iter()
            .enumerate()
            .map(|(ply, ex)| {
                let perspective = if (ply + opening_plies) % 2 == 0 {
                    result
                } else {
                    -result
                };
                ex.complete(perspective)
            })
            .collect()
    }
}
