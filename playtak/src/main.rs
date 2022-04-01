use std::{
    fs::File,
    io::Write,
    str::FromStr,
    sync::mpsc::{channel, Receiver, TryRecvError},
    thread::spawn,
    time::Duration,
};

use alpha_tak::{config::KOMI, model::network::Network, player::Player, sys_time, use_cuda};
use clap::Parser;
use cli::Args;
use tak::*;
use takparse::Move;
use tokio::{
    select,
    signal::ctrl_c,
    sync::mpsc::{unbounded_channel, UnboundedSender},
    time::Instant,
};
use tokio_takconnect::{connect_as, Client, Color, GameParameters, GameUpdate, SeekParameters};

mod cli;

const WHITE_FIRST_MOVE: &str = "a1";

async fn create_seek(client: &mut Client, color: Color) {
    // Hardcoded for now
    client
        .seek(
            SeekParameters::new(
                None,
                color,
                GameParameters::new(
                    5,
                    Duration::from_secs(10 * 60),
                    Duration::from_secs(15),
                    2 * KOMI,
                    21,
                    1,
                    false,
                    false,
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if !(args.no_gpu || use_cuda()) {
        panic!("Could not enable CUDA.");
    }

    let (channel_tx, channel_rx) = channel::<(UnboundedSender<Move>, Receiver<Move>)>();

    spawn(move || {
        let network = Network::<5>::load(&args.model_path)
            .unwrap_or_else(|_| panic!("could not load model at {}", args.model_path));

        while let Ok((tx, rx)) = channel_rx.recv() {
            let mut game = Game::<5>::with_komi(KOMI);

            let mut opening = Vec::new();
            if args.seek_as_white {
                let first = Turn::from_ptn(WHITE_FIRST_MOVE).unwrap();
                opening.push(first.clone());
                game.play(first.clone()).unwrap();
            }
            let mut player = Player::<5, _>::new(&network, opening, KOMI);

            loop {
                match rx.try_recv() {
                    Ok(m) => {
                        let turn = Turn::from_ptn(&m.to_string()).unwrap();
                        player.play_move(&game, &turn);
                        game.play(turn).unwrap();

                        if game.winner() != GameResult::Ongoing {
                            println!("Opponent ended the game");
                            break;
                        }

                        println!("My turn");

                        let start = Instant::now();
                        while Instant::now().duration_since(start) < Duration::from_secs(20) {
                            player.rollout(&game, 200);
                        }

                        let turn = player.pick_move(&game, true);
                        tx.send(Move::from_str(&turn.to_ptn()).unwrap()).unwrap();
                        game.play(turn).unwrap();
                    }
                    // Ponder
                    Err(TryRecvError::Empty) => player.rollout(&game, 100),
                    // Game ended
                    Err(TryRecvError::Disconnected) => break,
                }
            }

            // create analysis file
            if let Ok(mut file) = File::create(format!("analysis_{}.ptn", sys_time())) {
                file.write_all(player.get_analysis().to_ptn().as_bytes()).unwrap();
            }
        }
    });

    // Connect to PlayTak
    let mut client = connect_as(args.username, args.password).await.unwrap();

    select! {
        _ = ctrl_c() => (),
        _ = async move {
            loop {
                create_seek(&mut client, if args.seek_as_white {Color::White} else {Color::Black}).await;
                println!("Created seek");

                let mut playtak_game = client.game().await.unwrap();
                println!("Game started");

                let (tx, mut rx) = {
                    let (outbound_tx, outbound_rx) = channel::<Move>();
                    let (inbound_tx, inbound_rx) = unbounded_channel::<Move>();
                    channel_tx.send((inbound_tx, outbound_rx)).unwrap();
                    (outbound_tx, inbound_rx)
                };

                if args.seek_as_white {
                    playtak_game.play(WHITE_FIRST_MOVE.parse().unwrap()).await.unwrap();
                }

                loop {
                    println!("Opponent's turn");
                    match playtak_game.update().await.unwrap() {
                        GameUpdate::Played(m) => {
                            println!("Opponent played {m}");

                            tx.send(m).unwrap();

                            if let Some(m) = rx.recv().await {
                                println!("Playing {m}");
                                if playtak_game.play(m).await.is_err() {
                                    println!("Failed to play move!");
                                }
                            }
                        }
                        GameUpdate::Ended(result) => {
                            println!("Game over! {result:?}");
                            break;
                        }
                        _ => {}
                    }
                }
            }
        } => (),
    }

    println!("Shutting down...");
}
