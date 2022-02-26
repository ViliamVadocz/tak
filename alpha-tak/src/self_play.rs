use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
};

use arrayvec::ArrayVec;
use rand::random;
use tak::{
    colour::Colour,
    game::{Game, GameResult},
    tile::Tile,
    turn::Turn,
};

use crate::{
    agent::{Agent, Batcher},
    example::{Example, IncompleteExample},
    mcts::Node,
    network::Network,
    turn_map::Lut,
    KOMI,
};

const SELF_PLAY_GAMES: usize = 1000;
const ROLLOUTS_PER_MOVE: u32 = 1000;
const OPENING_PLIES: usize = 3;
const DIRICHLET_NOISE: f32 = 0.15;
const NOISE_RATIO: f32 = 0.6;
const TEMPERATURE_PLIES: u64 = 20;

/// Run multiple games against self.
#[allow(dead_code)]
pub fn self_play<const N: usize>(network: &Network<N>) -> Vec<Example<N>>
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    (0..SELF_PLAY_GAMES).fold(Vec::new(), |mut examples, i| {
        examples.extend(self_play_game(network).into_iter());
        println!("self-play game {i}/{SELF_PLAY_GAMES}");
        examples
    })
}

/// Run multiple games against self concurrently.
pub fn self_play_async<const N: usize>(network: &Network<N>) -> Vec<Example<N>>
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    const WORKERS: usize = 128;
    println!("Starting self-play with {WORKERS} workers");

    fn new_worker<const N: usize>(
        tx: Sender<Vec<Example<N>>>,
        receivers: &mut ArrayVec<Receiver<Game<N>>, WORKERS>,
        transmitters: &mut ArrayVec<Sender<(Vec<f32>, f32)>, WORKERS>,
        overwrite: Option<usize>,
    ) -> JoinHandle<()>
    where
        [[Option<Tile>; N]; N]: Default,
        Turn<N>: Lut,
    {
        let (game_tx, game_rx) = channel();
        let (policy_tx, policy_rx) = channel();
        if let Some(i) = overwrite {
            receivers[i] = game_rx;
            transmitters[i] = policy_tx;
        } else {
            receivers.push(game_rx);
            transmitters.push(policy_tx);
        }
        let batcher = Batcher::new(game_tx, policy_rx);
        thread::spawn(move || {
            tx.send(self_play_game(&batcher)).unwrap();
        })
    }

    // initialize worker threads
    let mut workers: ArrayVec<_, WORKERS> = ArrayVec::new();
    let mut receivers: ArrayVec<_, WORKERS> = ArrayVec::new();
    let mut transmitters: ArrayVec<_, WORKERS> = ArrayVec::new();
    let (examples_tx, examples_rx) = channel();
    for _ in 0..WORKERS {
        workers.push(new_worker(
            examples_tx.clone(),
            &mut receivers,
            &mut transmitters,
            None,
        ));
    }

    let mut completed_games = 0;
    let mut done_threads = [false; WORKERS];
    while completed_games < SELF_PLAY_GAMES || workers.iter().any(|handle| handle.is_running()) {
        // collect game states
        let mut communicators = Vec::with_capacity(WORKERS);
        let mut batch = Vec::with_capacity(WORKERS);
        for (i, rx) in receivers.iter().enumerate() {
            if let Ok(game) = rx.try_recv() {
                communicators.push(i);
                batch.push(game);
            }
        }
        if batch.is_empty() {
            // println!("empty batch!");
            continue;
        }

        // run prediction
        let (policies, evals) = network.policy_eval_batch(&batch);

        // send out outputs
        for (i, r) in communicators
            .into_iter()
            .zip(policies.into_iter().zip(evals.into_iter()))
        {
            transmitters[i].send(r).unwrap();
        }

        for (i, handle) in workers.iter_mut().enumerate() {
            // track when threads finish
            if !handle.is_running() && !done_threads[i] {
                completed_games += 1;
                println!("self-play game {completed_games}/{SELF_PLAY_GAMES}");
                // start a new thread when one finishes
                if completed_games <= SELF_PLAY_GAMES - WORKERS + 1 {
                    *handle = new_worker(examples_tx.clone(), &mut receivers, &mut transmitters, Some(i));
                } else {
                    done_threads[i] = true;
                }
            }
        }
    }

    // collect examples
    examples_rx
        .iter()
        .take(completed_games)
        .fold(Vec::new(), |mut a, b| {
            a.extend(b.into_iter());
            a
        })
}

/// Run a single game against self.
fn self_play_game<const N: usize, A: Agent<N>>(agent: &A) -> Vec<Example<N>>
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    let mut game_examples = Vec::new();
    let mut game = Game::with_komi(KOMI);

    // make random moves for the first few turns to diversify training data
    for _ in 0..OPENING_PLIES {
        game.nth_place(random()).unwrap();
    }

    // initialize MCTS
    let mut node = Node::default();
    while matches!(game.winner(), GameResult::Ongoing) {
        node.rollout(game.clone(), agent); // at least one rollout to initialize children.
        node.apply_dirichlet(DIRICHLET_NOISE, NOISE_RATIO);
        for _ in 0..ROLLOUTS_PER_MOVE {
            node.rollout(game.clone(), agent);
        }
        // and incomplete example
        game_examples.push(IncompleteExample {
            game: game.clone(),
            policy: node.improved_policy(),
        });
        // pick a turn and play it
        let turn = node.pick_move(game.ply > TEMPERATURE_PLIES);
        node = node.play(&turn);
        game.play(turn).unwrap();
    }
    let winner = game.winner();
    println!("{winner:?} in {} plies\n{}", game.ply, game.board);
    // complete examples by filling in game result
    let result = match winner {
        GameResult::Winner(Colour::White) => 1.,
        GameResult::Winner(Colour::Black) => -1.,
        GameResult::Draw => 0.,
        GameResult::Ongoing => unreachable!(),
    };
    game_examples
        .into_iter()
        .enumerate()
        .map(|(ply, ex)| {
            let perspective = if (ply + OPENING_PLIES) % 2 == 0 {
                result
            } else {
                -result
            };
            ex.complete(perspective)
        })
        .collect()
}
