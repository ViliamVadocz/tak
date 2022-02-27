use std::collections::HashMap;

use rand_distr::{Distribution, WeightedIndex};
use tak::*;

use super::node::Node;

impl<const N: usize> Node<N> {
    pub fn improved_policy(&self) -> HashMap<Turn<N>, u32> {
        let mut policy = HashMap::new();
        // after many rollouts the visited counts become a better estimate for policy
        // (not normalized)
        for (turn, child) in self.children.as_ref().expect("you must rollout at least once") {
            policy.insert(turn.clone(), child.visited_count);
        }
        policy
    }

    #[must_use]
    pub fn play(self, turn: &Turn<N>) -> Node<N> {
        let mut children = self.children.expect("do at least one rollout");
        children.remove(turn).expect("all turns should be in there")
    }

    pub fn pick_move(&self, exploitation: bool) -> Turn<N> {
        let improved_policy = self.improved_policy();

        if exploitation {
            // when exploiting always pick the move with largest policy
            improved_policy
                .into_iter()
                .max_by_key(|(_, value)| *value)
                .unwrap()
                .0
        } else {
            // split into turns and weights
            let mut turns = vec![];
            let mut weights = vec![];
            for (turn, weight) in improved_policy {
                turns.push(turn);
                weights.push(weight);
            }
            // randomly pick based on weights from improved policy
            let mut rng = rand::thread_rng();
            let distr = WeightedIndex::new(&weights).unwrap();
            let index = distr.sample(&mut rng);
            turns.swap_remove(index)
        }
    }
}
