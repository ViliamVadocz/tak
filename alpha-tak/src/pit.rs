use std::{sync::mpsc::channel, thread};

use arrayvec::ArrayVec;
use rand::random;
use tak::*;

use crate::{
    agent::{Agent, Batcher},
    config::{KOMI, PIT_MATCHES, ROLLOUTS_PER_MOVE},
    model::network::Network,
    search::{node::Node, turn_map::Lut},
};

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
    println!("Playing {PIT_MATCHES} pit matches at the same time");
    let opening = random::<usize>() / 2; // avoid overflow

    // initialize worker threads
    let mut workers: ArrayVec<_, PIT_MATCHES> = ArrayVec::new();
    let mut receivers_old: ArrayVec<_, PIT_MATCHES> = ArrayVec::new();
    let mut transmitters_old: ArrayVec<_, PIT_MATCHES> = ArrayVec::new();
    let mut receivers_new: ArrayVec<_, PIT_MATCHES> = ArrayVec::new();
    let mut transmitters_new: ArrayVec<_, PIT_MATCHES> = ArrayVec::new();
    let (results_tx, results_rx) = channel();
    for i in 0..PIT_MATCHES {
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
            tx.send(play_pit_game(&batcher_new, &batcher_old, opening + i))
                .unwrap();
        }));
    }

    while workers.iter().any(|thread| thread.is_running()) {
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
            let (policies, evals) = new.policy_eval_batch(&batch);

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
            let (policies, evals) = old.policy_eval_batch(&batch);

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
pub fn pit<const N: usize>(new: &Network<N>, old: &Network<N>) -> PitResult
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    let opening = random::<usize>() / 2; // avoid overflow
    (0..(PIT_MATCHES / 2)).fold(PitResult::default(), |mut result, i| {
        let (as_white, as_black) = play_pit_game(new, old, opening + i);
        result.update(as_white, Colour::White);
        result.update(as_black, Colour::Black);
        result
    })
}

/// Play an opening from both sides with two different agents.
fn play_pit_game<const N: usize, A: Agent<N>>(new: &A, old: &A, opening: usize) -> (GameResult, GameResult)
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: Lut,
{
    // Play game from both sides
    let mut results = ArrayVec::<_, 2>::new();
    for my_colour in [Colour::White, Colour::Black] {
        let mut game = Game::with_komi(KOMI);
        game.opening(opening).unwrap();
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
        let winner = game.winner();
        println!(
            "{winner:?} as {my_colour:?} in {} plies\n{}",
            game.ply, game.board
        );
        results.push(winner);
    }
    (results[0], results[1])
}
