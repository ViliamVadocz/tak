use tak::*;
use test::Bencher;

use super::network::Network;
use crate::agent::Agent;

#[bench]
fn forward_pass_1(b: &mut Bencher) {
    // tch::maybe_init_cuda();
    let game = Game::<5>::default();
    let network = Network::<5>::default();
    b.iter(|| network.policy_and_eval(&game))
}

fn forward_pass_n(b: &mut Bencher, n: usize) {
    tch::maybe_init_cuda();
    let game = Game::<5>::default();
    let games = vec![game; n];
    let network = Network::<5>::default();
    b.iter(|| network.policy_eval_batch(&games))
}

#[bench]
fn forward_pass_128(b: &mut Bencher) {
    forward_pass_n(b, 128)
}
