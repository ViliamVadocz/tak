mod cli;
mod pit;
mod self_play;
mod training_loop;

use std::fs::create_dir;

use alpha_tak::{
    config::N,
    example::{load_examples, save_examples},
    model::network::Network,
    sys_time,
    use_cuda,
};
use clap::Parser;
use cli::Args;
use self_play::self_play;
use training_loop::training_loop;

const MODEL_DIR: &str = "models";
const EXAMPLE_DIR: &str = "examples";
const GAME_DIR: &str = "games";

fn main() {
    let args = Args::parse();
    if !(args.no_gpu || use_cuda()) {
        println!("Could not enable CUDA.");
        return;
    }

    // Make folders if they do not exist yet
    create_dir(format!("./{MODEL_DIR}/")).unwrap();
    create_dir(format!("./{EXAMPLE_DIR}/")).unwrap();
    create_dir(format!("./{GAME_DIR}/")).unwrap();

    if args.only_self_play {
        only_self_play(args.model_path)
    } else {
        train(args.model_path, args.examples)
    }
}

/// Load or create a network
fn get_network(model_path: Option<String>) -> Network<N> {
    match &model_path {
        Some(m) if m != "random" => {
            Network::<N>::load(m).unwrap_or_else(|_| panic!("couldn't load model at {m}"))
        }
        _ => {
            println!("generating random model");
            Network::<N>::default()
        }
    }
}

fn only_self_play(model_path: Option<String>) {
    let network = get_network(model_path);
    loop {
        let examples = self_play(&network);
        save_examples(&examples, format!("{EXAMPLE_DIR}/{}.data", sys_time()));
    }
}

fn train(model_path: Option<String>, example_paths: Vec<String>) {
    let network = get_network(model_path);

    // optionally load examples
    let mut examples = Vec::new();
    for ex_path in example_paths {
        println!("loading {ex_path}");
        examples.extend(
            load_examples(&ex_path)
                .unwrap_or_else(|_| panic!("could not load example at {ex_path}"))
                .into_iter(),
        );
    }

    // begin training loop
    training_loop(network, examples)
}
