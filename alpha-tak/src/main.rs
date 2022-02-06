#![feature(thread_is_running)]

use std::{fs::File, io::Write, time::SystemTime};

use example::Example;
use network::Network;
use tak::{
    game::{Game, GameResult},
    pos::Pos,
    ptn::ToPTN,
    tile::{Shape, Tile},
    turn::Turn,
};
use tch::Cuda;
use turn_map::Lut;

use crate::{mcts::Node, pit::pit_async, self_play::self_play_async};

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

        example_game(&nn);
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

fn example_game<const N: usize>(network: &Network<N>)
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    const SECONDS_PER_TURN: u64 = 10;
    println!("running example game with {SECONDS_PER_TURN} seconds per turn");

    let mut game = Game::default();
    let mut turns = Vec::new();
    // opening
    let turn0 = Turn::Place {
        pos: Pos { x: 0, y: 0 },
        shape: Shape::Flat,
    };
    let turn1 = Turn::Place {
        pos: Pos { x: N - 1, y: 0 },
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
    let sys_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if let Ok(mut file) = File::create(format!("examples/{}.ptn", sys_time)) {
        let mut turns = turns.into_iter();
        let mut turn_num = 1;
        let mut out = String::new();
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
