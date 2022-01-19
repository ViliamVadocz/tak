use std::{error::Error, path::Path};

use tak::game::Game;
use tch::{
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

const EPOCHS: usize = 1; // TODO make bigger

#[derive(Debug)]
pub struct Network<const N: usize> {
    vs: nn::VarStore,
    conv1: nn::Conv2D,
    conv2: nn::Conv2D,
    conv3: nn::Conv2D,
    fc1: nn::Linear,
    fc2: nn::Linear,
    fc3: nn::Linear,
}

impl<const N: usize> Network<N> {
    pub fn predict(&self, game: &Game<N>, train: bool) -> (Tensor, Tensor) {
        let input = game_repr(game);
        let output = self.forward_t(&input, train);
        let mut vec = output.split(moves_dims(N) as i64, 0);
        let eval = vec.pop().unwrap();
        let policy = vec.pop().unwrap();
        (policy, eval)
    }

    pub fn train(&mut self, examples: Vec<Example<N>>) {
        println!("starting training");
        let mut opt = nn::Adam::default().build(&self.vs, 1e-4).unwrap();
        // TODO somehow batch example together
        for epoch in 0..EPOCHS {
            println!("epoch: {}", epoch);
            for Example { game, pi, v } in &examples {
                let (policy, eval) = self.predict(game, true);

                let loss_pi = -policy.dot(pi);
                let loss_v = (eval - v).square();
                let total_loss = loss_v + loss_pi;

                opt.zero_grad();  // TODO is this needed?
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
        let conv1 = nn::conv2d(root, d1 as i64, 32, 3, ConvConfig {
            padding: 3,
            ..Default::default()
        });
        let conv2 = nn::conv2d(root, 32, 64, 3, ConvConfig {
            padding: 3,
            ..Default::default()
        });
        let conv3 = nn::conv2d(root, 64, 128, 3, ConvConfig {
            padding: 3,
            ..Default::default()
        });
        let fc1 = nn::linear(root, 2048, 1024, Default::default());
        let fc2 = nn::linear(root, 1024, moves_dims(N) as i64, Default::default());
        let fc3 = nn::linear(root, 1024, 1, Default::default());
        Network {
            vs,
            conv1,
            conv2,
            conv3,
            fc1,
            fc2,
            fc3,
        }
    }
}

impl<const N: usize> nn::ModuleT for Network<N> {
    fn forward_t(&self, input: &Tensor, train: bool) -> Tensor {
        let s = input
            .unsqueeze(0)
            .apply(&self.conv1)
            .max_pool2d_default(2)
            .apply(&self.conv2)
            .max_pool2d_default(2)
            .apply(&self.conv3)
            .max_pool2d_default(2)
            .reshape(&[2048])
            .apply(&self.fc1)
            .relu()
            .dropout(0.5, train);
        let policy = s.apply(&self.fc2).log_softmax(0, Kind::Float);
        let eval = s.apply(&self.fc3).tanh();
        // would be nice if I could just return two values
        Tensor::cat(&[policy, eval], 0)
    }
}
