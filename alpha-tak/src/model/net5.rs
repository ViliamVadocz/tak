use std::path::Path;

use arrayvec::ArrayVec;
use tak::*;
use tch::{nn, Kind, Tensor};

use super::{
    network::{Eval, Network, Policy},
    res_block::ResBlock,
};
use crate::{
    repr::{game_repr, input_channels, possible_moves},
    DEVICE,
};

const RES_BLOCKS: usize = 8;
const FILTERS: i64 = 128;
const N: i64 = 5;

#[derive(Debug)]
pub struct Net5 {
    vs: nn::VarStore,
    initial_conv: nn::Conv2D,
    initial_batch_norm: nn::BatchNorm,
    residual_blocks: ArrayVec<ResBlock, RES_BLOCKS>,
    fully_connected_policy: nn::Linear,
    fully_connected_eval: nn::Linear,
}

impl Default for Net5 {
    fn default() -> Self {
        let vs = nn::VarStore::new(*DEVICE);
        let root = &vs.root();

        let conv_config = nn::ConvConfig {
            padding: 1,
            ..Default::default()
        };

        let initial_conv = nn::conv2d(root, input_channels(N as usize) as i64, FILTERS, 3, conv_config);
        let initial_batch_norm = nn::batch_norm2d(root, FILTERS, Default::default());

        let mut residual_blocks = ArrayVec::new();
        for _ in 0..RES_BLOCKS {
            let conv1 = nn::conv2d(root, FILTERS, FILTERS, 3, conv_config);
            let conv2 = nn::conv2d(root, FILTERS, FILTERS, 3, conv_config);
            let batch_norm1 = nn::batch_norm2d(root, FILTERS, Default::default());
            let batch_norm2 = nn::batch_norm2d(root, FILTERS, Default::default());
            residual_blocks.push(ResBlock {
                conv1,
                conv2,
                batch_norm1,
                batch_norm2,
            });
        }

        let fully_connected_policy = nn::linear(
            root,
            FILTERS * N * N,
            possible_moves(N as usize) as i64,
            Default::default(),
        );
        let fully_connected_eval = nn::linear(root, FILTERS * N * N, 1, Default::default());

        Net5 {
            vs,
            initial_conv,
            initial_batch_norm,
            residual_blocks,
            fully_connected_policy,
            fully_connected_eval,
        }
    }
}

impl Net5 {
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
}

impl Network<5> for Net5 {
    fn vs(&self) -> &nn::VarStore {
        &self.vs
    }

    fn save<T: AsRef<Path>>(&self, path: T) -> Result<(), tch::TchError> {
        self.vs.save(path)?;
        Ok(())
    }

    fn load<T: AsRef<Path>>(path: T) -> Result<Self, tch::TchError> {
        let mut nn = Self::default();
        nn.vs.load(path)?;
        Ok(nn)
    }

    fn forward_mcts(&self, input: Tensor) -> (Tensor, Tensor) {
        let s = self.forward_conv(input, false);
        let policy = s.apply(&self.fully_connected_policy).softmax(1, Kind::Float);
        let eval = s.apply(&self.fully_connected_eval).tanh_();
        (policy, eval)
    }

    fn forward_training(&self, input: Tensor) -> (Tensor, Tensor) {
        let s = self.forward_conv(input, true);
        let policy = s.apply(&self.fully_connected_policy).log_softmax(1, Kind::Float);
        let eval = s.apply(&self.fully_connected_eval).tanh_();
        (policy, eval)
    }

    fn policy_eval(&self, games: &[Game<5>]) -> Vec<(Policy, Eval)> {
        if games.is_empty() {
            return Vec::new();
        }
        let game_tensors: Vec<_> = games.iter().map(game_repr).collect();
        let input = Tensor::stack(&game_tensors, 0).to_device_(*DEVICE, Kind::Float, true, false);
        let (policy, eval) = self.forward_mcts(input);
        let policies: Vec<Vec<f32>> = policy.into();
        let evals: Vec<f32> = eval.into();
        policies.into_iter().zip(evals.into_iter()).collect()
    }
}
