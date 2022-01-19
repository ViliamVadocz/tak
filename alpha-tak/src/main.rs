use network::Network;
use self_play::play_until_better;
use tch::Device;

#[macro_use]
extern crate lazy_static;

pub mod example;
pub mod mcts;
pub mod network;
pub mod repr;
pub mod self_play;
pub mod turn_map;

fn main() {
    println!("using gpu: {}", Device::cuda_if_available().is_cuda());

    let mut nn = Network::<4>::default();
    // let mut nn = Network::<4>::load("models/000.varstore").unwrap();
    for i in 0..1000 {
        nn = play_until_better(nn);
        println!("saving model");
        nn.save(format!("models/{i:03}.varstore")).unwrap();
    }
}
