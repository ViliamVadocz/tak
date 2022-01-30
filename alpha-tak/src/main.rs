use network::Network;
use self_play::play_until_better;
use tch::Cuda;

#[macro_use]
extern crate lazy_static;

mod agent;
mod example;
mod mcts;
mod network;
mod repr;
mod self_play;
mod turn_map;

const START: usize = 0;

fn main() {
    tch::maybe_init_cuda();
    println!("CUDA: {}", Cuda::is_available());

    let mut nn = if let Ok(nn) = Network::<4>::load(format!("models/{START:03}.model")) {
        println!("using saved model");
        nn
    } else {
        println!("generating random model");
        Network::<4>::default()
    };

    let mut examples = Vec::new();
    let mut i = START;
    loop {
        nn = play_until_better(nn, &mut examples);
        println!("saving model");
        i += 1;
        nn.save(format!("models/{i:03}.model")).unwrap();
    }
}
