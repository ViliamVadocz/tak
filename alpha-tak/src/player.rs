use std::{
    ops::DerefMut,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
        Mutex,
    },
    thread::spawn,
};

use tak::*;

use crate::{
    analysis::Analysis,
    example::{Example, IncompleteExample},
    model::network::Network,
    search::{Node, NodeDebugInfo},
};

type Request<const N: usize> = (Game<N>, u32);
type Response<const N: usize> = (Vec<Vec<usize>>, Vec<Game<N>>);

pub struct Player<'a, const N: usize, NET: Network<N>> {
    node: Arc<Mutex<Node>>,
    network: &'a NET,

    request_tx: Sender<Request<N>>,
    response_rx: Receiver<Response<N>>,
    batch: u32,

    save_examples: bool,
    create_analysis: bool,

    examples: Vec<IncompleteExample<N>>,
    analysis: Analysis,
}

impl<'a, const N: usize, NET: Network<N>> Player<'a, N, NET> {
    pub fn new(
        network: &'a NET,
        batch: u32,
        save_examples: bool,
        create_analysis: bool,
        game: &Game<N>,
    ) -> Self {
        let (request_tx, request_rx) = channel();
        let (response_tx, response_rx) = channel();

        let instance = Self {
            node: Default::default(),
            network,
            request_tx,
            response_rx,
            batch,
            save_examples,
            create_analysis,
            examples: Vec::new(),
            analysis: Analysis::new(N as u8, game.half_komi),
        };

        // Create virtual rollout thread.
        let node = instance.node.clone();
        Self::run_rollout_thread(node, request_rx, response_tx);

        // Request the first batch.
        instance.request_batch(game);

        instance
    }

    fn run_rollout_thread(
        node: Arc<Mutex<Node>>,
        request_rx: Receiver<Request<N>>,
        response_tx: Sender<Response<N>>,
    ) {
        spawn(move || {
            while let Ok((game, batch)) = request_rx.recv() {
                let mut node = node.lock().unwrap();
                let paths: (Vec<_>, Vec<_>) = (0..batch)
                    .filter_map(|_| {
                        let mut path = vec![];
                        let mut game = game.clone();
                        if node.virtual_rollout(&mut game, &mut path) == GameResult::Ongoing {
                            Some((path, game))
                        } else {
                            None
                        }
                    })
                    .unzip();

                response_tx.send(paths).unwrap();
            }
        });
    }

    fn request_batch(&self, game: &Game<N>) {
        self.request_tx.send((game.clone(), self.batch)).unwrap();
    }

    fn consume_batch(&self) {
        let (paths, games) = self.response_rx.recv().unwrap();
        let net_outputs = self.network.policy_eval(games.as_slice());

        let mut node = self.node.lock().unwrap();
        net_outputs.into_iter().zip(paths).for_each(|(result, path)| {
            node.devirtualize_path::<N, _>(&mut path.into_iter(), &result);
        });
    }

    pub fn debug(&self, depth: usize) -> NodeDebugInfo {
        self.node.lock().unwrap().debug(depth)
    }

    /// Do a batch of rollouts.
    pub fn rollout(&mut self, game: &Game<N>) {
        self.request_batch(game);
        self.consume_batch();
    }

    /// Pick a move to play.
    pub fn pick_move(&mut self, game: &Game<N>, exploitation: bool) -> Move {
        self.node.lock().unwrap().pick_move(exploitation)
    }

    /// Update the search tree, analysis, and create an example.
    pub fn play_move(&mut self, game: &Game<N>, my_move: Move) {
        // rollout stale paths
        // necessary to update policies accordingly
        // TODO: avoid rolling out nodes that are going to be discarded
        self.consume_batch();

        let mut node = self.node.lock().unwrap();

        // Save example.
        if self.save_examples {
            self.examples.push(IncompleteExample {
                game: game.clone(),
                policy: node.improved_policy(),
            });
        }

        // Update analysis.
        if self.create_analysis {
            self.analysis.update(&node, my_move);
        }

        *node = std::mem::take(node.deref_mut()).play(my_move);

        // Refill queue.
        let mut game = game.clone();
        game.play(my_move).unwrap();
        self.request_batch(&game);
    }

    /// Complete collected examples with the game result and return them.
    /// The examples in the Player will be empty after this method is used.
    pub fn get_examples(&mut self, result: GameResult) -> Vec<Example<N>> {
        let white_result = match result {
            GameResult::Winner {
                color: Color::White, ..
            } => 1.,
            GameResult::Winner {
                color: Color::Black, ..
            } => -1.,
            GameResult::Draw { .. } => 0.,
            GameResult::Ongoing { .. } => unreachable!("cannot complete examples with ongoing game"),
        };
        std::mem::take(&mut self.examples)
            .into_iter()
            .map(|ex| {
                let perspective = if ex.game.to_move == Color::White {
                    white_result
                } else {
                    -white_result
                };
                ex.complete(perspective)
            })
            .collect()
    }

    /// Get the analysis of the game
    pub fn get_analysis(&mut self) -> Analysis {
        std::mem::take(&mut self.analysis)
    }
}
