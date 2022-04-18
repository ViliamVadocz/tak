use std::{collections::HashMap, thread::spawn};

use rand_distr::{Distribution, WeightedIndex};
use tak::*;

use super::node::Node;

impl<const N: usize> Node<N> {
    fn check_initialized(&self) {
        assert!(self.is_initialized(), "node must be initialized");
    }

    pub fn improved_policy(&self) -> HashMap<Turn<N>, u32> {
        self.check_initialized();
        // after many rollouts the visit counts become a better estimate
        // for policy (not normalized)
        HashMap::from_iter(
            self.children
                .iter()
                .map(|(turn, child)| (turn.clone(), child.visits)),
        )
    }

    #[must_use]
    pub fn play(mut self, turn: &Turn<N>) -> Node<N> {
        self.check_initialized();
        let child = self
            .children
            .remove(turn)
            .expect("attempted to play invalid move");

        spawn(move || drop(self));

        child
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
