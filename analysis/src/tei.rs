use std::{io, process, time};

use alpha_tak::{
    config::{KOMI, N},
    model::network::Network,
    player::Player,
    use_cuda,
};
use clap::Parser;
use cli::Args;
use tak::{FromPTN, Game, ToPTN, Turn};

mod cli;

fn main() -> io::Result<()> {
    let args: Args = Args::parse();
    if !(args.no_gpu || use_cuda()) {
        println!("Could not enable CUDA.");
        return Ok(());
    }

    // TODO make nice

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

    println!("id name WilemBot");
    println!("id author Viliam Vadocz");
    println!("option name HalfKomi type spin default 4 min 4 max 4");
    println!("teiok");

    let mut game: Game<N> = Game::default();

    loop {
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

        match input_words[0] {
            "teinewgame" => {
                if input_words.get(1) != Some(&"5") {
                    eprintln!("Unsupported size");
                    process::exit(1)
                } else {
                    game = Game::default();
                }
            }
            "setoption" => {
                if input_words.get(1..=4) != Some(&["name", "HalfKomi", "value", "4"]) {
                    eprintln!("Unsupported option string {}", input);
                    process::exit(1)
                }
            }
            "position" => {
                if input_words.get(1..) == Some(&["startpos"]) {
                    game = Game::default();
                } else if input_words.get(1..=2) == Some(&["startpos", "moves"]) {
                    game = Game::default();
                    for turn in input_words.iter().skip(3) {
                        if let Ok(turn) = Turn::from_ptn(turn) {
                            if game.possible_turns().contains(&turn) {
                                game.play(turn).unwrap();
                            } else {
                                eprintln!("Illegal move {:?}", turn);
                                process::exit(1)
                            }
                        } else {
                            eprintln!("Couldn't parse move {}", turn);
                            process::exit(1)
                        }
                    }
                } else if input_words.get(1) == Some(&"tps") {
                    eprintln!("tps positions strings are not supported");
                    process::exit(1)
                } else {
                    eprintln!("Unexpected position string {}", input);
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
            _ => {}
        }
    }
}

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
