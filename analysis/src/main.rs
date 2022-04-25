use std::{
    error::Error,
    fs::write,
    io::{stdout, Write},
    sync::mpsc::channel,
    thread,
};

use alpha_tak::{use_cuda, Net5, Network, Player};
use clap::Parser;
use cli::Args;
use mimalloc::MiMalloc;
use tak::*;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod cli;

fn main() {
    let args = Args::parse();
    if !(args.no_gpu || use_cuda()) {
        println!("Could not enable CUDA.");
        return;
    }

    let network = Net5::load(&args.model_path)
        .unwrap_or_else(|_| panic!("could not load model at {}", args.model_path));

    let mut game = Game::<5>::with_komi(2);
    let mut player = Player::new(&network, args.batch_size, false, true, &game);

    while matches!(game.result(), GameResult::Ongoing) {
        // Get input from user.
        let (tx, rx) = channel();
        thread::spawn(move || {
            tx.send(get_input()).unwrap();
        });

        loop {
            // Do rollouts while we wait for input.
            player.rollout(&game);

            if let Ok(input) = rx.try_recv() {
                clear_screen();
                if input.chars().all(char::is_whitespace) {
                    println!("{:.10}", player.debug(3));
                } else {
                    try_play_move(&mut player, &mut game, input).unwrap_or_else(|err| println!("{err}"));
                }
                break;
            }
        }
    }

    write("analysis.ptn", player.get_analysis().to_string()).unwrap();
    println!("created a file `analysis.ptn` with the analysis of this game");
}

fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    stdout().flush().unwrap()
}

fn get_input() -> String {
    print!("[leave empty for network eval] your move: ");
    std::io::stdout().flush().unwrap();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    line
}

fn try_play_move(
    player: &mut Player<'_, 5, Net5>,
    game: &mut Game<5>,
    input: String,
) -> Result<(), Box<dyn Error>> {
    let my_move = input.trim().parse()?;
    let before = game.safe_play(my_move)?;
    player.play_move(&before, my_move);
    Ok(())
}
