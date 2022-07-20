use std::{fs::File, io::Write};

use alpha_tak::*;
use array_init::array_init;
use rand::{thread_rng, Rng};
use tak::*;

use crate::EXAMPLE_DIR;

const SELF_PLAY_GAMES: u32 = 1000;
const BATCH_SIZE: u32 = 32;
const ROLLOUTS: u32 = 10_000;

const NOISE_ALPHA: f32 = 0.2;
const NOISE_RATIO: f32 = 0.3;
const NOISE_PLIES: u16 = 80;

const EXPLOIT_PLIES: u16 = 40;
const QUAD_ROLLOUT_PLIES: u16 = 10;

pub fn self_play<const N: usize, NET: Network<N>>(network: &NET) -> Vec<Example<N>> {
    let mut examples = Vec::new();

    let mut example_file = File::create(format!("{EXAMPLE_DIR}/{}.data", sys_time())).unwrap();

    let mut rng = thread_rng();
    for i in 0..SELF_PLAY_GAMES {
        println!("self_play game {i}/{SELF_PLAY_GAMES}");
        let mut game = Game::with_komi(2);
        let mut player = Player::new(network, BATCH_SIZE, true, true, &game);

        // Do random opening.
        // for _ in 0..RANDOM_PLIES {
        //     let my_move = *game.possible_moves().choose(&mut rng).unwrap();
        //     player.play_move(my_move, &game, false);
        //     game.play(my_move).unwrap();
        // }

        // first
        let my_move = "a1".parse().unwrap();
        player.play_move(my_move, &game, false);
        game.play(my_move).unwrap();

        // second
        let my_move = if rng.gen::<f64>() < 0.5 {
            "a6".parse().unwrap()
        } else {
            "f6".parse().unwrap()
        };

        // let my_move = match rng.gen::<f64>() {
        //     x if x < 0.4 => "a6".parse().unwrap(),
        //     x if x < 0.8 => "f6".parse().unwrap(),
        //     _ => "a3".parse().unwrap(),
        // };
        player.play_move(my_move, &game, false);
        game.play(my_move).unwrap();

        while game.result() == GameResult::Ongoing {
            if game.ply < NOISE_PLIES {
                player.add_noise(NOISE_ALPHA, NOISE_RATIO, &game);
            }
            for _ in 0..if game.ply < QUAD_ROLLOUT_PLIES {
                4 * ROLLOUTS
            } else {
                ROLLOUTS
            } {
                player.rollout(&game);
            }
            let my_move = player.pick_move(game.ply >= EXPLOIT_PLIES);
            player.play_move(my_move, &game, true);
            game.play(my_move).unwrap();
        }
        println!("{:?} in {} plies", game.result(), game.ply);

        // Print analysis as debug.
        println!("BEGIN ANALYSIS");
        println!("{}", player.get_analysis().without_branches());
        println!("END ANALYSIS");

        // Save examples as we go to a file.
        let new_examples = player.get_examples(game.result());
        for example in &new_examples {
            writeln!(example_file, "{example}").unwrap();
        }
        example_file.flush().unwrap();
        // Save examples to output vector.
        examples.extend(new_examples.into_iter());
    }

    examples
}

const WORKERS: usize = 32;

pub fn self_play_parallel<const N: usize, NET: Network<N>>(network: &NET) -> Vec<Example<N>> {
    let mut examples = Vec::new();
    let mut example_file = File::create(format!("{EXAMPLE_DIR}/{}.data", sys_time())).unwrap();

    let mut rng = thread_rng();

    let mut nodes: [Node; WORKERS] = array_init(|_| Node::default());
    let mut games: [Option<Game<N>>; WORKERS] = array_init(|_| Some(Game::with_komi(2)));
    let mut incomplete_examples: [Vec<IncompleteExample<N>>; WORKERS] = array_init(|_| Vec::new());

    let mut completed_games = 0;

    while games.iter().any(Option::is_some) {
        // Play opening moves
        for game in games.iter_mut() {
            let game = if let Some(g) = game.as_mut() { g } else { continue };
            if game.ply == 0 {
                game.play("a1".parse().unwrap()).unwrap();
                game.play(if rng.gen::<bool>() {"a6".parse().unwrap()} else {"f6".parse().unwrap()}).unwrap();
            }
        }

        // Play winning moves if there are any
        for ((game, node), exs) in games.iter_mut().zip(nodes.iter_mut()).zip(incomplete_examples.iter_mut()) {
            let inner_game = if let Some(g) = game.as_mut() { g } else { continue };

            let mut win = false;
            let policy = inner_game
                .possible_moves()
                .into_iter()
                .map(|my_move| {
                    let mut clone = inner_game.clone();
                    clone.play(my_move).unwrap();
                    let visits = if matches!(clone.result(), GameResult::Winner { color, .. } if color == inner_game.to_move) {
                        win = true;
                        1_000 // high fake visits for winning moves
                    } else {
                        1 // at least one visit for all possible moves
                    };
                    (my_move, visits)
                })
                .collect::<Vec<_>>();
            if win {
                exs.push(IncompleteExample {
                    game: inner_game.clone(),
                    policy,
                });

                completed_games += 1;
                println!("win {completed_games}");

                let white_result = result_to_number(GameResult::Winner { color: inner_game.to_move, road: false });

                // Reset objects.
                *node = Node::default();
                if completed_games + WORKERS < SELF_PLAY_GAMES as usize {
                    *inner_game = Game::with_komi(2);
                } else {
                    *game = None;
                }

                // Complete examples.
                let new_examples: Vec<_> = exs.drain(..).map(|ex| {
                    let perspective = if ex.game.to_move == Color::White {
                        white_result
                    } else {
                        -white_result
                    };
                    ex.complete(perspective)
                }).collect();
                for e in &new_examples {
                    writeln!(example_file, "{e}").unwrap();
                }
                examples.extend(new_examples.into_iter());
            }
        }

        // Apply noise at the start of a ply.
        for (game, node) in games.iter().zip(nodes.iter_mut()) {
            let game = if let Some(g) = game.as_ref() { g } else { continue };
            if game.ply < NOISE_PLIES {
                node.rollout(game.clone(), network);
                node.apply_dirichlet(NOISE_ALPHA, NOISE_RATIO);
            }
        }
        for _ in 0..ROLLOUTS {
            // Virtual rollouts.
            let (indices, (for_eval, paths)): (Vec<_>, (Vec<_>, Vec<_>)) = games
                .clone()
                .into_iter()
                .enumerate()
                .filter_map(|(i, game)| game.map(|g| (i, g)))
                .zip(nodes.iter_mut())
                .filter_map(|((i, mut game), node)| {
                    let mut path = Vec::new();
                    if node.virtual_rollout(&mut game, &mut path) == GameResult::Ongoing {
                        Some((i, (game, path)))
                    } else {
                        // TODO maybe do more rollouts until virtual.
                        None
                    }
                })
                .unzip();

            // TODO Put this part on another thread and pipeline.
            // Network evaluation and de-virtualization.
            let network_output = network.policy_eval(&for_eval);
            network_output
                .into_iter()
                .zip(paths)
                .zip(indices)
                .for_each(|((net_output, path), i)| {
                    nodes[i].devirtualize_path::<N, _>(&mut path.into_iter(), &net_output);
                });
        }

        nodes
            .iter_mut()
            .zip(games.iter_mut())
            .zip(incomplete_examples.iter_mut())
            .filter(|((_, game), _)| game.is_some())
            .for_each(|((node, game), exs)| {
                let inner_game = game.as_mut().unwrap();

                let my_move = node.pick_move(inner_game.ply >= EXPLOIT_PLIES);

                exs.push(IncompleteExample {
                    game: inner_game.clone(),
                    policy: node.improved_policy(),
                });

                *node = std::mem::take(node).play(my_move);
                inner_game.play(my_move).unwrap();

                let result = inner_game.result();
                if result != GameResult::Ongoing {
                    completed_games += 1;
                    println!("normal game end {completed_games}");

                    // Reset objects.
                    *node = Node::default();
                    if completed_games + WORKERS < SELF_PLAY_GAMES as usize {
                        *inner_game = Game::with_komi(2);
                    } else {
                        *game = None;
                    }

                    // Complete examples.
                    let white_result = result_to_number(result);
                    let new_examples: Vec<_> = exs.drain(..).map(|ex| {
                        let perspective = if ex.game.to_move == Color::White {
                            white_result
                        } else {
                            -white_result
                        };
                        ex.complete(perspective)
                    }).collect();
                    for e in &new_examples {
                        writeln!(example_file, "{e}").unwrap();
                    }
                    examples.extend(new_examples.into_iter());
                }
            });
    }

    examples
}

fn result_to_number(result: GameResult) -> f32 {
    match result {
        GameResult::Winner {
            color: Color::White, ..
        } => 1.,
        GameResult::Winner {
            color: Color::Black, ..
        } => -1.,
        GameResult::Draw { .. } => 0.,
        GameResult::Ongoing { .. } => unreachable!("cannot complete examples with ongoing game"),
    }
}
