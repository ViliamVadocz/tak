use std::collections::HashMap;

use tak::*;

use super::{
    node::{InnerNode, Node},
    turn_map::Lut,
};
use crate::agent::Agent;

impl<const N: usize> Node<N>
where
    Turn<N>: Lut,
{
    pub fn rollout(&mut self, game: Game<N>, path: &mut Vec<Turn<N>>) -> GameResult {
        let node;
        let result = if let Some(n) = self.0 {
            // we've been here before - recurse if we can
            node = n;
            match node.result {
                GameResult::Ongoing => node.select(game, path),
                r => r,
            }
        } else {
            // uninitialized node - stop the recursion
            node = self.initialize(game);
            node.result
        };

        match result {
            // our virtual visit ended on a terminal node - propagate a concrete score
            GameResult::Winner { colour, .. } => {
                node.apply_eval(if colour == game.to_move { -1.0 } else { 1.0 })
            }
            GameResult::Draw { .. } => node.apply_eval(0.0),

            // we've cut the recursion short of a terminal node - count a virtual visit
            GameResult::Ongoing => node.virtual_visits += 1,
        }

        result
    }
}

impl<const N: usize> InnerNode<N>
where
    Turn<N>: Lut,
{
    fn apply_eval(&mut self, reward: f32) {
        let scaled_reward = self.expected_reward * self.visits as f32;
        self.visits += 1;
        self.expected_reward = (scaled_reward + reward) / self.visits as f32;
    }

    fn select(&mut self, game: Game<N>, path: &mut Vec<Turn<N>>) -> GameResult {
        if self.children.is_empty() {
            // lazily initialize the children
            self.children = game
                .possible_turns()
                .into_iter()
                .map(|turn| (turn, Node::default()))
                .collect();
        }

        // select the node to recurse into
        let ((turn, node), _) = self
            .children
            .iter_mut()
            .map(|pair| {
                (
                    pair,
                    pair.1
                         .0
                        .map_or(f32::INFINITY, |child| self.upper_confidence_bound(&child)),
                )
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("tried comparing nan"))
            .expect("tried to select on a node without children");

        // update the game state
        game.play(turn.clone());
        // add the move to our path
        path.push(turn.clone());
        // continue the rollout
        node.rollout(game, path)
    }
}
