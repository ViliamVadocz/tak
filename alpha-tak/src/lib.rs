#![feature(test)]
#![feature(thread_is_running)]

extern crate test;

use std::{
    fs::File,
    io::Write,
    sync::mpsc::channel,
    thread,
    time::{Duration, SystemTime},
};

use search::turn_map::Lut;
use tak::*;
use tch::{Cuda, Device};

use crate::{
    config::{KOMI, N},
    model::network::Network,
    player::Player,
    search::node::Node,
};

#[macro_use]
extern crate lazy_static;

pub mod model;
pub mod search;

pub mod analysis;
pub mod config;
pub mod threadpool;

pub mod agent;
pub mod example;
pub mod player;
pub mod repr;

lazy_static! {
    static ref DEVICE: Device = Device::cuda_if_available();
}

/// Try initializing CUDA
/// Returns whether CUDA is available
pub fn use_cuda() -> bool {
    tch::maybe_init_cuda();
    Cuda::is_available()
}

pub fn play(model_path: String, colour: Colour, seconds_per_move: u64) {
    // load or create network
    let network =
        Network::<N>::load(&model_path).unwrap_or_else(|_| panic!("couldn't load model at {model_path}"));

    let mut game = Game::<N>::with_komi(KOMI);
    let net_colour = colour.next();

    let mut debug_info = String::new();

    let mut node = Node::default();
    while matches!(game.winner(), GameResult::Ongoing) {
        if game.to_move == net_colour {
            // do rollouts
            let start_turn = SystemTime::now();
            while SystemTime::now().duration_since(start_turn).unwrap().as_secs() < seconds_per_move {
                for _ in 0..100 {
                    node.rollout(game.clone(), &network);
                }
            }
            debug_info += &format!(
                "move: {}, to move: {:?},  ply: {}\n{}",
                game.ply / 2 + 1,
                game.to_move,
                game.ply,
                node.debug(None)
            );
            debug_info += &node.debug(None);
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
                    debug_info += &format!(
                        "move: {}, to move: {:?},  ply: {}\n{}",
                        backup.ply / 2 + 1,
                        backup.to_move,
                        backup.ply,
                        node.debug(None)
                    );
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

// TODO use example game as training data
fn example_game<const N: usize>(network: &Network<N>)
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    const SECONDS_PER_TURN: u64 = 10;
    println!("running example game with {SECONDS_PER_TURN} seconds per turn");

    let mut game = Game::with_komi(KOMI);
    // opening
    let turn0 = Turn::from_ptn("a1").unwrap();
    let turn1 = Turn::from_ptn(if rand::random::<f64>() < 0.5 { "e1" } else { "e5" }).unwrap();
    game.play(turn0.clone()).unwrap();
    game.play(turn1.clone()).unwrap();
    let mut player = Player::new(network, vec![turn0, turn1]);

    while matches!(game.winner(), GameResult::Ongoing) {
        // do rollouts
        let start_turn = SystemTime::now();
        while SystemTime::now().duration_since(start_turn).unwrap().as_secs() < SECONDS_PER_TURN {
            player.rollout(&game, 100);
        }
        let turn = player.pick_move(&game, true);
        game.play(turn).unwrap();
    }

    // save analysis for review
    if let Ok(mut file) = File::create(format!("examples/{}.ptn", sys_time())) {
        file.write_all(player.get_analysis().to_ptn().as_bytes()).unwrap();
    };
}

pub fn sys_time() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
