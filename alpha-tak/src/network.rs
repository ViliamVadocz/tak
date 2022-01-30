use std::{error::Error, path::Path};

use tch::{
    data::Iter2,
    nn,
    nn::{ConvConfig, ModuleT, OptimizerConfig},
    Device,
    Kind,
    Tensor,
};

use crate::{
    example::Example,
    repr::{game_repr, input_dims, moves_dims},
};

const EPOCHS: usize = 10;
const BATCH_SIZE: i64 = 1_000_000;
const LEARNING_RATE: f64 = 1e-4;
const WEIGHT_DECAY: f64 = 1e-4;

#[derive(Debug)]
pub struct Network<const N: usize> {
    vs: nn::VarStore,
    conv1: nn::Conv2D,
    conv2: nn::Conv2D,
    conv3: nn::Conv2D,
    fc1: nn::Linear,
    fc2: nn::Linear,
}

impl<const N: usize> Network<N> {
    pub fn train(&mut self, examples: &[Example<N>]) {
        println!("starting training with {} examples", examples.len());
        let mut opt = nn::Adam {
            wd: WEIGHT_DECAY,
            ..Default::default()
        }
        .build(&self.vs, LEARNING_RATE)
        .unwrap();

        let games: Vec<_> = examples
            .iter()
            .map(|Example { game, .. }| game_repr(game))
            .collect();
        let targets: Vec<_> = examples
            .iter()
            .map(|Example { pi, v, .. }| Tensor::cat(&[pi, v], 0))
            .collect();

        for epoch in 0..EPOCHS {
            // Batch examples
            let mut batch_iter =
                Iter2::new(&Tensor::stack(&games, 0), &Tensor::stack(&targets, 0), BATCH_SIZE);
            let batch_iter = batch_iter
                .to_device(Device::cuda_if_available())
                .return_smaller_last_batch()
                .shuffle();

            for (input, target) in batch_iter {
                let batch_size = input.size()[0];
                let output = self.forward_t(&input, true);
                // get prediction
                let mut vec = output.split(moves_dims(N) as i64, 1);
                let eval = vec.pop().unwrap();
                let policy = vec.pop().unwrap();

                // Get target
                let mut vec = target.split(moves_dims(N) as i64, 1);
                let z = vec.pop().unwrap();
                let p = vec.pop().unwrap();

                let loss_p = -(p * policy).sum(Kind::Float) / batch_size;
                let loss_z = (z - eval).square().sum(Kind::Float) / batch_size;
                println!("{epoch}: p={loss_p:.4?}\tz={loss_z:.4?}");
                let total_loss = loss_z + loss_p;

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
        let conv1 = nn::conv2d(root, d1 as i64, 128, 3, ConvConfig {
            padding: 1,
            ..Default::default()
        });
        let conv2 = nn::conv2d(root, 128, 128, 3, ConvConfig {
            padding: 1,
            ..Default::default()
        });
        let conv3 = nn::conv2d(root, 128, 128, 3, ConvConfig {
            padding: 1,
            ..Default::default()
        });
        let fc1 = nn::linear(
            root,
            (N * N * 128) as i64,
            moves_dims(N) as i64,
            Default::default(),
        );
        let fc2 = nn::linear(root, (N * N * 128) as i64, 1, Default::default());
        Network {
            vs,
            conv1,
            conv2,
            conv3,
            fc1,
            fc2,
        }
    }
}

impl<const N: usize> nn::ModuleT for Network<N> {
    fn forward_t(&self, input: &Tensor, _train: bool) -> Tensor {
        let s = input
            .apply(&self.conv1)
            .apply(&self.conv2)
            .apply(&self.conv3)
            .reshape(&[-1, (N * N * 128) as i64]);
        let policy = s.apply(&self.fc1).log_softmax(1, Kind::Float);
        let eval = s.apply(&self.fc2).tanh();
        // would be nice if I could just return two values
        Tensor::cat(&[policy, eval], 1)
    }
}
