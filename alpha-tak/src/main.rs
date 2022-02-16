#![feature(thread_is_running)]

use std::{fs::File, io::Write, time::SystemTime};

use example::{save_examples, Example};
use network::Network;
use rand::random;
use tak::{
    game::{Game, GameResult},
    pos::Pos,
    ptn::ToPTN,
    tile::{Shape, Tile},
    turn::Turn,
};
use tch::Cuda;
use turn_map::Lut;

use crate::example::load_examples;
#[allow(unused_imports)]
use crate::{
    mcts::Node,
    pit::{pit, pit_async},
    self_play::{self_play, self_play_async},
};

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

const MAX_EXAMPLES: usize = 100_000;
const WIN_RATE_THRESHOLD: f64 = 0.55;

pub const KOMI: i32 = 2;

fn main() {
    tch::maybe_init_cuda();
    println!("CUDA: {}", Cuda::is_available());

    let mut args = std::env::args();
    let _ = args.next();

    // load or create network
    let network = if let Some(model_path) = args.next() {
        Network::<5>::load(&model_path).unwrap_or_else(|_| panic!("couldn't load model at {model_path}"))
    } else {
        println!("generating random model");
        Network::<5>::default()
    };

    // optionally load examples
    let mut examples = Vec::new();
    for examples_path in args {
        println!("loading {examples_path}");
        examples.extend(load_examples(&examples_path).into_iter());
    }

    // begin training loop
    training_loop(network, examples)
}

pub fn training_loop<const N: usize>(mut network: Network<N>, mut examples: Vec<Example<N>>) -> !
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    loop {
        if !examples.is_empty() {
            let new_network = {
                let mut nn = copy(&network);
                nn.train(&examples);
                nn
            };

            println!("pitting two networks against each other");
            let results = pit_async(&new_network, &network);
            println!("{:?}", results);

            if results.win_rate() > WIN_RATE_THRESHOLD {
                network = new_network;
                println!("saving model");
                network.save(format!("models/{}.model", sys_time())).unwrap();

                // it seems it improves more often if only training on fresh examples
                examples.clear();

                // run an example game to qualitative analysis
                example_game(&network);
            }
        }

        // do self-play to get new examples
        let new_examples = self_play_async(&network);
        save_examples(&new_examples);

        // keep only the latest MAX_EXAMPLES examples
        examples.extend(new_examples.into_iter());
        if examples.len() > MAX_EXAMPLES {
            examples.reverse();
            examples.truncate(MAX_EXAMPLES);
            examples.reverse();
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

fn example_game<const N: usize>(network: &Network<N>)
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    const SECONDS_PER_TURN: u64 = 30;
    println!("running example game with {SECONDS_PER_TURN} seconds per turn");

    let mut game = Game::with_komi(KOMI);
    let mut turns = Vec::new();
    // opening
    let turn0 = Turn::Place {
        pos: Pos { x: 0, y: 0 },
        shape: Shape::Flat,
    };
    let turn1 = Turn::Place {
        // randomly pick between diagonal or adjacent corners
        pos: Pos {
            x: N - 1,
            y: if random::<f64>() < 0.5 { 0 } else { N - 1 },
        },
        shape: Shape::Flat,
    };
    turns.push(turn0.to_ptn());
    turns.push(turn1.to_ptn());
    game.play(turn0).unwrap();
    game.play(turn1).unwrap();

    let mut node = Node::default();
    while matches!(game.winner(), GameResult::Ongoing) {
        // do rollouts
        let start_turn = SystemTime::now();
        while SystemTime::now().duration_since(start_turn).unwrap().as_secs() < SECONDS_PER_TURN {
            node.rollout(game.clone(), network);
        }
        println!("{}", node.debug(&game, Some(5)));
        let turn = node.pick_move(true);
        turns.push(turn.to_ptn());
        node = node.play(&turn);
        game.play(turn).unwrap();
    }

    println!("result: {:?}\n{}", game.winner(), game.board);

    // save example for review
    if let Ok(mut file) = File::create(format!("examples/{}.ptn", sys_time())) {
        let mut turns = turns.into_iter();
        let mut turn_num = 1;
        let mut out = format!("[Size \"{N}\"]\n[Komi \"{KOMI}\"]\n");
        while let Some(white) = turns.next() {
            out.push_str(&format!(
                "{turn_num}. {white} {}\n",
                turns.next().unwrap_or_default()
            ));
            turn_num += 1;
        }
        file.write_all(out.as_bytes()).unwrap();
    };
}

pub fn sys_time() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
