use std::{
    error::Error,
    fs::File,
    io::Write,
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
    thread::spawn,
    time::Duration,
};

use alpha_tak::{config::KOMI, model::network::Network, player::Player, sys_time, use_cuda};
use clap::Parser;
use cli::Args;
use tak::{GameResult, *};
use tokio::{select, signal::ctrl_c, time::Instant};
use tokio_takconnect::{
    connect_as,
    connect_guest,
    Client,
    Color,
    GameParameters,
    GameUpdate,
    SeekParameters,
};

mod cli;

const WHITE_FIRST_MOVE: &str = "e5";
const THINK_SECONDS: u64 = 15;

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
                    Duration::from_secs(10),
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
        .unwrap();
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if !(args.no_gpu || use_cuda()) {
        panic!("could not enable CUDA");
    }

    let (net_tx, playtak_rx) = channel();
    let (playtak_tx, net_rx) = channel();

    let model_path = args.model_path.clone();
    spawn(move || run_bot(&model_path, net_tx, net_rx));
    playtak(args, playtak_tx, playtak_rx).await.unwrap();
}

fn run_bot(model_path: &str, tx: Sender<Message>, rx: Receiver<Message>) {
    let network =
        Network::<5>::load(model_path).unwrap_or_else(|_| panic!("could not load model at {model_path}"));

    'game_loop: loop {
        let mut game = Game::<5>::with_komi(KOMI);
        let mut player = Player::new(&network, vec![], KOMI);

        'turn_loop: loop {
            match rx.try_recv() {
                // Play a move.
                Ok(Message::MoveRequest) => {
                    if game.winner() != GameResult::Ongoing {
                        tx.send(Message::GameEnded).unwrap();
                        continue;
                    }

                    // Check for moves that win on the spot.
                    let mut insta_win = None;
                    for turn in game.possible_turns() {
                        let mut clone = game.clone();
                        clone.play(turn.clone()).unwrap();
                        if matches!(clone.winner(), GameResult::Winner { colour, .. } if colour == game.to_move)
                        {
                            insta_win = Some(turn);
                            break;
                        }
                    }

                    // Pick turn to play.
                    let turn = if let Some(game_winning_turn) = insta_win {
                        game_winning_turn
                    } else if game.ply == 0 {
                        // Hardcoded opening.
                        let first = Turn::from_ptn(WHITE_FIRST_MOVE).unwrap();
                        player.play_move(&game, &first);
                        first
                    } else {
                        // Apply noise to hopefully prevent farming.
                        if game.ply < 16 {
                            println!("Applying noise!");
                            player.apply_dirichlet(&game, 1.0, 0.4);
                        }
                        println!("Doing rollouts...");
                        // Do rollouts for a set amount of time.
                        let start = Instant::now();
                        while Instant::now().duration_since(start) < Duration::from_secs(THINK_SECONDS) {
                            player.rollout(&game, 500);
                        }
                        print!("{}", player.debug(Some(5)));

                        player.pick_move(&game, true)
                    };

                    println!("=== Network played  {}", turn.to_ptn());
                    tx.send(Message::Turn(turn.to_ptn())).unwrap();
                    game.play(turn).unwrap();
                }

                // Opponent played a move.
                Ok(Message::Turn(s)) => {
                    print!("{}", player.debug(Some(5)));
                    println!("=== Opponent played {s}");

                    let turn = Turn::from_ptn(&s).unwrap();
                    player.play_move(&game, &turn);
                    game.play(turn).unwrap()
                }

                // Game ended.
                Ok(Message::GameEnded) => {
                    break 'turn_loop;
                }

                // Ponder.
                Err(TryRecvError::Empty) => player.rollout(&game, 100),

                // Other thread ended.
                Err(TryRecvError::Disconnected) => break 'game_loop,
            }
        }

        println!("Game ended, creating analysis file");
        // Create analysis file.
        if let Ok(mut file) = File::create(format!("analysis_{}.ptn", sys_time())) {
            file.write_all(player.get_analysis().to_ptn().as_bytes())
                .unwrap_or_else(|err| println!("{err}"));
        }
    }
}

enum Message {
    MoveRequest,
    Turn(String),
    GameEnded,
}

async fn playtak(args: Args, tx: Sender<Message>, rx: Receiver<Message>) -> Result<(), Box<dyn Error>> {
    // Connect to PlayTak
    let mut client = if let (Some(username), Some(password)) = (args.username, args.password) {
        println!("Connecting as {username}");
        connect_as(username, password).await
    } else {
        println!("Connecting as guest");
        connect_guest().await
    }?;

    let mut seek_as_white = false;
    select! {
        _ = ctrl_c() => (),
        _ = async move {
            loop {
                create_seek(&mut client, if seek_as_white {Color::White} else {Color::Black}).await;
                println!("Created seek (white: {seek_as_white})");

                if run_playtak_game(&mut client, &tx, &rx, seek_as_white).await.is_err() {
                    break;
                }

                // Alternate seek colours.
                seek_as_white = !seek_as_white;
            }
        } => (),
    }

    println!("Shutting down...");
    Ok(())
}

async fn run_playtak_game(
    client: &mut Client,
    tx: &Sender<Message>,
    rx: &Receiver<Message>,
    seek_as_white: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut game = client.game().await?;
    println!("Game started!");

    let mut take_my_turn = seek_as_white;
    loop {
        if take_my_turn {
            tx.send(Message::MoveRequest)?;
            match rx.recv()? {
                Message::Turn(m) => {
                    if game.play(m.parse()?).await.is_err() {
                        println!("Failed to play move!");
                    }
                }
                Message::GameEnded => {}
                _ => {}
            }
        }

        match game.update().await? {
            GameUpdate::Played(m) => {
                tx.send(Message::Turn(m.to_string()))?;
            }
            GameUpdate::Ended(_result) => {
                tx.send(Message::GameEnded)?;
                break Ok(());
            }
            _ => {}
        }

        take_my_turn = true;
    }
}
