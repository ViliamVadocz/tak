use alpha_tak::{play, train, use_cuda};
use clap::{Parser, Subcommand};
use tak::*;

/// AlphaTak Command Line Interface
#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
    /// Disable GPU usage
    #[clap(short, long)]
    no_gpu: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Train the network
    Train {
        /// Path to model, use "random" or leave blank if you want a new model
        model_path: Option<String>,
        /// Paths to example files
        examples: Vec<String>,
    },
    /// Play against the bot
    Play {
        /// Path to model
        model_path: String,
        #[clap(short, long, default_value_t = Colour::White)]
        /// Colour to play as against the bot
        colour: Colour,
        /// Number of seconds per move for the network
        #[clap(short, long, default_value_t = 10)]
        seconds_per_move: u64,
    },
}

fn main() {
    let args = Args::parse();
    if !(args.no_gpu || use_cuda()) {
        println!("Could not enable CUDA.");
        return;
    }

    match args.command {
        Command::Train { model_path, examples } => train(model_path, examples),
        Command::Play {
            model_path,
            colour,
            seconds_per_move,
        } => play(model_path, colour, seconds_per_move),
    };
}
