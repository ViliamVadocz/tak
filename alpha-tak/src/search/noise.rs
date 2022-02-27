use rand_distr::{Dirichlet, Distribution};

use super::node::Node;

impl<const N: usize> Node<N> {
    pub fn apply_dirichlet(&mut self, alpha: f32, ratio: f32) {
        let count = self
            .children
            .as_ref()
            .expect("you must rollout at least once")
            .len();
        let dirichlet = Dirichlet::new(&vec![alpha; count]).unwrap();
        let samples = dirichlet.sample(&mut rand::thread_rng());
        for (node, noise) in self.children.as_mut().unwrap().values_mut().zip(samples) {
            node.policy = noise * ratio + node.policy * (1. - ratio);
        }
    }
}
