use std::path::Path;

use rand::{prelude::SliceRandom, thread_rng};
use tak::*;
use tch::{
    nn::{Adam, Optimizer, OptimizerConfig, VarStore},
    Kind,
    TchError,
    Tensor,
};

use crate::{example::Example, DEVICE};

const LEARNING_RATE: f64 = 1e-4;
const WEIGHT_DECAY: f64 = 1e-4;

// Each training step is made up of multiple chunks.
// This was done to reduce GPU memory usage.
// Product of CHUNK_SIZE and CHUNKS_IN_STEP is the effective batch size.
const CHUNK_SIZE: usize = 500;
const CHUNKS_IN_STEP: usize = 5;

pub type Policy = Vec<f32>;
pub type Eval = f32;

pub trait Network<const N: usize>: Default {
    fn vs(&self) -> &VarStore;

    fn save<T: AsRef<Path>>(&self, path: T) -> Result<(), TchError>;
    fn load<T: AsRef<Path>>(path: T) -> Result<Self, TchError>;

    fn forward_mcts(&self, input: Tensor) -> (Tensor, Tensor);
    fn forward_training(&self, input: Tensor) -> (Tensor, Tensor);
    fn policy_eval(&self, games: &[Game<N>]) -> Vec<(Policy, Eval)>;

    /// Train the network on a set of examples.
    fn train(&mut self, examples: &[Example<N>]) {
        println!("starting training with {} examples", examples.len());

        let mut opt = Adam {
            wd: WEIGHT_DECAY,
            ..Default::default()
        }
        .build(self.vs(), LEARNING_RATE)
        .unwrap();

        // Shuffle only the references to the examples so that the real storage
        // of examples preserves order from oldest to newest.
        let mut refs: Vec<_> = examples.iter().collect();
        refs.shuffle(&mut thread_rng());
        // Training happens in batches made up of multiple chunks
        // (to reduce GPU memory load).
        for (i, chunk) in refs.chunks_exact(CHUNK_SIZE).enumerate() {
            self.train_inner(&mut opt, chunk, i);
        }
    }

    fn train_inner(&mut self, opt: &mut Optimizer, examples: &[&Example<N>], chunk_num: usize) {
        let symmetries = examples.iter().flat_map(|ex| ex.to_tensors());
        // Manually unzip.
        let mut inputs = Vec::new();
        let mut policies = Vec::new();
        let mut results = Vec::new();
        for (game, pi, v) in symmetries {
            inputs.push(game);
            policies.push(pi);
            results.push(v);
        }

        // Get network output.
        let input = Tensor::stack(&inputs, 0).to_device_(*DEVICE, Kind::Float, true, false);
        let batch_size = input.size()[0];
        let (policy, eval) = self.forward_training(input);

        // Get the target.
        let p = Tensor::stack(&policies, 0)
            .to_device_(*DEVICE, Kind::Float, true, false)
            .view(policy.size().as_slice());
        let z = Tensor::of_slice(&results)
            .unsqueeze_(1)
            .to_device_(*DEVICE, Kind::Float, true, false);

        // Calculate loss.
        let loss_p = -(p * policy).sum(Kind::Float) / batch_size;
        let loss_z = (z - eval).square_().sum(Kind::Float) / batch_size;
        println!("p={loss_p:?}\t z={loss_z:?}");
        let total_loss = loss_z + loss_p;

        // Back-propagate loss.
        total_loss.backward();
        // If we have done enough chunks, do an optimization step.
        if (chunk_num + 1) % CHUNKS_IN_STEP == 0 {
            println!("making step!");
            opt.step();
            opt.zero_grad();
        }
    }
}
