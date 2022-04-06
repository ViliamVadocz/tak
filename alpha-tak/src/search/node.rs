use std::collections::HashMap;

use tak::*;

#[derive(Clone, Debug)]
pub struct Node<const N: usize> {
    pub result: GameResult,
    pub policy: f32,
    pub expected_reward: f32,
    pub visited_count: u32,
    pub virtual_count: u32,
    pub children: HashMap<Turn<N>, Option<Node<N>>>,
}
