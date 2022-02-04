use tak::game::Game;
use tch::{nn::ModuleT, Device};

use crate::{
    network::Network,
    repr::{game_repr, moves_dims},
};

pub trait Agent<const N: usize> {
    fn policy_and_eval(&self, game: &Game<N>) -> (Vec<f32>, f32);
}

impl<const N: usize> Agent<N> for Network<N> {
    fn policy_and_eval(&self, game: &Game<N>) -> (Vec<f32>, f32) {
        let input = game_repr(game).to_device(Device::cuda_if_available());
        let output = self.forward_t(&input.unsqueeze(0), false);
        let mut vec = output.split(moves_dims(N) as i64, 1);
        let eval = vec.pop().unwrap().into();
        let policy = vec.pop().unwrap().exp().into(); // undoing log (UGLY)
        (policy, eval)
    }
}
