use rand::{prelude::SliceRandom, thread_rng};
use tak::*;
use tch::{
    data::Iter2,
    nn::{self, Optimizer, OptimizerConfig},
    Kind,
    Tensor,
};

use super::network::Network;
use crate::{
    config::{BATCH_SIZE, LEARNING_RATE, MAX_TRAIN_SIZE, WEIGHT_DECAY},
    example::Example,
    repr::moves_dims,
    search::turn_map::Lut,
    DEVICE,
};

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

        // shuffle only the references to the examples so that the real storage
        // of examples preserves order from oldest to newest.
        let mut refs: Vec<_> = examples.iter().collect();
        refs.shuffle(&mut thread_rng());
        for chunk in refs.chunks(MAX_TRAIN_SIZE) {
            self.train_inner(&mut opt, chunk)
        }
    }

    fn train_inner(&mut self, opt: &mut Optimizer, examples: &[&Example<N>])
    where
        Turn<N>: Lut,
        [[Option<Tile>; N]; N]: Default,
    {
        // batch examples
        let mut batch_iter = {
            println!("creating symmetries");
            let symmetries = examples.iter().flat_map(|ex| ex.to_tensors());
            let mut inputs = Vec::new();
            let mut policies = Vec::new();
            let mut results = Vec::new();
            for (game, pi, v) in symmetries {
                inputs.push(game);
                policies.push(pi);
                results.push(v);
            }
            let pi = Tensor::stack(&policies, 0);
            let v = Tensor::of_slice(&results).unsqueeze_(1);
            let targets = Tensor::cat(&[pi, v], 1);
            Iter2::new(&Tensor::stack(&inputs, 0), &targets, BATCH_SIZE)
        };
        let batch_iter = batch_iter.shuffle();

        for (mut input, mut target) in batch_iter {
            input = input.to_device_(*DEVICE, Kind::Float, true, false);
            target = target.to_device_(*DEVICE, Kind::Float, true, false);

            let batch_size = input.size()[0];
            let (policy, eval) = self.forward_training(input);

            // get target
            let mut vec = target.split(moves_dims(N) as i64, 1);
            let z = vec.pop().unwrap();
            let p = vec.pop().unwrap();

            // calculate loss
            let loss_p = -(p * policy).sum(Kind::Float) / batch_size;
            let loss_z = (z - eval).square_().sum(Kind::Float) / batch_size;
            println!("p={loss_p:?}\t z={loss_z:?}");
            let total_loss = loss_z + loss_p;

            opt.zero_grad();
            opt.backward_step(&total_loss);
        }
    }
}
