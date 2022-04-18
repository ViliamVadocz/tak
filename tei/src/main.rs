use std::{io, process, time};

use alpha_tak::{
    config::{KOMI, N},
    model::network::Network,
    player::Player,
    use_cuda,
};
use clap::Parser;
use cli::Args;
use mimalloc::MiMalloc;
use tak::{FromPTN, Game, ToPTN, Turn};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod cli;

fn main() -> io::Result<()> {
    let args: Args = Args::parse();
    if !(args.no_gpu || use_cuda()) {
        println!("Could not enable CUDA.");
        return Ok(());
    }

    let network = Network::<N>::load(&args.model_path)
        .unwrap_or_else(|_| panic!("could not load model at {}", args.model_path));

    {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim() != "tei" {
            eprintln!("Unexpected input {}", input);
            process::exit(1)
        }
    }

    // Register bot with TEI
    // Read more here: https://github.com/MortenLohne/racetrack#tei
    println!("id name WilemBot");
    println!("id author Viliam Vadocz");
    println!("option name HalfKomi type spin default 4 min 4 max 4");
    println!("teiok");

    let mut game = Game::with_komi(KOMI);

    loop {
        // Wait for input line.
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input_words = input.split_whitespace().collect::<Vec<_>>();

        if input.is_empty() {
            eprintln!("Got EOF from GUI");
            process::exit(1)
        }

        if input_words.is_empty() {
            continue;
        }

        // TODO Put this whole thing in a function that returns a result
        // so that you can then handle all the errors in one swoop
        match input_words[0] {
            "teinewgame" => {
                // Start new game.
                if input_words.get(1) != Some(&"5") {
                    eprintln!("Unsupported size");
                    process::exit(1)
                } else {
                    game = Game::with_komi(KOMI);
                }
            }
            "setoption" => {
                // Set TEI options.
                if input_words.get(1..=4) != Some(&["name", "HalfKomi", "value", "4"]) {
                    eprintln!("Unsupported option string {input}");
                    process::exit(1)
                }
            }
            "position" => {
                // Set the game position.
                if input_words.get(1..) == Some(&["startpos"]) {
                    game = Game::with_komi(KOMI);
                } else if input_words.get(1..=2) == Some(&["startpos", "moves"]) {
                    game = Game::with_komi(KOMI);
                    for ptn_turn in input_words.iter().skip(3) {
                        if let Ok(turn) = Turn::from_ptn(ptn_turn) {
                            if !game.play(turn).is_ok() {
                                eprintln!("Illegal move {ptn_turn:?}");
                                process::exit(1)
                            }
                        } else {
                            eprintln!("Couldn't parse move {ptn_turn}");
                            process::exit(1)
                        }
                    }
                } else if input_words.get(1) == Some(&"tps") {
                    if let Some(tps) = input_words.get(2) {
                        game = Game::from_ptn(&format!("[Komi \"2\"]\n[TPS \"{tps}\"]\n")).unwrap_or_else(
                            |err| {
                                eprintln!("{err}");
                                process::exit(1)
                            },
                        );
                    } else {
                        eprintln!("Expected TPS string {input}");
                        process::exit(1)
                    }
                } else {
                    eprintln!("Unexpected position string {input}");
                    process::exit(1)
                }
            }
            "go" => {
                if input_words.get(1) == Some(&"infinite") {
                    calculate_move_time(&network, game.clone(), time::Duration::MAX, time::Duration::ZERO)
                } else if input_words.get(1) == Some(&"wtime") {
                    let index_offset = match game.colour() {
                        tak::Colour::White => 0,
                        tak::Colour::Black => 2,
                    };

                    // TODO: This parsing is very brittle, and assumes the GUI
                    // will only send strings like
                    // go wtime 10000 btime 10000 winc 100 binc 100, even though other
                    // strings may be sent

                    let our_time = input_words
                        .get(2 + index_offset)
                        .and_then(|word| word.parse().ok())
                        .map(time::Duration::from_millis);

                    let our_increment = input_words
                        .get(6 + index_offset)
                        .unwrap_or(&"0")
                        .parse()
                        .ok()
                        .map(time::Duration::from_millis);

                    // TODO: Re-use the tree from last move's search
                    if let Some((time, inc)) = our_time.zip(our_increment) {
                        calculate_move_time(&network, game.clone(), time, inc)
                    } else {
                        eprintln!("Couldn't parse go time string {}", input);
                        process::exit(1)
                    }
                } else {
                    eprintln!("Couldn't parse go input {}", input);
                    process::exit(1)
                }
            }
            "isready" => {
                println!("readyok")
            }
            "quit" => process::exit(0),
            _ => {}
        }
    }
}

// TODO Rewrite to use BatchPlayer, keep player around between moves, don't
// search by amount of nodes, but use time...
fn calculate_move_time(network: &Network<N>, game: Game<N>, time: time::Duration, increment: time::Duration) {
    let mut player: Player<N, _> = Player::new(network, vec![], KOMI);

    let start_time = time::Instant::now();
    let mut total_nodes = 0;

    for i in 0.. {
        let nodes_this_iteration = (10.0 * 1.42_f32.powi(i)) as usize;
        player.rollout(&game, nodes_this_iteration);
        total_nodes += nodes_this_iteration;

        // TODO: Extract score and pv from the search

        if start_time.elapsed() > time / 10 + increment / 2 {
            let turn = player.pick_move(&game, true);
            println!(
                "info score cp 0 depth {} nodes {} time {} nps {:.0} pv {}",
                i,
                total_nodes,
                start_time.elapsed().as_millis(),
                total_nodes as f32 / start_time.elapsed().as_secs_f32(),
                turn.to_ptn()
            );
            println!("bestmove {}", turn.to_ptn());
            break;
        } else {
            println!(
                "info score cp 0 depth {} nodes {} time {} nps {:.0}",
                i,
                total_nodes,
                start_time.elapsed().as_millis(),
                total_nodes as f32 / start_time.elapsed().as_secs_f32()
            );
        }
    }
}
