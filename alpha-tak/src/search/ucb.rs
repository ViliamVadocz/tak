use super::node::InnerNode;
use crate::config::{EXPLORATION_BASE, EXPLORATION_INIT};

pub fn exploration_rate(n: f32) -> f32 {
    ((1.0 + n + EXPLORATION_BASE) / EXPLORATION_BASE).ln() + EXPLORATION_INIT
}

impl<const N: usize> InnerNode<N> {
    pub fn upper_confidence_bound(&self, child: &InnerNode<N>) -> f32 {
        // U(s, a) = Q(s, a) + C(s) * P(s, a) * sqrt(N(s)) / (1 + N(s, a))
        child.expected_reward
            + exploration_rate(self.visited_count as f32)
                * child.policy
                * ((self.visited_count as f32).sqrt() / (1.0 + child.visited_count as f32))
    }
}
