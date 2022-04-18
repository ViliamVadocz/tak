use std::thread::spawn;

use alpha_tak::use_cuda;
use clap::Parser;
use log::LevelFilter;
use simple_logging::log_to_file;
use tokio::sync::mpsc::unbounded_channel;

use crate::{bot::run_bot, cli::Args, playtak::seek_loop};

mod bot;
mod cli;
mod message;
mod playtak;
mod seek;

const WHITE_FIRST_MOVE: &str = "e5";
const OPENING_BOOK: [(&str, &str); 4] = [("a1", "e5"), ("a5", "e1"), ("e1", "a5"), ("e5", "a1")];

const PONDER_ROLLOUT_LIMIT: u64 = 10_000;

#[tokio::main]
async fn main() {
    log_to_file("playtak.log", LevelFilter::Debug).unwrap();

    let args = Args::parse();
    if !(args.no_gpu || use_cuda()) {
        panic!("could not enable CUDA");
    }

    let (net_tx, playtak_rx) = unbounded_channel();
    let (playtak_tx, net_rx) = unbounded_channel();

    let args_clone = args.clone();
    spawn(move || run_bot(args_clone, net_tx, net_rx));
    seek_loop(args, playtak_tx, playtak_rx).await.unwrap();
}