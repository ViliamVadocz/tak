use tak::{tile::Tile, turn::Turn};
use tch::{
    data::Iter2,
    nn::{self, OptimizerConfig},
    Device,
    Kind,
    Tensor,
};

use crate::{example::Example, network::Network, repr::moves_dims, turn_map::Lut, MAX_EXAMPLES};

const EPOCHS: usize = 1; // idk seems to over-fit otherwise
const BATCH_SIZE: i64 = 10_000;
const LEARNING_RATE: f64 = 1e-4;
const WEIGHT_DECAY: f64 = 1e-4;

impl<const N: usize> Network<N> {
    // TODO validation data
    pub fn train(&mut self, examples: &[Example<N>])
    where
        Turn<N>: Lut,
        [[Option<Tile>; N]; N]: Default,
    {
        if examples.len() > MAX_EXAMPLES {
            println!("too many examples, splitting training up");
            self.train(&examples[0..MAX_EXAMPLES]);
            self.train(&examples[MAX_EXAMPLES..]);
            return;
        }
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
            // batch examples
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

                // get target
                let mut vec = target.split(moves_dims(N) as i64, 1);
                let z = vec.pop().unwrap();
                let p = vec.pop().unwrap();

                // calculate loss
                let loss_p = -(p * policy).sum(Kind::Float) / batch_size;
                let loss_z = (z - eval).square().sum(Kind::Float) / batch_size;
                println!("epoch {epoch}:\t p={loss_p:?}\t z={loss_z:?}");
                let total_loss = loss_z + loss_p;

                opt.zero_grad();
                opt.backward_step(&total_loss);
            }
        }
    }
}
