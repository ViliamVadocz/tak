use std::collections::HashMap;

use tak::*;

use super::{node::Node, turn_map::Lut};
use crate::agent::Agent;

impl<const N: usize> Node<N>
where
    Turn<N>: Lut,
{
    fn update_stats(&mut self, reward: f32) {
        let scaled_reward = self.expected_reward * self.visited_count as f32;
        self.visited_count += 1;
        self.expected_reward = (scaled_reward + reward) / self.visited_count as f32;
    }

    pub fn virtual_rollout<A: Agent<N>>(&mut self, game: Game<N>, path: &mut Vec<Turn<N>>) -> GameResult {
        let rollout_result = match self.result {
            GameResult::Ongoing => todo!(), // self.virtual_rollout_next(game),
            r => r,
        };

        match rollout_result {
            GameResult::Winner { colour, .. } => {
                self.update_stats(if colour == game.to_move { -1.0 } else { 1.0 })
            }
            GameResult::Draw { .. } => self.update_stats(0.0),
            _ => {}
        };

        rollout_result
    }

    fn expand_node<A: Agent<N>>(&mut self, game: Game<N>) {
        self.children = Some(
            game.possible_turns()
                .into_iter()
                .map(|turn| (turn, Node::new()))
                .collect(),
        );
    }

    fn virtual_rollout_next<A: Agent<N>>(&mut self, mut game: Game<N>) -> GameResult {
        // pick which node to rollout
        let mut children = self.children.take().unwrap();
        let (turn, next_node) = children
            .iter_mut()
            .max_by(|(_, a), (_, b)| {
                self.upper_confidence_bound(a)
                    .partial_cmp(&self.upper_confidence_bound(b))
                    .expect("tried comparing nan")
            })
            .unwrap();

        // rollout next node
        game.play(turn.clone()).unwrap();
        let eval = next_node.rollout(game, agent);
        self.children = Some(children);

        // take the mean of the expected reward and eval
        self.expected_reward =
            ((self.visited_count - 1) as f32 * self.expected_reward + eval) / (self.visited_count as f32);

        -eval
    }
}
