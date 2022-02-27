use std::sync::mpsc::{Receiver, Sender};

use tak::*;
use tch::Device;

use crate::{model::network::Network, repr::game_repr};

pub trait Agent<const N: usize> {
    fn policy_and_eval(&self, game: &Game<N>) -> (Vec<f32>, f32);
}

impl<const N: usize> Agent<N> for Network<N> {
    fn policy_and_eval(&self, game: &Game<N>) -> (Vec<f32>, f32) {
        let input = game_repr(game).to_device(Device::cuda_if_available());
        let (policy, eval) = self.forward_mcts(input.unsqueeze(0));
        (policy.into(), eval.into())
    }
}

pub struct Batcher<const N: usize> {
    tx: Sender<Game<N>>,
    rx: Receiver<(Vec<f32>, f32)>,
}

impl<const N: usize> Batcher<N> {
    pub fn new(tx: Sender<Game<N>>, rx: Receiver<(Vec<f32>, f32)>) -> Self {
        Batcher { tx, rx }
    }
}

impl<const N: usize> Agent<N> for Batcher<N> {
    fn policy_and_eval(&self, game: &Game<N>) -> (Vec<f32>, f32) {
        self.tx.send(game.clone()).unwrap();
        self.rx.recv().unwrap()
    }
}
