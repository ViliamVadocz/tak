use tak::*;

use super::{node::Node, turn_map::Lut};
use crate::config::{EXPLORATION_BASE, EXPLORATION_INIT};

impl<const N: usize> Node<N>
where
    Turn<N>: Lut,
{
    pub fn rollout(&mut self, game: &mut Game<N>, path: &mut Vec<Turn<N>>) -> GameResult {
        let result = if self.is_initialized() {
            // we've been here before - recurse if we can
            match self.result {
                GameResult::Ongoing => self.select(game, path),
                r => r,
            }
        } else {
            // uninitialized node - cache the winner and stop the recursion
            self.result = game.winner();
            self.result
        };

        match result {
            // our rollout ended on a terminal node - propagate a concrete score
            GameResult::Winner { colour, .. } => {
                self.update_concrete(if colour == game.to_move { -1.0 } else { 1.0 })
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
            self.children[&turn].devirtualize_path(path, result)
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
            .map(|pair| (pair, self.upper_confidence_bound(&pair.1)))
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("tried comparing nan"))
            .expect("tried to select on a node without children");

        // update the game state
        game.play(turn.clone());
        // add the move to our path
        path.push(turn.clone());
        // continue the rollout
        node.rollout(game, path)
    }

    fn upper_confidence_bound(&self, child: &Node<N>) -> f32 {
        fn exploration_rate(n: f32) -> f32 {
            ((1.0 + n + EXPLORATION_BASE) / EXPLORATION_BASE).ln() + EXPLORATION_INIT
        }

        // U(s, a) = Q(s, a) + C(s) * P(s, a) * sqrt(N(s)) / (1 + N(s, a))
        child.expected_reward
            + exploration_rate(self.visit_count())
                * child.policy
                * (self.visit_count().sqrt() / (1.0 + child.visit_count()))
    }

    fn update_concrete(&mut self, reward: f32) {
        let scaled_reward = self.expected_reward * self.visits as f32;
        self.visits += 1;
        self.expected_reward = (scaled_reward + reward) / self.visits as f32;
    }
}
