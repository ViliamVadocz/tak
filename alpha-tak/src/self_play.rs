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

const SELF_PLAY_GAMES: usize = 1000;
const ROLLOUTS_PER_MOVE: u32 = 200;
const PIT_GAMES: u32 = 200;
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
pub fn self_play_batch<const N: usize>(network: &Network<N>) -> Vec<Example<N>>
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
            if !handle.is_running() {
                completed_games += 1;
                println!("self-play game {completed_games}/{SELF_PLAY_GAMES}");
                if completed_games < SELF_PLAY_GAMES {
                    // start a new thread when one finishes
                    *handle = new_worker(examples_tx.clone(), &mut receivers, &mut transmitters, Some(i));
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

#[derive(Debug)]
struct PitResult {
    wins: u32,
    #[allow(dead_code)]
    draws: u32,
    losses: u32,
}

impl PitResult {
    pub fn win_rate(&self) -> f64 {
        // (self.wins as f64 + self.draws as f64 / 2.) / (self.wins + self.draws +
        // self.losses) as f64
        self.wins as f64 / (self.wins + self.losses) as f64
    }
}

/// Pits two networks against each other.
/// Returns wins, draws, and losses of the match.
fn pit<const N: usize>(new: &Network<N>, old: &Network<N>) -> PitResult
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    println!("pitting two networks against each other");
    let mut wins = 0;
    let mut draws = 0;
    let mut losses = 0;

    for i in 0..PIT_GAMES {
        let win_all = PitResult {
            wins: wins + PIT_GAMES - i,
            draws,
            losses,
        };
        let lose_all = PitResult {
            wins,
            draws,
            losses: losses + PIT_GAMES - i,
        };
        if win_all.win_rate() < WIN_RATE_THRESHOLD || lose_all.win_rate() > WIN_RATE_THRESHOLD {
            println!("ending early because result is already determined");
            break;
        }

        println!("pit game: {i}/{PIT_GAMES}");
        // TODO add komi?
        let mut game = Game::default();
        game.opening(i as usize / 2).unwrap();
        let my_colour = if i % 2 == 0 { Colour::White } else { Colour::Black };

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
        let game_result = game.winner();
        match game_result {
            GameResult::Winner(winner) => {
                if winner == my_colour {
                    wins += 1;
                } else {
                    losses += 1;
                }
            }
            GameResult::Draw => draws += 1,
            GameResult::Ongoing => unreachable!(),
        }
        println!(
            "{game_result:?} as {my_colour:?} in {} plies [{wins}/{draws}/{losses}]\n{}",
            game.ply, game.board
        );
    }

    PitResult { wins, draws, losses }
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
        examples.extend(self_play_batch(&network).into_iter());
        if examples.len() > MAX_EXAMPLES {
            examples.reverse();
            examples.truncate(MAX_EXAMPLES);
            examples.reverse();
        }

        let mut new_network = copy(&network);
        new_network.train(examples);
        let results = pit(&new_network, &network);
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
