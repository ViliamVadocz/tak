use std::{error::Error, ops::Add, path::Path};

use arrayvec::ArrayVec;
use tak::game::Game;
use tch::{nn, nn::ConvConfig, Device, Kind, Tensor};

use crate::repr::{game_repr, input_channels, moves_dims};

const RES_BLOCKS: usize = 8;
const FILTERS: i64 = 256;

#[derive(Debug)]
struct ResBlock {
    conv1: nn::Conv2D,
    conv2: nn::Conv2D,
    batch_norm1: nn::BatchNorm,
    batch_norm2: nn::BatchNorm,
}

impl ResBlock {
    fn forward(&self, input: Tensor, train: bool) -> Tensor {
        input
            .apply_t(&self.conv1, train)
            .apply_t(&self.batch_norm1, train)
            .relu()
            .apply_t(&self.conv2, train)
            .apply_t(&self.batch_norm2, train)
            .add(&input)
            .relu()
    }
}

#[derive(Debug)]
pub struct Network<const N: usize> {
    pub vs: nn::VarStore,
    initial_conv: nn::Conv2D,
    initial_batch_norm: nn::BatchNorm,
    residual_blocks: ArrayVec<ResBlock, RES_BLOCKS>,
    fully_connected_policy: nn::Linear,
    fully_connected_eval: nn::Linear,
}

impl<const N: usize> Network<N> {
    pub fn save<T: AsRef<Path>>(&self, path: T) -> Result<(), Box<dyn Error>> {
        self.vs.save(path)?;
        Ok(())
    }

    pub fn load<T: AsRef<Path>>(path: T) -> Result<Network<N>, Box<dyn Error>> {
        let mut nn = Self::default();
        nn.vs.load(path)?;
        Ok(nn)
    }
}

impl<const N: usize> Default for Network<N> {
    fn default() -> Self {
        let vs = nn::VarStore::new(Device::cuda_if_available());
        let root = &vs.root();

        let conv_config = ConvConfig {
            padding: 1,
            ..Default::default()
        };

        let initial_conv = nn::conv2d(root, input_channels(N) as i64, FILTERS, 3, conv_config);
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
            FILTERS * (N * N) as i64,
            moves_dims(N) as i64,
            Default::default(),
        );
        let fully_connected_eval = nn::linear(root, FILTERS * (N * N) as i64, 1, Default::default());

        Network {
            vs,
            initial_conv,
            initial_batch_norm,
            residual_blocks,
            fully_connected_policy,
            fully_connected_eval,
        }
    }
}

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
                    .relu(),
                |prev, res_block| res_block.forward(prev, train),
            )
            .view([-1, FILTERS * (N * N) as i64])
    }

    pub fn forward_mcts(&self, input: Tensor) -> (Tensor, Tensor) {
        let s = self.forward_conv(input, false);
        let policy = s.apply(&self.fully_connected_policy).softmax(1, Kind::Float);
        let eval = s.apply(&self.fully_connected_eval).tanh();
        (policy, eval)
    }

    pub fn forward_training(&self, input: Tensor) -> (Tensor, Tensor) {
        let s = self.forward_conv(input, true);
        let policy = s.apply(&self.fully_connected_policy).log_softmax(1, Kind::Float);
        let eval = s.apply(&self.fully_connected_eval).tanh();
        (policy, eval)
    }

    pub fn policy_eval_batch(&self, games: &[Game<N>]) -> (Vec<Vec<f32>>, Vec<f32>) {
        let game_tensors: Vec<_> = games.iter().map(game_repr).collect();
        let input = Tensor::stack(&game_tensors, 0).to_device(Device::cuda_if_available());
        let (policy, eval) = self.forward_mcts(input);
        let policies: Vec<Vec<f32>> = policy.into();
        let evals: Vec<f32> = eval.into();
        (policies, evals)
    }
}
