use network::Network;
use self_play::play_until_better;
use tch::Cuda;

#[macro_use]
extern crate lazy_static;

pub mod example;
pub mod mcts;
pub mod network;
pub mod repr;
pub mod self_play;
pub mod turn_map;

const START: usize = 0;

fn main() {
    tch::maybe_init_cuda();
    println!("CUDA: {}", Cuda::is_available());

    let mut nn = if START == 0 {
        Network::<4>::default()
    } else {
        Network::<4>::load(format!("models/{START:03}.model")).unwrap()
    };
    let mut examples = Vec::new();
    for i in (START + 1)..1000 {
        nn = play_until_better(nn, &mut examples);
        println!("saving model");
        nn.save(format!("models/{i:03}.model")).unwrap();
    }
}
