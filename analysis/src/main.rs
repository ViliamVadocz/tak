use std::{
    fs::{read_to_string, File},
    io::{stdout, Write},
    sync::mpsc::channel,
    thread,
};

use alpha_tak::{
    analysis::Analysis,
    batch_player::BatchPlayer,
    config::{KOMI, N},
    model::network::Network,
    use_cuda,
};
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

    // TODO make nice

    let network = Network::<N>::load(&args.model_path)
        .unwrap_or_else(|_| panic!("could not load model at {}", args.model_path));

    if let Some(file_path) = args.ptn_file {
        let content = read_to_string(file_path).expect("get good scrub");
        let turns = Vec::<Turn<N>>::from_ptn(&content).expect("idk bozo");
        let analysis = analysis_for_file(&network, turns, args.batch_size);

        if let Ok(mut file) = File::create("analysis.ptn") {
            file.write_all(analysis.to_ptn().as_bytes()).unwrap();
            println!("created a file `analysis.ptn` with the analysis of this game");
        }

        return;
    }

    let mut game = Game::<N>::with_komi(KOMI);
    let mut player = BatchPlayer::new(&game, &network, vec![], game.komi, args.batch_size);

    while matches!(game.winner(), GameResult::Ongoing) {
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
                    println!("{}", player.debug(Some(5)));
                } else {
                    try_play_move(&mut player, &mut game, input).unwrap_or_else(|err| println!("{err}"));
                }
                break;
            }
        }
    }

    if let Ok(mut file) = File::create("analysis.ptn") {
        file.write_all(player.get_analysis().to_ptn().as_bytes()).unwrap();
        println!("created a file `analysis.ptn` with the analysis of this game");
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

fn try_play_move(player: &mut BatchPlayer<'_, 5>, game: &mut Game<5>, input: String) -> StrResult<()> {
    let turn = Turn::from_ptn(&input)?;
    let mut copy = game.clone();
    copy.play(turn.clone())?;
    player.play_move(game, &turn);
    game.play(turn)
}

fn analysis_for_file(network: &Network<N>, turns: Vec<Turn<N>>, batch_size: u32) -> Analysis<N> {
    let mut game = Game::with_komi(KOMI);
    let mut player = BatchPlayer::new(&game, network, vec![], game.komi, batch_size);

    for turn in turns {
        println!("Analysing {}", turn.to_ptn());
        player.rollout(&game);
        player.play_move(&game, &turn);
        game.play(turn).unwrap();
    }

    player.get_analysis()
}
