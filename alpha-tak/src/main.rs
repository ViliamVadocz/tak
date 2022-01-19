use network::Network;
use self_play::play_until_better;

#[macro_use]
extern crate lazy_static;

pub mod example;
pub mod mcts;
pub mod network;
pub mod repr;
pub mod self_play;
pub mod turn_map;

fn main() {
    for i in 0..1000 {
        let nn = Network::<4>::default();
        let new = play_until_better(nn);
        println!("saving model");
        new.save(format!("models/{i:03}.varstore")).unwrap();
    }
}
