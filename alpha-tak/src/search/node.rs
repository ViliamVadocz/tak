use std::collections::HashMap;

use tak::*;

#[derive(Clone, Debug, Default)]
pub struct Node<const N: usize> {
    pub result: Option<GameResult>,
    pub policy: f32,
    pub expected_reward: f32,
    pub visited_count: u32,
    pub children: Option<HashMap<Turn<N>, Node<N>>>,
}

impl<const N: usize> Node<N> {
    pub fn init(policy: f32) -> Self {
        Node {
            policy,
            ..Default::default()
        }
    }
}
