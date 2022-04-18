use tak::*;

use crate::{agent::Agent, repr::moves_dims, search::node::Node};

struct TestAgent {}
impl<const N: usize> Agent<N> for TestAgent {
    fn policy_and_eval(&self, game: &Game<N>) -> (Vec<f32>, f32) {
        let l = game.possible_turns().len() as f32;
        (vec![1. / l; moves_dims(3)], 0.)
    }
}

#[test]
fn mate_in_one() {
    let mut game = Game::<3>::from_ptn("1. a3 c3 2. c2 a2").unwrap();
    let mut node = Node::default();
    for _ in 0..1000 {
        node.rollout(game.clone(), &TestAgent {});
    }
    let turn = node.pick_move(true);
    game.play(turn).unwrap();
    assert_eq!(game.winner(), GameResult::Winner {
        colour: Colour::White,
        road: true
    })
}

#[test]
fn prevent_mate_in_two() {
    let mut game = Game::<3>::from_ptn("1. a3 c3 2. c2").unwrap();
    let mut node = Node::default();

    // black move
    for _ in 0..1000 {
        node.rollout(game.clone(), &TestAgent {});
    }
    let turn = node.pick_move(true);
    node = node.play(&turn);
    game.play(turn).unwrap();
    assert_eq!(game.winner(), GameResult::Ongoing);

    // white move
    for _ in 0..1000 {
        node.rollout(game.clone(), &TestAgent {});
    }
    let turn = node.pick_move(true);
    let _ = node.play(&turn);
    game.play(turn).unwrap();
    assert_eq!(game.winner(), GameResult::Ongoing);
}

#[test]
fn white_win_3s() {
    let mut game = Game::<3>::from_ptn("1. a3 c3").unwrap();
    let mut node = Node::default();

    while matches!(game.winner(), GameResult::Ongoing) {
        for _ in 0..100_000 {
            node.rollout(game.clone(), &TestAgent {});
        }
        println!("{}", node.debug(None));

        let turn = node.pick_move(true);
        node = node.play(&turn);
        game.play(turn).unwrap();
    }

    assert_eq!(game.winner(), GameResult::Winner {
        colour: Colour::White,
        road: true
    });
}
