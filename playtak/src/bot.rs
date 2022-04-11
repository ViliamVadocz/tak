use std::{sync::mpsc::{Receiver, Sender, TryRecvError}, time::{Instant, Duration}, fs::write};

use alpha_tak::{config::KOMI, model::network::Network, player::Player, sys_time};
use tak::*;

use crate::{message::Message, WHITE_FIRST_MOVE, THINK_SECONDS, OPENING_BOOK};

pub fn run_bot(model_path: &str, tx: Sender<Message>, rx: Receiver<Message>) {
    let network =
        Network::<5>::load(model_path).unwrap_or_else(|_| panic!("could not load model at {model_path}"));

    'game_loop: loop {
        let mut game = Game::<5>::with_komi(KOMI);
        let mut player = Player::new(&network, vec![], KOMI);
        let mut last_move: String = String::new();

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
                        // Apply noise to hopefully prevent farming.
                        if game.ply < 16 {
                            println!("Applying noise!");
                            player.apply_dirichlet(&game, 1.0, 0.35);
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
                    last_move = s;

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
        write(
            format!("analysis_{}.ptn", sys_time()), 
            player.get_analysis().to_ptn()).unwrap_or_else(|err| println!("{err}")
        );
    }
}
