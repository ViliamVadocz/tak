#![feature(thread_is_running)]

use std::time::SystemTime;

use example::Example;
use network::Network;
use tak::{tile::Tile, turn::Turn};
use tch::Cuda;
use turn_map::Lut;

use crate::{pit::pit_async, self_play::self_play_async};

#[macro_use]
extern crate lazy_static;

mod agent;
mod example;
mod mcts;
mod network;
mod pit;
mod repr;
mod self_play;
mod turn_map;

const MAX_EXAMPLES: usize = 1_000_000;
const WIN_RATE_THRESHOLD: f64 = 0.55;

fn main() {
    tch::maybe_init_cuda();
    println!("CUDA: {}", Cuda::is_available());

    let mut args = std::env::args();
    let mut nn = if let Some(model_path) = args.nth(1) {
        Network::<4>::load(&model_path).unwrap_or_else(|_| panic!("couldn't load model at {model_path}"))
    } else {
        println!("generating random model");
        Network::<4>::default()
    };

    let mut examples = Vec::new();
    loop {
        nn = play_until_better(nn, &mut examples);
        println!("saving model");

        let sys_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        nn.save(format!("models/{sys_time}.model")).unwrap();
    }
}

/// Do self-play and test against previous iteration
/// until an improvement is seen.
pub fn play_until_better<const N: usize>(network: Network<N>, examples: &mut Vec<Example<N>>) -> Network<N>
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    loop {
        // examples.extend(self_play(&network).into_iter());
        examples.extend(self_play_async(&network).into_iter());
        if examples.len() > MAX_EXAMPLES {
            examples.reverse();
            examples.truncate(MAX_EXAMPLES);
            examples.reverse();
        }

        let mut new_network = copy(&network);
        new_network.train(examples);

        println!("pitting two networks against each other");
        // let results = pit(&new_network, &network);
        let results = pit_async(&new_network, &network);
        println!("{:?}", results);
        if results.win_rate() > WIN_RATE_THRESHOLD {
            return new_network;
        }
    }
}

fn copy<const N: usize>(network: &Network<N>) -> Network<N> {
    // copy network values by file (UGLY)
    let mut dir = std::env::temp_dir();
    dir.push("model");
    network.save(&dir).unwrap();
    Network::<N>::load(&dir).unwrap()
}
