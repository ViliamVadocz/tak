use tak::{GameResult, *};

use super::{move_map::move_index, node::Node};
use crate::model::network::{Eval, Network, Policy};

// search
const EXPLORATION_BASE: f32 = 500.0;
const EXPLORATION_INIT: f32 = 4.0;

fn exploration_rate(n: f32) -> f32 {
    ((1.0 + n + EXPLORATION_BASE) / EXPLORATION_BASE).ln() + EXPLORATION_INIT
}

impl Node {
    /// Do a basic rollout.
    pub fn rollout<const N: usize, NET: Network<N>>(&mut self, mut game: Game<N>, network: &NET) {
        let mut path = vec![];
        // Perform a virtual rollout.
        if matches!(self.virtual_rollout(&mut game, &mut path), GameResult::Ongoing) {
            // The game result isn't concrete - devirtualize the path.
            self.devirtualize_path::<N, _>(&mut path.into_iter(), &network.policy_eval(&[game])[0]);
        }
    }

    #[must_use]
    pub fn virtual_rollout<const N: usize>(
        &mut self,
        game: &mut Game<N>,
        path: &mut Vec<usize>,
    ) -> GameResult {
        let curr_color = game.to_move;

        let result = if self.is_initialized() {
            // We've been here before - recurse if we can.
            match self.result {
                GameResult::Ongoing => self.select(game, path),
                r => r,
            }
        } else {
            // Uninitialized node - initialize it and stop recursion.
            self.result = game.result();
            if self.result == GameResult::Ongoing {
                let possible_moves = game.possible_moves();
                let temp_policy = 1.0 / possible_moves.len() as f32;
                self.children = possible_moves
                    .into_iter()
                    .map(|m| (m, Node::new(temp_policy)))
                    .collect();
            }
            self.result
        };

        match result {
            // Our rollout ended on a terminal node - propagate a concrete score.
            GameResult::Winner { color, .. } => {
                self.update_concrete(if color == curr_color { -1.0 } else { 1.0 })
            }
            GameResult::Draw { .. } => self.update_concrete(0.0),

            // We've cut the recursion short of a terminal node - count a virtual visit.
            GameResult::Ongoing => self.virtual_visits += 1,
        }

        result
    }

    pub fn devirtualize_path<const N: usize, I: Iterator<Item = usize>>(
        &mut self,
        path: &mut I,
        net_output: &(Policy, Eval),
    ) -> f32 {
        self.virtual_visits -= 1;

        let eval = if let Some(index) = path.next() {
            self.children[index].1.devirtualize_path::<N, _>(path, net_output)
        } else {
            let (policy, eval) = net_output;

            // Replace the temporary policies with the correct values.
            self.children.iter_mut().for_each(|(mov, child)| {
                child.policy = policy[move_index(mov, N)];
            });

            *eval
        };
        // Negate eval because we are switching the perspective.
        let eval = -eval;

        self.update_concrete(eval);
        eval
    }

    #[must_use]
    fn select<const N: usize>(&mut self, game: &mut Game<N>, path: &mut Vec<usize>) -> GameResult {
        let visit_count = self.visit_count();
        let upper_confidence_bound = |child: &Node| -> f32 {
            // U(s, a) = Q(s, a) + C(s) * P(s, a) * sqrt(N(s)) / (1 + N(s, a))
            child.expected_reward
                + exploration_rate(visit_count)
                    * child.policy
                    * (visit_count.sqrt() / (1.0 + child.visit_count()))
        };

        // Select the node to recurse into.
        let (_ucb, (index, (my_move, node))) = self
            .children
            .iter_mut()
            .enumerate()
            .map(|(index, (mov, child))| (upper_confidence_bound(child), (index, (mov, child))))
            .max_by(|(a, _), (b, _)| a.partial_cmp(b).expect("tried comparing nan"))
            .expect("tried to select on a node without children");
        // Update the game state.
        game.play(*my_move).unwrap();
        // Add the move to our path.
        path.push(index);
        // Continue the rollout.
        node.virtual_rollout(game, path)
    }

    fn update_concrete(&mut self, reward: f32) {
        let scaled_reward = self.expected_reward * self.visits as f32;
        self.visits += 1;
        self.expected_reward = (scaled_reward + reward) / self.visits as f32;
    }
}
