use tak::*;

use super::{node::Node, turn_map::Lut};
use crate::{
    agent::Agent,
    config::{EXPLORATION_BASE, EXPLORATION_INIT},
};

impl<const N: usize> Node<N>
where
    Turn<N>: Lut,
{
    pub fn rollout<A: Agent<N>>(&mut self, mut game: Game<N>, agent: &A) {
        let mut path = vec![];
        // perform a virtual rollout
        if matches!(self.virtual_rollout(&mut game, &mut path), GameResult::Ongoing) {
            // the game result isn't concrete - devirtualize the path
            self.devirtualize_path(&mut path.into_iter(), &agent.policy_and_eval(&game));
        }
    }

    pub fn virtual_rollout(&mut self, game: &mut Game<N>, path: &mut Vec<Turn<N>>) -> GameResult {
        let curr_colour = game.to_move;

        let result = if self.is_initialized() {
            // we've been here before - recurse if we can
            match self.result {
                GameResult::Ongoing => self.select(game, path),
                r => r,
            }
        } else {
            // uninitialized node - initialize it and stop the recursion
            self.result = game.winner();
            if self.result == GameResult::Ongoing {
                self.children = game
                    .possible_turns()
                    .into_iter()
                    .map(|turn| (turn, Node::default()))
                    .collect();
            }
            self.result
        };

        match result {
            // our rollout ended on a terminal node - propagate a concrete score
            GameResult::Winner { colour, .. } => {
                self.update_concrete(if colour == curr_colour { -1.0 } else { 1.0 })
            }
            GameResult::Draw { .. } => self.update_concrete(0.0),

            // we've cut the recursion short of a terminal node - count a virtual visit
            GameResult::Ongoing => self.virtual_visits += 1,
        }

        result
    }

    pub fn devirtualize_path<I: Iterator<Item = Turn<N>>>(
        &mut self,
        path: &mut I,
        result: &(Vec<f32>, f32),
    ) -> f32 {
        self.virtual_visits -= 1;

        let eval = -if let Some(turn) = path.next() {
            self.children
                .get_mut(&turn)
                .unwrap()
                .devirtualize_path(path, result)
        } else {
            let (policy, eval) = result;

            // replace the policies with the correct values
            self.children.iter_mut().for_each(|(turn, child)| {
                let move_index = turn.turn_map();
                child.policy = policy[move_index];
            });

            *eval
        };

        self.update_concrete(eval);

        eval
    }

    fn select(&mut self, game: &mut Game<N>, path: &mut Vec<Turn<N>>) -> GameResult {
        let visit_count = self.visit_count();
        let upper_confidence_bound = |child: &Node<N>| {
            fn exploration_rate(n: f32) -> f32 {
                ((1.0 + n + EXPLORATION_BASE) / EXPLORATION_BASE).ln() + EXPLORATION_INIT
            }

            // U(s, a) = Q(s, a) + C(s) * P(s, a) * sqrt(N(s)) / (1 + N(s, a))
            child.expected_reward
                + exploration_rate(visit_count)
                    * child.policy
                    * (visit_count.sqrt() / (1.0 + child.visit_count()))
        };

        // select the node to recurse into
        let (_, (turn, node)) = self
            .children
            .iter_mut()
            .map(|pair| (upper_confidence_bound(pair.1), pair))
            .max_by(|(a, _), (b, _)| a.partial_cmp(b).expect("tried to compare nan"))
            .expect("tried to select on a node without children");

        // update the game state
        game.play(turn.clone()).unwrap();
        // add the move to our path
        path.push(turn.clone());
        // continue the rollout
        node.virtual_rollout(game, path)
    }

    fn update_concrete(&mut self, reward: f32) {
        let scaled_reward = self.expected_reward * self.visits as f32;
        self.visits += 1;
        self.expected_reward = (scaled_reward + reward) / self.visits as f32;
    }
}
