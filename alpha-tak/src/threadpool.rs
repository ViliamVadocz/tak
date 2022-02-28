use std::{
    cmp::min,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
};

use arrayvec::ArrayVec;
use tak::Game;

use crate::{agent::Batcher, example::Example, model::network::Network};

type Output<const N: usize> = Vec<Example<N>>;

pub fn thread_pool<const N: usize, const WORKERS: usize, F>(
    network: &Network<N>,
    number_of_games: usize,
    func: F,
) -> Vec<Output<N>>
where
    F: Fn(&Batcher<N>) -> Output<N> + Copy + Send + 'static,
{
    let mut workers: ArrayVec<_, WORKERS> = ArrayVec::new();
    let mut game_receivers: ArrayVec<_, WORKERS> = ArrayVec::new();
    let mut policy_senders: ArrayVec<_, WORKERS> = ArrayVec::new();

    // initialize workers
    for _ in 0..min(WORKERS, number_of_games) {
        workers.push(Some(new_worker(
            func,
            &mut game_receivers,
            &mut policy_senders,
            None,
        )));
    }

    let mut completed_games = 0;
    let mut outputs = Vec::new();
    while completed_games < number_of_games
        || workers
            .iter()
            .any(|handle| handle.as_ref().map(|h| h.is_running()).unwrap_or_default())
    {
        // collect game states
        let mut communicators = [false; WORKERS];
        let mut batch = Vec::with_capacity(WORKERS);
        for (i, rx) in game_receivers.iter().enumerate() {
            if let Ok(game) = rx.try_recv() {
                communicators[i] = true;
                batch.push(game);
            }
        }
        if batch.is_empty() {
            continue;
        }
        // run prediction
        let (policies, evals) = network.policy_eval_batch(&batch);

        // send out outputs
        for (i, r) in communicators
            .into_iter()
            .enumerate()
            .filter(|(_, communicated)| *communicated)
            .map(|(i, _)| i)
            .zip(policies.into_iter().zip(evals.into_iter()))
        {
            policy_senders[i].send(r).unwrap();
        }

        for (i, maybe_handle) in workers.iter_mut().enumerate() {
            // track when threads finish
            if let Some(handle) = maybe_handle.take() {
                *maybe_handle = if !handle.is_running() {
                    completed_games += 1;
                    println!("{completed_games}/{number_of_games}");
                    outputs.push(handle.join().unwrap());

                    // start a new thread when one finishes
                    if completed_games + WORKERS <= number_of_games + 1 {
                        Some(new_worker(
                            func,
                            &mut game_receivers,
                            &mut policy_senders,
                            Some(i),
                        ))
                    } else {
                        None
                    }
                } else {
                    Some(handle)
                };
            }
        }
    }

    outputs
}

fn new_worker<const N: usize, const WORKERS: usize, F>(
    func: F,
    game_receivers: &mut ArrayVec<Receiver<Game<N>>, WORKERS>,
    policy_senders: &mut ArrayVec<Sender<(Vec<f32>, f32)>, WORKERS>,
    overwrite: Option<usize>,
) -> JoinHandle<Output<N>>
where
    F: Fn(&Batcher<N>) -> Output<N> + Send + 'static,
{
    let (game_tx, game_rx) = channel();
    let (policy_tx, policy_rx) = channel();
    if let Some(i) = overwrite {
        game_receivers[i] = game_rx;
        policy_senders[i] = policy_tx;
    } else {
        game_receivers.push(game_rx);
        policy_senders.push(policy_tx);
    }
    let batcher = Batcher::new(game_tx, policy_rx);
    thread::spawn(move || func(&batcher))
}
