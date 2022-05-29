use std::{
    fs::write,
    thread,
    time::{Duration, Instant},
};

use alpha_tak::{sys_time, Net5, Net6, Network, Player};
use tak::*;
use tokio::sync::mpsc::{error::TryRecvError, UnboundedReceiver, UnboundedSender};

use crate::{
    cli::Args,
    message::Message,
    ANALYSIS_DIR,
    KOMI,
    OPENING_BOOK,
    PONDER_ROLLOUT_LIMIT,
    WHITE_FIRST_MOVE,
};

pub fn run_bot(args: Args, tx: UnboundedSender<Message>, mut rx: UnboundedReceiver<Message>) {
    let model_path = &args.model_path;
    let network = Net6::load(model_path).unwrap_or_else(|_| panic!("could not load model at {model_path}"));

    'game_loop: loop {
        let mut game = Game::<6>::with_komi(KOMI as i8);
        let mut player = Player::new(&network, 64, false, true, &game);
        let mut last_move: String = String::new();
        let mut ponder_rollouts = 0;

        'turn_loop: loop {
            match if ponder_rollouts < PONDER_ROLLOUT_LIMIT {
                rx.try_recv()
            } else {
                rx.blocking_recv().ok_or(TryRecvError::Disconnected)
            } {
                // Play a move.
                Ok(Message::MoveRequest) => {
                    println!("Did {ponder_rollouts} ponder rollouts.");
                    ponder_rollouts = 0;

                    println!("A move has been requested.");
                    if game.result() != GameResult::Ongoing {
                        tx.send(Message::GameEnded).unwrap();
                        continue;
                    }

                    // Check for moves that win on the spot.
                    let mut instant_win = None;
                    for my_move in game.possible_moves() {
                        let mut clone = game.clone();
                        clone.play(my_move).unwrap();
                        if matches!(clone.result(), GameResult::Winner { color, .. } if color == game.to_move)
                        {
                            instant_win = Some(my_move);
                            break;
                        }
                    }

                    let mut book = None;
                    if game.ply == 1 {
                        for opening in OPENING_BOOK {
                            if opening.0 == last_move {
                                book = Some(opening.1.parse().unwrap());
                                break;
                            }
                        }
                    }

                    // Pick turn to play.
                    let (my_move, with_info) = if game.ply == 0 {
                        (WHITE_FIRST_MOVE.parse().unwrap(), false)
                    } else if let Some(game_winning_turn) = instant_win {
                        (game_winning_turn, false)
                    } else if let Some(book_turn) = book {
                        (book_turn, false)
                    } else {
                        println!("Doing rollouts...");
                        // Do rollouts for a set amount of time.
                        let start = Instant::now();
                        while Instant::now().duration_since(start) < Duration::from_secs(args.time_to_think) {
                            player.rollout(&game);
                        }
                        print!("{:.10}", player.debug(5));

                        (player.pick_move(true), true)
                    };

                    player.play_move(my_move, &game, with_info);

                    println!("=== Network played  {my_move}");
                    tx.send(Message::Move(my_move)).unwrap();
                    game.play(my_move).unwrap();
                }

                // Opponent played a move.
                Ok(Message::Move(their_move)) => {
                    print!("{:.10}", player.debug(5));
                    println!("=== Opponent played {their_move}");

                    last_move = their_move.to_string();

                    player.play_move(their_move, &game, game.ply > 1);
                    game.play(their_move).unwrap()
                }

                // Game ended.
                Ok(Message::GameEnded) => {
                    break 'turn_loop;
                }

                // Ponder.
                Err(TryRecvError::Empty) => {
                    ponder_rollouts += 1;
                    player.rollout(&game);
                    thread::yield_now()
                }

                // Other thread ended.
                Err(TryRecvError::Disconnected) => {
                    println!("Receiver disconnected.");
                    break 'game_loop;
                }
            }
        }

        println!("Game ended, creating analysis file");
        // Create analysis file.
        write(
            format!("./{ANALYSIS_DIR}/analysis_{}.ptn", sys_time()),
            player.get_analysis().to_string(),
        )
        .unwrap_or_else(|err| println!("{err}"));
    }
}
