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
use tch::{Device, Tensor};

use crate::{
    agent::{Agent, Batcher},
    example::{Example, IncompleteExample},
    mcts::Node,
    network::Network,
    repr::game_repr,
    turn_map::Lut,
};

const SELF_PLAY_GAMES: usize = 2000;
const ROLLOUTS_PER_MOVE: u32 = 1000;
const PIT_MATCHES: usize = 128;
const WIN_RATE_THRESHOLD: f64 = 0.55;
const MAX_EXAMPLES: usize = 1_000_000;
const OPENING_PLIES: usize = 6;

/// Run multiple games against self.
#[allow(dead_code)]
fn self_play<const N: usize>(network: &Network<N>) -> Vec<Example<N>>
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
    while completed_games < SELF_PLAY_GAMES || workers.iter().any(|handle| handle.is_running()) {
        // collect game states
        let mut communicators: ArrayVec<_, WORKERS> = ArrayVec::new();
        let mut batch: ArrayVec<_, WORKERS> = ArrayVec::new();
        for (i, rx) in receivers.iter().enumerate() {
            if let Ok(game) = rx.try_recv() {
                communicators.push(i);
                batch.push(game);
            }
        }
        if batch.is_empty() {
            continue;
        }

        // run prediction
        let game_tensors: Vec<_> = batch.iter().map(game_repr).collect();
        let input = Tensor::stack(&game_tensors, 0).to_device(Device::cuda_if_available());
        let (policy, eval) = network.forward_mcts(input);
        let policies: Vec<Vec<f32>> = policy.into();
        let evals: Vec<f32> = eval.into();

        // send out outputs
        for (i, r) in communicators
            .into_iter()
            .zip(policies.into_iter().zip(evals.into_iter()))
        {
            transmitters[i].send(r).unwrap();
        }

        for (i, handle) in workers.iter_mut().enumerate() {
            // track when threads finish
            if !handle.is_running() && completed_games < SELF_PLAY_GAMES {
                completed_games += 1;
                println!("self-play game {completed_games}/{SELF_PLAY_GAMES}");
                // start a new thread when one finishes
                *handle = new_worker(examples_tx.clone(), &mut receivers, &mut transmitters, Some(i));
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
    let mut game = Game::default(); // TODO add komi?

    // make random moves for the first few turns to diversify training data
    for _ in 0..OPENING_PLIES {
        game.nth_place(random()).unwrap();
    }

    // initialize MCTS
    let mut node = Node::default();
    while matches!(game.winner(), GameResult::Ongoing) {
        for _ in 0..ROLLOUTS_PER_MOVE {
            node.rollout(game.clone(), agent);
        }
        // and incomplete example
        game_examples.push(IncompleteExample {
            game: game.clone(),
            policy: node.improved_policy(),
        });
        // pick a turn and play it
        let turn = node.pick_move(false);
        node = node.play(&turn);
        game.play(turn).unwrap();
    }
    let winner = game.winner();
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
            let perspective = if ply % 2 == 0 { result } else { -result };
            ex.complete(perspective)
        })
        .collect()
}

#[derive(Debug, Default)]
struct PitResult {
    wins: u32,
    #[allow(dead_code)]
    draws: u32,
    losses: u32,
}

impl PitResult {
    pub fn win_rate(&self) -> f64 {
        // another option:
        // (self.wins as f64 + self.draws as f64 / 2.) /
        // (self.wins + self.draws + self.losses) as f64
        self.wins as f64 / (self.wins + self.losses) as f64
    }

    pub fn update(&mut self, result: GameResult, colour: Colour) {
        match result {
            GameResult::Winner(winner) => {
                if winner == colour {
                    self.wins += 1
                } else {
                    self.losses += 1
                }
            }
            GameResult::Draw => self.draws += 1,
            GameResult::Ongoing => {}
        }
    }
}

// TODO cleanup
fn pit_async<const N: usize>(new: &Network<N>, old: &Network<N>) -> PitResult
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    println!("Starting {PIT_MATCHES} pit games asynchronously");

    // initialize worker threads
    let mut workers: ArrayVec<_, PIT_MATCHES> = ArrayVec::new();
    let mut receivers_old: ArrayVec<_, PIT_MATCHES> = ArrayVec::new();
    let mut transmitters_old: ArrayVec<_, PIT_MATCHES> = ArrayVec::new();
    let mut receivers_new: ArrayVec<_, PIT_MATCHES> = ArrayVec::new();
    let mut transmitters_new: ArrayVec<_, PIT_MATCHES> = ArrayVec::new();
    let (results_tx, results_rx) = channel();
    for _ in 0..PIT_MATCHES {
        let (game_tx_new, game_rx_new) = channel();
        let (policy_tx_new, policy_rx_new) = channel();
        receivers_new.push(game_rx_new);
        transmitters_new.push(policy_tx_new);
        let batcher_new = Batcher::new(game_tx_new, policy_rx_new);

        let (game_tx_old, game_rx_old) = channel();
        let (policy_tx_old, policy_rx_old) = channel();
        let batcher_old = Batcher::new(game_tx_old, policy_rx_old);
        receivers_old.push(game_rx_old);
        transmitters_old.push(policy_tx_old);

        let tx = results_tx.clone();
        workers.push(thread::spawn(move || {
            tx.send(play_pit_game(&batcher_new, &batcher_old)).unwrap();
        }));
    }

    while workers.iter().any(|handle| handle.is_running()) {
        // collect game states for new
        let mut communicators = Vec::new();
        let mut batch = Vec::new();
        for (i, rx) in receivers_new.iter().enumerate() {
            if let Ok(game) = rx.try_recv() {
                communicators.push(i);
                batch.push(game);
            }
        }
        if !batch.is_empty() {
            // run prediction
            let game_tensors: Vec<_> = batch.iter().map(game_repr).collect();
            let input = Tensor::stack(&game_tensors, 0).to_device(Device::cuda_if_available());
            let (policy, eval) = new.forward_mcts(input);
            let policies: Vec<Vec<f32>> = policy.into();
            let evals: Vec<f32> = eval.into();

            // send out outputs
            for (i, r) in communicators
                .into_iter()
                .zip(policies.into_iter().zip(evals.into_iter()))
            {
                transmitters_new[i].send(r).unwrap();
            }
        }

        // collect game states for old
        let mut communicators = Vec::new();
        let mut batch = Vec::new();
        for (i, rx) in receivers_old.iter().enumerate() {
            if let Ok(game) = rx.try_recv() {
                communicators.push(i);
                batch.push(game);
            }
        }
        if !batch.is_empty() {
            // run prediction
            let game_tensors: Vec<_> = batch.iter().map(game_repr).collect();
            let input = Tensor::stack(&game_tensors, 0).to_device(Device::cuda_if_available());
            let (policy, eval) = old.forward_mcts(input);
            let policies: Vec<Vec<f32>> = policy.into();
            let evals: Vec<f32> = eval.into();

            // send out outputs
            for (i, r) in communicators
                .into_iter()
                .zip(policies.into_iter().zip(evals.into_iter()))
            {
                transmitters_old[i].send(r).unwrap();
            }
        }
    }

    // collect examples
    results_rx
        .iter()
        .take(PIT_MATCHES)
        .fold(PitResult::default(), |mut r, (as_white, as_black)| {
            r.update(as_white, Colour::White);
            r.update(as_black, Colour::Black);
            r
        })
}

/// Pits two networks against each other.
/// Returns wins, draws, and losses of the match.
#[allow(dead_code)]
fn pit<const N: usize>(new: &Network<N>, old: &Network<N>) -> PitResult
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    (0..(PIT_MATCHES / 2)).fold(PitResult::default(), |mut result, _| {
        let (as_white, as_black) = play_pit_game(new, old);
        result.update(as_white, Colour::White);
        result.update(as_black, Colour::Black);
        result
    })
}

/// Play an opening from both sides with two different agents.
fn play_pit_game<const N: usize, A: Agent<N>>(new: &A, old: &A) -> (GameResult, GameResult)
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    let id: usize = random();

    // Play game from both sides
    let mut results = ArrayVec::<_, 2>::new();
    for my_colour in [Colour::White, Colour::Black] {
        let mut game = Game::default(); // TODO add komi?
        game.opening(id).unwrap();
        // Initialize MCTS
        let mut my_node = Node::default();
        let mut opp_node = Node::default();
        while matches!(game.winner(), GameResult::Ongoing) {
            // At least one rollout to initialize all the moves in the trees
            my_node.rollout(game.clone(), new);
            opp_node.rollout(game.clone(), old);
            let turn = if game.to_move == my_colour {
                for _ in 0..ROLLOUTS_PER_MOVE {
                    my_node.rollout(game.clone(), new);
                }
                my_node.pick_move(true)
            } else {
                for _ in 0..ROLLOUTS_PER_MOVE {
                    opp_node.rollout(game.clone(), old);
                }
                opp_node.pick_move(true)
            };
            my_node = my_node.play(&turn);
            opp_node = opp_node.play(&turn);
            game.play(turn).unwrap();
        }

        results.push(game.winner());
    }
    (results[0], results[1])
}

/// Do self-play and test against previous iteration
/// until an improvement is seen.
pub fn play_until_better<const N: usize>(network: Network<N>, examples: &mut Vec<Example<N>>) -> Network<N>
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    loop {
        // examples.extend(self_play(&network).into_iter());
        examples.extend(self_play_async(&network).into_iter());
        if examples.len() > MAX_EXAMPLES {
            examples.reverse();
            examples.truncate(MAX_EXAMPLES);
            examples.reverse();
        }

        let mut new_network = copy(&network);
        new_network.train(examples);

        println!("pitting two networks against each other");
        // let results = pit(&new_network, &network);
        let results = pit_async(&new_network, &network);
        println!("{:?}", results);
        if results.win_rate() > WIN_RATE_THRESHOLD {
            return new_network;
        }
    }
}

fn copy<const N: usize>(network: &Network<N>) -> Network<N> {
    // copy network values by file (UGLY)
    let mut dir = std::env::temp_dir();
    dir.push("model");
    network.save(&dir).unwrap();
    Network::<N>::load(&dir).unwrap()
}
