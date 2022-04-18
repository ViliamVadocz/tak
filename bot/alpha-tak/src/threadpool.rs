use std::{
    cmp::min,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
};

use arrayvec::ArrayVec;
use tak::Game;

use crate::{agent::Batcher, model::network::Network};

// This code is still ugly
// TODO rewrite again

pub fn thread_pool<const N: usize, const WORKERS: usize, F, O>(
    network: &Network<N>,
    number_of_games: usize,
    func: F,
) -> Vec<O>
where
    F: Fn(&Batcher<N>, usize) -> O + Copy + Send + 'static,
    O: Send + 'static,
{
    let mut workers: ArrayVec<_, WORKERS> = ArrayVec::new();
    let mut game_receivers: ArrayVec<_, WORKERS> = ArrayVec::new();
    let mut policy_senders: ArrayVec<_, WORKERS> = ArrayVec::new();

    // initialize workers
    let mut index = 0;
    for _ in 0..min(WORKERS, number_of_games) {
        workers.push(Some(new_worker(
            func,
            &mut game_receivers,
            &mut policy_senders,
            None,
            index,
        )));
        index += 1;
    }

    let mut completed_games = 0;
    let mut outputs = Vec::new();
    while completed_games < number_of_games || workers.iter().any(|worker| worker.is_some()) {
        // collect game states
        let mut communicators = [false; WORKERS];
        let mut batch = Vec::with_capacity(WORKERS);
        for (i, rx) in game_receivers.iter().enumerate() {
            if let Ok(game) = rx.try_recv() {
                communicators[i] = true;
                batch.push(game);
            }
        }

        if !batch.is_empty() {
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
        }

        for (i, maybe_handle) in workers.iter_mut().enumerate() {
            // track when threads finish
            if let Some(handle) = std::mem::take(maybe_handle) {
                *maybe_handle = if handle.is_finished() {
                    completed_games += 1;
                    println!("{completed_games}/{number_of_games}");
                    outputs.push(handle.join().unwrap());

                    // start a new thread when one finishes
                    if completed_games + WORKERS <= number_of_games + 1 {
                        index += 1;
                        Some(new_worker(
                            func,
                            &mut game_receivers,
                            &mut policy_senders,
                            Some(i),
                            index - 1,
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

fn new_worker<const N: usize, const WORKERS: usize, F, O>(
    func: F,
    game_receivers: &mut ArrayVec<Receiver<Game<N>>, WORKERS>,
    policy_senders: &mut ArrayVec<Sender<(Vec<f32>, f32)>, WORKERS>,
    overwrite: Option<usize>,
    index: usize,
) -> JoinHandle<O>
where
    F: Fn(&Batcher<N>, usize) -> O + Send + 'static,
    O: Send + 'static,
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
    thread::spawn(move || func(&batcher, index))
}

pub fn thread_pool_2<const N: usize, const WORKERS: usize, F, O>(
    network_1: &Network<N>,
    network_2: &Network<N>,
    number_of_games: usize,
    func: F,
) -> Vec<O>
where
    F: Fn(&Batcher<N>, &Batcher<N>, usize) -> O + Copy + Send + 'static,
    O: Send + 'static,
{
    let mut workers: ArrayVec<_, WORKERS> = ArrayVec::new();
    let mut game_receivers_1: ArrayVec<_, WORKERS> = ArrayVec::new();
    let mut game_receivers_2: ArrayVec<_, WORKERS> = ArrayVec::new();
    let mut policy_senders_1: ArrayVec<_, WORKERS> = ArrayVec::new();
    let mut policy_senders_2: ArrayVec<_, WORKERS> = ArrayVec::new();

    // initialize workers
    let mut index = 0;
    for _ in 0..min(WORKERS, number_of_games) {
        workers.push(Some(new_worker_2(
            func,
            &mut game_receivers_1,
            &mut game_receivers_2,
            &mut policy_senders_1,
            &mut policy_senders_2,
            None,
            index,
        )));
        index += 1;
    }

    let mut completed_games = 0;
    let mut outputs = Vec::new();
    while completed_games < number_of_games || workers.iter().any(|worker| worker.is_some()) {
        // collect game states
        let mut communicators = [false; WORKERS];
        let mut batch = Vec::with_capacity(WORKERS);
        for (i, rx) in game_receivers_1.iter().enumerate() {
            if let Ok(game) = rx.try_recv() {
                communicators[i] = true;
                batch.push(game);
            }
        }
        if !batch.is_empty() {
            // run prediction
            let (policies, evals) = network_1.policy_eval_batch(&batch);

            // send out outputs
            for (i, r) in communicators
                .into_iter()
                .enumerate()
                .filter(|(_, communicated)| *communicated)
                .map(|(i, _)| i)
                .zip(policies.into_iter().zip(evals.into_iter()))
            {
                policy_senders_1[i].send(r).unwrap();
            }
        }

        // collect game states
        let mut communicators = [false; WORKERS];
        let mut batch = Vec::with_capacity(WORKERS);
        for (i, rx) in game_receivers_2.iter().enumerate() {
            if let Ok(game) = rx.try_recv() {
                communicators[i] = true;
                batch.push(game);
            }
        }
        if !batch.is_empty() {
            // run prediction
            let (policies, evals) = network_2.policy_eval_batch(&batch);

            // send out outputs
            for (i, r) in communicators
                .into_iter()
                .enumerate()
                .filter(|(_, communicated)| *communicated)
                .map(|(i, _)| i)
                .zip(policies.into_iter().zip(evals.into_iter()))
            {
                policy_senders_2[i].send(r).unwrap();
            }
        }

        for (i, maybe_handle) in workers.iter_mut().enumerate() {
            // track when threads finish
            if let Some(handle) = std::mem::take(maybe_handle) {
                *maybe_handle = if handle.is_finished() {
                    completed_games += 1;
                    println!("{completed_games}/{number_of_games}");
                    outputs.push(handle.join().unwrap());

                    // start a new thread when one finishes
                    if completed_games + WORKERS <= number_of_games + 1 {
                        index += 1;
                        Some(new_worker_2(
                            func,
                            &mut game_receivers_1,
                            &mut game_receivers_2,
                            &mut policy_senders_1,
                            &mut policy_senders_2,
                            Some(i),
                            index - 1,
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

fn new_worker_2<const N: usize, const WORKERS: usize, F, O>(
    func: F,
    game_receivers_1: &mut ArrayVec<Receiver<Game<N>>, WORKERS>,
    game_receivers_2: &mut ArrayVec<Receiver<Game<N>>, WORKERS>,
    policy_senders_1: &mut ArrayVec<Sender<(Vec<f32>, f32)>, WORKERS>,
    policy_senders_2: &mut ArrayVec<Sender<(Vec<f32>, f32)>, WORKERS>,
    overwrite: Option<usize>,
    index: usize,
) -> JoinHandle<O>
where
    F: Fn(&Batcher<N>, &Batcher<N>, usize) -> O + Send + 'static,
    O: Send + 'static,
{
    let (game_tx_1, game_rx_1) = channel();
    let (game_tx_2, game_rx_2) = channel();
    let (policy_tx_1, policy_rx_1) = channel();
    let (policy_tx_2, policy_rx_2) = channel();
    if let Some(i) = overwrite {
        game_receivers_1[i] = game_rx_1;
        game_receivers_2[i] = game_rx_2;
        policy_senders_1[i] = policy_tx_1;
        policy_senders_2[i] = policy_tx_2;
    } else {
        game_receivers_1.push(game_rx_1);
        game_receivers_2.push(game_rx_2);
        policy_senders_1.push(policy_tx_1);
        policy_senders_2.push(policy_tx_2);
    }
    let batcher_1 = Batcher::new(game_tx_1, policy_rx_1);
    let batcher_2 = Batcher::new(game_tx_2, policy_rx_2);
    thread::spawn(move || func(&batcher_1, &batcher_2, index))
}
