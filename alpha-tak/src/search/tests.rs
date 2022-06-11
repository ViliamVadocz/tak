use tak::*;

use super::node::Node;
use crate::{model::network::Network, repr::output_size};

#[derive(Default)]
struct DummyNet {}
impl<const N: usize> Network<N> for DummyNet {
    fn vs(&self) -> &tch::nn::VarStore {
        unimplemented!()
    }

    fn save<T: AsRef<std::path::Path>>(&self, _path: T) -> Result<(), tch::TchError> {
        unimplemented!()
    }

    fn load<T: AsRef<std::path::Path>>(_path: T) -> Result<Self, tch::TchError> {
        unimplemented!()
    }

    fn forward_mcts(&self, _input: tch::Tensor) -> (tch::Tensor, tch::Tensor) {
        unimplemented!()
    }

    fn forward_training(&self, _input: tch::Tensor) -> (tch::Tensor, tch::Tensor) {
        unimplemented!()
    }

    fn policy_eval(
        &self,
        games: &[Game<N>],
    ) -> Vec<(crate::model::network::Policy, crate::model::network::Eval)> {
        vec![(vec![1.0; output_size(N)], 0.0); games.len()]
    }
}

#[test]
fn win_in_one() {
    let mut game = Game::<3>::from_ptn_moves(&["a3", "c3", "c2", "a2"]).unwrap();
    let mut node = Node::default();

    for _ in 0..1000 {
        node.rollout(game.clone(), &DummyNet {})
    }
    game.play(node.pick_move(true)).unwrap();
    assert_eq!(game.result(), GameResult::Winner {
        color: Color::White,
        road: true
    })
}

#[test]
fn prevent_win_in_two() {
    let mut game = Game::<3>::from_ptn_moves(&["a3", "c3", "c2"]).unwrap();
    let mut node = Node::default();

    // Black move.
    for _ in 0..1000 {
        node.rollout(game.clone(), &DummyNet {});
    }
    let my_move = node.pick_move(true);
    node = node.play(my_move);
    game.play(my_move).unwrap();
    assert_eq!(game.result(), GameResult::Ongoing);

    // White move.
    for _ in 0..1000 {
        node.rollout(game.clone(), &DummyNet {});
    }
    game.play(node.pick_move(true)).unwrap();
    assert_eq!(game.result(), GameResult::Ongoing);
}
