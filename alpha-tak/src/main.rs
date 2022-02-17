#![feature(thread_is_running)]

use std::{
    env::Args,
    fs::File,
    io::Write,
    sync::mpsc::channel,
    thread,
    time::{Duration, SystemTime},
};

use example::{save_examples, Example};
use network::Network;
use rand::random;
use tak::{
    colour::Colour,
    game::{Game, GameResult},
    pos::Pos,
    ptn::{FromPTN, ToPTN},
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
    match args.next() {
        Some(s) if s == "play" => play(args),
        Some(s) if s == "train" => train(args),
        _ => println!("usage: alpha-tak (play|train) <model_path> [<example_path>*]"),
    }
}

fn play(mut args: Args) {
    const SECONDS_PER_MOVE: u64 = 10;

    // load or create network
    let model_path = args.next().expect("you need to supply a model path");
    let network =
        Network::<5>::load(&model_path).unwrap_or_else(|_| panic!("couldn't load model at {model_path}"));

    let mut game = Game::<5>::with_komi(KOMI);
    let colour = if random() { Colour::White } else { Colour::Black };
    println!("the network is playing as {colour:?}");

    let mut debug_info = String::new();

    let mut node = Node::default();
    while matches!(game.winner(), GameResult::Ongoing) {
        if game.to_move == colour {
            // do rollouts
            let start_turn = SystemTime::now();
            while SystemTime::now().duration_since(start_turn).unwrap().as_secs() < SECONDS_PER_MOVE {
                for _ in 0..100 {
                    node.rollout(game.clone(), &network);
                }
            }
            debug_info += &node.debug(&game, None);
            debug_info.push('\n');

            let turn = node.pick_move(game.ply > 3);
            println!("network plays: {}", turn.to_ptn());
            node = node.play(&turn);
            game.play(turn).unwrap();
        } else {
            // create a thread to get input from user
            let (tx, rx) = channel();
            thread::spawn(move || {
                let turn = loop {
                    print!("your move: ");
                    std::io::stdout().flush().unwrap();
                    let mut line = String::new();
                    let _ = std::io::stdin().read_line(&mut line).unwrap();
                    match Turn::from_ptn(&line) {
                        Ok(turn) => break turn,
                        Err(err) => println!("{err}"),
                    }
                };
                tx.send(turn).unwrap();
            });
            // think on opponent's turn
            let turn = loop {
                match rx.try_recv() {
                    Ok(t) => break t,
                    Err(_) => {
                        for _ in 0..100 {
                            node.rollout(game.clone(), &network);
                        }
                    }
                }
            };
            // try playing your move
            let backup = game.clone();
            match game.play(turn.clone()) {
                Ok(_) => {
                    debug_info += &node.debug(&backup, None);
                    debug_info.push('\n');
                    node = node.play(&turn);
                }
                Err(err) => {
                    println!("{err}");
                    game = backup;
                }
            }
        }
    }

    println!("game ended! result: {:?}", game.winner());
    thread::sleep(Duration::from_secs(3));
    println!("{debug_info}");
}

fn train(mut args: Args) {
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
        examples.extend(
            load_examples(&examples_path)
                .unwrap_or_else(|_| panic!("could not load example at {examples_path}"))
                .into_iter(),
        );
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
