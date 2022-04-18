use std::collections::HashMap;

use tak::*;

#[derive(Clone, Debug)]
pub struct Node<const N: usize> {
    pub result: GameResult,
    pub policy: f32,
    pub expected_reward: f32,
    pub visits: u32,
    pub virtual_visits: u32,
    pub children: HashMap<Turn<N>, Node<N>>,
}

impl<const N: usize> Default for Node<N> {
    fn default() -> Self {
        Self {
            result: GameResult::Ongoing,
            policy: 1.0,
            expected_reward: 0.0,
            visits: 0,
            virtual_visits: 0,
            children: HashMap::new(),
        }
    }
}

impl<const N: usize> Node<N> {
    pub fn is_initialized(&self) -> bool {
        self.visits != 0 || self.virtual_visits != 0
    }

    pub fn is_policy_initialized(&self) -> bool {
        self.visits != 0
    }

    pub fn visit_count(&self) -> f32 {
        (self.visits + self.virtual_visits) as f32
    }
}
