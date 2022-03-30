use std::time::Duration;

use alpha_tak::{config::KOMI, model::network::Network, player::Player, use_cuda};
use clap::Parser;
use cli::Args;
use tak::*;
use tokio::{select, time::Instant};
use tokio_takconnect::{connect_guest, Client, Color, GameParameters, GameUpdate, SeekParameters};

mod cli;

fn setup() -> Network<5> {
    let args = Args::parse();
    if !(args.no_gpu || use_cuda()) {
        panic!("Could not enable CUDA.");
    }

    Network::<5>::load(&args.model_path)
        .unwrap_or_else(|_| panic!("could not load model at {}", args.model_path))
}

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
                    true,
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
    let network = setup();

    let mut client = connect_guest().await.unwrap();

    loop {
        create_seek(&mut client).await;
        println!("Created seek");

        let mut playtak_game = client.game().await.unwrap();
        println!("Game started");

        let mut game = Game::<5>::with_komi(KOMI);
        let mut player = Player::<5, _>::new(&network, vec![], KOMI);

        'game_loop: loop {
            println!("Opponent's turn");
            'opponent_turn: loop {
                select! {
                    update = playtak_game.update() => {
                        println!("Game update received");
                        match update.unwrap() {
                            GameUpdate::Played(m) => {
                                println!("Opponent played {m}");
                                let turn = Turn::from_ptn(&m.to_string()).unwrap();
                                player.play_move(&game, &turn);
                                game.play(turn).unwrap();
                                break 'opponent_turn;
                            },
                            GameUpdate::Ended(result) => {
                                println!("Game over! {result:?}");
                                break 'game_loop;
                            }
                            _ => (),
                        }
                    },

                    // pondering
                    _ = async {player.rollout(&game, 100)} => {}
                }
            }

            // my turn
            println!("My turn");
            let start = Instant::now();
            while Instant::now().duration_since(start) < Duration::from_secs(20) {
                player.rollout(&game, 200);
            }
            let turn = player.pick_move(&game, true);
            println!("Playing {}", turn.to_ptn());
            playtak_game.play(turn.to_ptn().parse().unwrap()).await.unwrap();
            game.play(turn).unwrap();
        }
    }
}
