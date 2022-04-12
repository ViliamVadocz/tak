use std::{
    fs::write,
    thread,
    time::{Duration, Instant},
};

use alpha_tak::{batch_player::BatchPlayer, config::KOMI, model::network::Network, sys_time};
use tak::*;
use tokio::sync::mpsc::{error::TryRecvError, UnboundedReceiver, UnboundedSender};

use crate::{message::Message, OPENING_BOOK, PONDER_ROLLOUT_LIMIT, THINK_SECONDS, WHITE_FIRST_MOVE};

pub fn run_bot(model_path: &str, tx: UnboundedSender<Message>, mut rx: UnboundedReceiver<Message>) {
    let network =
        Network::<5>::load(model_path).unwrap_or_else(|_| panic!("could not load model at {model_path}"));

    'game_loop: loop {
        let mut game = Game::<5>::with_komi(KOMI);
        let mut player = BatchPlayer::new(&game, &network, vec![], game.komi, 64);
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

                    let mut book = None;
                    if game.ply == 1 {
                        for opening in OPENING_BOOK {
                            if opening.0 == last_move {
                                book = Some(Turn::from_ptn(opening.1).unwrap());
                                break;
                            }
                        }
                    }

                    // Pick turn to play.
                    let turn = if game.ply == 0 {
                        let first = Turn::from_ptn(WHITE_FIRST_MOVE).unwrap();
                        player.play_move(&game, &first);
                        first
                    } else if let Some(game_winning_turn) = insta_win {
                        player.play_move(&game, &game_winning_turn);
                        game_winning_turn
                    } else if let Some(book_turn) = book {
                        player.play_move(&game, &book_turn);
                        book_turn
                    } else {
                        println!("Doing rollouts...");
                        // Do rollouts for a set amount of time.
                        let start = Instant::now();
                        while Instant::now().duration_since(start) < Duration::from_secs(THINK_SECONDS) {
                            player.rollout(&game);
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
                    last_move = s;

                    player.play_move(&game, &turn);
                    game.play(turn).unwrap()
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
                Err(TryRecvError::Disconnected) => break 'game_loop,
            }
        }

        println!("Game ended, creating analysis file");
        // Create analysis file.
        write(
            format!("analysis_{}.ptn", sys_time()),
            player.get_analysis().to_ptn(),
        )
        .unwrap_or_else(|err| println!("{err}"));
    }
}
