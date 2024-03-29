use rand_distr::{Dirichlet, Distribution};

use super::node::Node;

impl Node {
    pub fn apply_dirichlet(&mut self, alpha: f32, ratio: f32) {
        assert!(
            self.visits > 0,
            "cannot apply dirichlet noise without initialized policy"
        );
        let dirichlet = Dirichlet::new(&vec![alpha; self.children.len()]).unwrap();
        let samples = dirichlet.sample(&mut rand::thread_rng());
        for ((_move, node), noise) in self.children.iter_mut().zip(samples) {
            node.policy = noise * ratio + node.policy * (1. - ratio);
        }
    }
}
