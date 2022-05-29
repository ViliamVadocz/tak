use std::{
    error::Error,
    fs::write,
    io::{stdout, Write},
    sync::mpsc::channel,
    thread,
    time::{Duration, Instant},
};

use alpha_tak::{use_cuda, Net5, Net6, Network, Player};
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

    match args.board_size {
        5 => generic_main::<5, Net5>(args),
        6 => generic_main::<6, Net6>(args),
        n => println!("Unsupported board size: {n}"),
    }
}

fn generic_main<const N: usize, NET: Network<N>>(args: Args) {
    if args.ptn_file.is_some() {
        analyze_file::<N, NET>(args);
    } else if args.example_game {
        run_example_game::<N, NET>(args);
    } else {
        interactive_analysis::<N, NET>(args);
    }
}

/// Take a file and generate an analysis.
fn analyze_file<const N: usize, NET: Network<N>>(_args: Args) {
    todo!()
}

/// Run a game with the bot playing against itself
fn run_example_game<const N: usize, NET: Network<N>>(args: Args) {
    let network: NET = get_model(&args);
    let mut game = Game::<N>::with_komi(2);
    let mut player = Player::new(&network, args.batch_size, false, true, &game);

    // TODO allow custom openings
    // (and also make them work for different board sizes)
    for my_move in ["a1", "f1"] {
        let my_move = my_move.parse().unwrap();
        player.play_move(my_move, &game, false);
        game.play(my_move).unwrap();
    }

    let think_time = Duration::from_secs(args.think_seconds);

    while game.result() == GameResult::Ongoing {
        let start = Instant::now();
        while Instant::now().duration_since(start) < think_time {
            player.rollout(&game);
        }
        let my_move = player.pick_move(true);
        println!("{:.5}", player.debug(5));
        player.play_move(my_move, &game, true);
        game.play(my_move).unwrap();
    }

    save_analysis(player)
}

/// Run an interactive analysis where the user can input moves and see
/// intermediate evaluations.
fn interactive_analysis<const N: usize, NET: Network<N>>(args: Args) {
    let network: NET = get_model(&args);
    let mut game = Game::<N>::with_komi(2);
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

    save_analysis(player)
}

fn get_model<const N: usize, NET: Network<N>>(args: &Args) -> NET {
    if args.model_path == "random" {
        NET::default()
    } else {
        NET::load(&args.model_path).unwrap_or_else(|_| panic!("could not load model at {}", args.model_path))
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

fn try_play_move<const N: usize, NET: Network<N>>(
    player: &mut Player<'_, N, NET>,
    game: &mut Game<N>,
    input: String,
) -> Result<(), Box<dyn Error>> {
    let my_move = input.trim().parse()?;
    let before = game.safe_play(my_move)?;
    player.play_move(my_move, &before, true);
    Ok(())
}

fn save_analysis<const N: usize, NET: Network<N>>(mut player: Player<N, NET>) {
    write("analysis.ptn", player.get_analysis().to_string()).unwrap();
    println!("created a file `analysis.ptn` with the analysis of this game");
}
