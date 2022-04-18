use tak::*;
use tch::{Kind, Tensor};

use super::network::Network;
use crate::{config::FILTERS, repr::game_repr, DEVICE};

// Like forward_t in the nn::ModuleT trait,
// except we return two values (policy, eval)
impl<const N: usize> Network<N> {
    fn forward_conv(&self, input: Tensor, train: bool) -> Tensor {
        self.residual_blocks
            .iter()
            .fold(
                input
                    .apply_t(&self.initial_conv, train)
                    .apply_t(&self.initial_batch_norm, train)
                    .relu_(),
                |prev, res_block| res_block.forward(prev, train),
            )
            .view([-1, FILTERS * (N * N) as i64])
    }

    pub fn forward_mcts(&self, input: Tensor) -> (Tensor, Tensor) {
        let s = self.forward_conv(input, false);
        let policy = s.apply(&self.fully_connected_policy).softmax(1, Kind::Float);
        let eval = s.apply(&self.fully_connected_eval).tanh_();
        (policy, eval)
    }

    pub fn forward_training(&self, input: Tensor) -> (Tensor, Tensor) {
        let s = self.forward_conv(input, true);
        let policy = s.apply(&self.fully_connected_policy).log_softmax(1, Kind::Float);
        let eval = s.apply(&self.fully_connected_eval).tanh_();
        (policy, eval)
    }

    pub fn policy_eval_batch(&self, games: &[Game<N>]) -> (Vec<Vec<f32>>, Vec<f32>) {
        let game_tensors: Vec<_> = games.iter().map(game_repr).collect();
        let input = Tensor::stack(&game_tensors, 0).to_device_(*DEVICE, Kind::Float, true, false);
        let (policy, eval) = self.forward_mcts(input);
        let policies: Vec<Vec<f32>> = policy.into();
        let evals: Vec<f32> = eval.into();
        (policies, evals)
    }
}
