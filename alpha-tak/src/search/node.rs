use std::collections::HashMap;

use tak::*;

#[derive(Clone, Debug, Default)]
pub struct Node<const N: usize>(pub Option<InnerNode<N>>);

impl<const N: usize> Node<N> {
    pub fn initialize(&mut self, game: Game<N>) -> InnerNode<N> {
        let node = InnerNode {
            result: game.winner(),
            policy: 1.0,
            expected_reward: 0.0,
            visited_count: 0,
            virtual_count: 0,
            children: Default::default(),
        };
        self.0 = Some(node);
        node
    }
}

#[derive(Clone, Debug)]
pub struct InnerNode<const N: usize> {
    pub result: GameResult,
    pub policy: f32,
    pub expected_reward: f32,
    pub visited_count: u32,
    pub virtual_count: u32,
    pub children: HashMap<Turn<N>, Node<N>>,
}
