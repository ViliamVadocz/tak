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
        let x = rand::random::<usize>() % 1_000_000;
        nn.save(format!("models/{x:06}.model")).unwrap();
    }
}
