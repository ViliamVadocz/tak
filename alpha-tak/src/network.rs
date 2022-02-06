use std::{error::Error, path::Path};

use arrayvec::ArrayVec;
use tak::{tile::Tile, turn::Turn, game::Game};
use tch::{
    data::Iter2,
    nn,
    nn::{ConvConfig, OptimizerConfig},
    Device,
    Kind,
    Tensor,
};

use crate::{
    example::Example,
    repr::{input_dims, moves_dims, game_repr},
    turn_map::Lut,
};

const EPOCHS: usize = 1; // idk seems to over-fit otherwise
const BATCH_SIZE: i64 = 20_000;
const LEARNING_RATE: f64 = 1e-4;
const WEIGHT_DECAY: f64 = 1e-4;

const CONV_LAYERS: usize = 16;

#[derive(Debug)]
pub struct Network<const N: usize> {
    vs: nn::VarStore,
    convolutions: ArrayVec<nn::Conv2D, CONV_LAYERS>,
    batch_norms: ArrayVec<nn::BatchNorm, CONV_LAYERS>,
    fully_connected_policy: nn::Linear,
    fully_connected_eval: nn::Linear,
}

impl<const N: usize> Network<N> {
    // TODO validation data
    pub fn train(&mut self, examples: &[Example<N>])
    where
        Turn<N>: Lut,
        [[Option<Tile>; N]; N]: Default,
    {
        println!("starting training with {} examples", examples.len());
        let mut opt = nn::Adam {
            wd: WEIGHT_DECAY,
            ..Default::default()
        }
        .build(&self.vs, LEARNING_RATE)
        .unwrap();

        let symmetries = examples.iter().flat_map(|ex| ex.to_tensors());
        let mut inputs = Vec::new();
        let mut targets = Vec::new();
        for (game, pi, v) in symmetries {
            inputs.push(game);
            targets.push(Tensor::cat(&[pi, v], 0));
        }

        for epoch in 0..EPOCHS {
            // Batch examples
            let mut batch_iter = Iter2::new(
                &Tensor::stack(&inputs, 0),
                &Tensor::stack(&targets, 0),
                BATCH_SIZE,
            );
            let batch_iter = batch_iter.shuffle();

            for (mut input, mut target) in batch_iter {
                input = input.to_device(Device::cuda_if_available());
                target = target.to_device(Device::cuda_if_available());

                let batch_size = input.size()[0];
                let (policy, eval) = self.forward_training(input);

                // Get target
                let mut vec = target.split(moves_dims(N) as i64, 1);
                let z = vec.pop().unwrap();
                let p = vec.pop().unwrap();

                let loss_p = -(p * policy).sum(Kind::Float) / batch_size;
                let loss_z = (z - eval).square().sum(Kind::Float) / batch_size;
                println!("epoch {epoch}:\t p={loss_p:?}\t z={loss_z:?}");
                let total_loss = loss_z + loss_p;

                opt.zero_grad();
                opt.backward_step(&total_loss);
            }
        }
    }

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
        // TODO make sure dimensions work for any board size
        let vs = nn::VarStore::new(Device::cuda_if_available());
        let root = &vs.root();
        let [d1, _d2, _d3] = input_dims(N);

        let conv_config = ConvConfig {
            padding: 1,
            ..Default::default()
        };
        let mut convolutions = ArrayVec::new();
        let mut batch_norms = ArrayVec::new();
        convolutions.push(nn::conv2d(root, d1 as i64, 128, 3, conv_config));
        batch_norms.push(nn::batch_norm2d(root, 128, Default::default()));
        for _ in 1..CONV_LAYERS {
            convolutions.push(nn::conv2d(root, 128, 128, 3, conv_config));
            batch_norms.push(nn::batch_norm2d(root, 128, Default::default()));
        }
        let fully_connected_policy = nn::linear(
            root,
            (N * N * 128) as i64,
            moves_dims(N) as i64,
            Default::default(),
        );
        let fully_connected_eval = nn::linear(root, (N * N * 128) as i64, 1, Default::default());
        Network {
            vs,
            convolutions,
            batch_norms,
            fully_connected_policy,
            fully_connected_eval,
        }
    }
}

// Like forward_t in the nn::ModuleT trait, except we return two values (policy,
// eval)
impl<const N: usize> Network<N> {
    pub fn forward_mcts(&self, input: Tensor) -> (Tensor, Tensor) {
        let s = self
            .convolutions
            .iter()
            .zip(&self.batch_norms)
            .fold(input, |s, (conv, norm)| s.apply(conv).apply_t(norm, false))
            .reshape(&[-1, (N * N * 128) as i64]);
        let policy = s.apply(&self.fully_connected_policy).softmax(1, Kind::Float);
        let eval = s.apply(&self.fully_connected_eval).tanh();
        (policy, eval)
    }

    pub fn forward_training(&self, input: Tensor) -> (Tensor, Tensor) {
        let s = self
            .convolutions
            .iter()
            .zip(&self.batch_norms)
            .fold(input, |s, (conv, norm)| s.apply(conv).apply_t(norm, true))
            .reshape(&[-1, (N * N * 128) as i64]);
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
