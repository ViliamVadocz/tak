use rand::thread_rng;
use rand_distr::{Distribution, WeightedIndex};
use tak::*;

use super::node::Node;

impl Node {
    fn check_initialized(&self) {
        assert!(self.is_initialized(), "node must be initialized");
    }

    /// Generate an improved policy from the visits.
    pub fn improved_policy(&self) -> Vec<(Move, u32)> {
        self.check_initialized();
        // After many rollouts the visit counts become a better
        // estimate for policy (not normalized).
        self.children
            .iter()
            .map(|(mov, node)| (*mov, node.visits))
            .collect()
    }

    /// Get the sub-tree for the given move.
    /// This allows tree reuse.
    #[must_use]
    pub fn play(self, my_move: Move) -> Node {
        self.check_initialized();

        let (_, child) = self
            .children
            .into_iter()
            .find(|(mov, _node)| mov == &my_move)
            .expect("tried to play an invalid move");
        child
    }

    /// Select a move to play.
    /// When exploitation is true, it will return the move with the most visits.
    /// If exploitation is false, it will return a random move weighted by the
    /// number of visits.
    pub fn pick_move(&self, exploitation: bool) -> Move {
        let improved_policy = self.improved_policy();

        if exploitation {
            // When exploiting always pick the move with the most visits.
            improved_policy
                .into_iter()
                .max_by_key(|(_, value)| *value)
                .unwrap()
                .0
        } else {
            // Split into moves and weights.
            let (mut moves, weights): (Vec<_>, Vec<_>) = improved_policy.into_iter().unzip();
            // Randomly pick based on weights from the improved policy.
            let distr = WeightedIndex::new(&weights).unwrap();
            let index = distr.sample(&mut thread_rng());
            moves.swap_remove(index)
        }
    }
}
