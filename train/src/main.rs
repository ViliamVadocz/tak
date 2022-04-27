use std::{
    fs::{create_dir_all, read_to_string},
    str::FromStr,
};

use alpha_tak::{sys_time, use_cuda, Example, Net5, Net6, Network};
use clap::Parser;
use cli::Args;
use mimalloc::MiMalloc;
use pit::pit;
use self_play::self_play;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod cli;
mod pit;
mod self_play;

const MODEL_DIR: &str = "_models";
const EXAMPLE_DIR: &str = "_examples";
const GAME_DIR: &str = "_games";

const WIN_RATE_THRESHOLD: f64 = 0.55;

fn main() {
    // Make folders if they do not exist yet.
    create_dir_all(format!("./{MODEL_DIR}/")).unwrap();
    create_dir_all(format!("./{EXAMPLE_DIR}/")).unwrap();
    create_dir_all(format!("./{GAME_DIR}/")).unwrap();

    let args = Args::parse();
    if !(args.no_gpu || use_cuda()) {
        println!("Could not enable CUDA.");
        return;
    }

    match args.board_size {
        5 => train::<5, Net5>(args),
        6 => train::<6, Net6>(args),
        x => println!("Unsupported board size {x}."),
    }
}

fn get_network<const N: usize, NET: Network<N>>(model_path: Option<String>) -> NET {
    match &model_path {
        Some(m) if m != "random" => NET::load(m).unwrap_or_else(|_| panic!("couldn't load model at {m}")),
        _ => NET::default(),
    }
}

fn train<const N: usize, NET: Network<N>>(args: Args) -> ! {
    let network = get_network::<N, NET>(args.model_path);

    let mut examples = Vec::new();
    for ex_path in args.examples {
        println!("loading {ex_path}");
        examples.extend(
            read_to_string(ex_path)
                .unwrap()
                .split_terminator('\n')
                .map(Example::from_str)
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
                .into_iter(),
        );
    }

    training_loop(network, examples)
}

fn training_loop<const N: usize, NET: Network<N>>(mut network: NET, mut examples: Vec<Example<N>>) -> ! {
    loop {
        if !examples.is_empty() {
            // Train on examples.
            let new_network = {
                let mut nn = copy(&network);
                nn.train(&examples);
                nn
            };

            // Run pit games.
            println!("pitting two networks against each other");
            let results = pit(&new_network, &network);
            println!("{results:?}");

            // Save new network if it is better.
            if results.win_rate() > WIN_RATE_THRESHOLD {
                network = new_network;
                println!("saving model");
                network.save(format!("{MODEL_DIR}/{}.model", sys_time())).unwrap();

                // Clear examples after
                examples.clear();
            }
        }

        // Do self-play to get new examples.
        println!("starting self-play");
        let new_examples = self_play(&network);
        examples.extend(new_examples.into_iter())
    }
}

fn copy<const N: usize, NET: Network<N>>(network: &NET) -> NET {
    // copy network values by file (ugly but works)
    let mut dir = std::env::temp_dir();
    dir.push("model");
    network.save(&dir).unwrap();
    Network::<N>::load(&dir).unwrap()
}
