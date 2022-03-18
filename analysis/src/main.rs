use std::{
    io::{stdout, Write},
    sync::mpsc::channel,
    thread,
};

use alpha_tak::{model::network::Network, player::Player, use_cuda};
use clap::Parser;
use cli::Args;
use tak::*;

mod cli;

fn main() {
    let args = Args::parse();
    if !(args.no_gpu || use_cuda()) {
        println!("Could not enable CUDA.");
        return;
    }

    let network = Network::<5>::load(&args.model_path)
        .unwrap_or_else(|_| panic!("could not load model at {}", args.model_path));

    let mut player = Player::new(&network, vec![]);
    let mut game = Game::<5>::with_komi(2);

    while matches!(game.winner(), GameResult::Ongoing) {
        // Get input from user.
        let (tx, rx) = channel();
        thread::spawn(move || {
            tx.send(get_input()).unwrap();
        });

        loop {
            // Do rollouts while we wait for input.
            player.rollout(&game, 100);

            if let Ok(input) = rx.try_recv() {
                clear_screen();
                if input.chars().all(char::is_whitespace) {
                    println!("{}", player.debug(Some(5)));
                } else {
                    try_play_move(&mut player, &mut game, input).unwrap_or_else(|err| println!("{err}"));
                }
                break;
            }
        }
    }
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

fn try_play_move(player: &mut Player<'_, 5, Network<5>>, game: &mut Game<5>, input: String) -> StrResult<()> {
    let turn = Turn::from_ptn(&input)?;
    let mut copy = game.clone();
    copy.play(turn.clone())?;
    player.play_move(game, &turn);
    game.play(turn)
}
