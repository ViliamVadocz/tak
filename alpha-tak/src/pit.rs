use std::{sync::mpsc::channel, thread};

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
    mcts::Node,
    network::Network,
    repr::game_repr,
    turn_map::Lut,
};

const ROLLOUTS_PER_MOVE: u32 = 1000;
const PIT_MATCHES: usize = 128;

#[derive(Debug, Default)]
pub struct PitResult {
    wins: u32,
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

    fn update(&mut self, result: GameResult, colour: Colour) {
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
pub fn pit_async<const N: usize>(new: &Network<N>, old: &Network<N>) -> PitResult
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
pub fn pit<const N: usize>(new: &Network<N>, old: &Network<N>) -> PitResult
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
