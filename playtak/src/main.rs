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
use tokio::{select, signal::ctrl_c, sync::mpsc::unbounded_channel, time::Instant};
use tokio_takconnect::{connect_as, Client, Color, GameParameters, GameUpdate, SeekParameters};

mod cli;

async fn create_seek(client: &mut Client) {
    // Hardcoded for now
    client
        .seek(
            SeekParameters::new(
                None,
                Color::Black,
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

    let (tx, mut rx) = {
        let (main_tx, channel_rx) = channel::<Receiver<Move>>();
        let (tx, main_rx) = unbounded_channel::<Move>();

        spawn(move || {
            let network = Network::<5>::load(&args.model_path)
                .unwrap_or_else(|_| panic!("could not load model at {}", args.model_path));

            while let Ok(rx) = channel_rx.recv() {
                let mut game = Game::<5>::with_komi(KOMI);
                let mut player = Player::<5, _>::new(&network, vec![], KOMI);

                loop {
                    match rx.try_recv() {
                        Ok(m) => {
                            println!("My turn");

                            let turn = Turn::from_ptn(&m.to_string()).unwrap();
                            player.play_move(&game, &turn);
                            game.play(turn).unwrap();

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

        (main_tx, main_rx)
    };

    // Connect to PlayTak
    let mut client = connect_as(args.username, args.password).await.unwrap();

    select! {
        _ = ctrl_c() => (),
        _ = async move {
            loop {
                create_seek(&mut client).await;
                println!("Created seek");

                let mut playtak_game = client.game().await.unwrap();
                println!("Game started");

                let tx = {
                    let (game_tx, game_rx) = channel::<Move>();
                    tx.send(game_rx).unwrap();
                    game_tx
                };

                loop {
                    println!("Opponent's turn");
                    match playtak_game.update().await.unwrap() {
                        GameUpdate::Played(m) => {
                            println!("Opponent played {m}");

                            tx.send(m).unwrap();

                            let m = rx.recv().await.unwrap();

                            println!("Playing {m}");
                            if playtak_game.play(m).await.is_err() {
                                println!("Failed to play move!");
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
