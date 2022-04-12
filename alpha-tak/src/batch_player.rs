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
    search::{node::Node, turn_map::Lut},
};

pub struct BatchPlayer<'a, const N: usize> {
    node: Arc<Mutex<Node<N>>>,
    network: &'a Network<N>,
    examples: Vec<IncompleteExample<N>>,
    analysis: Analysis<N>,
    request_tx: Sender<(Game<N>, u32)>,
    response_rx: Receiver<(Vec<Vec<Turn<N>>>, Vec<Game<N>>)>,
    batch: u32,
}

impl<'a, const N: usize> BatchPlayer<'a, N>
where
    Turn<N>: Lut,
{
    fn request_batch(&self, game: &Game<N>) {
        self.request_tx.send((game.clone(), self.batch)).unwrap();
    }

    fn consume_batch(&self) {
        let (paths, games) = self.response_rx.recv().unwrap();

        let (policy_vecs, evals) = if games.is_empty() {
            Default::default()
        } else {
            self.network.policy_eval_batch(games.as_slice())
        };

        let mut node = self.node.lock().unwrap();
        policy_vecs
            .into_iter()
            .zip(evals)
            .zip(paths)
            .for_each(|(result, path)| {
                node.devirtualize_path(&mut path.into_iter(), &result);
            });
    }

    pub fn new(
        game: &Game<N>,
        network: &'a Network<N>,
        opening: Vec<Turn<N>>,
        komi: i32,
        batch: u32,
    ) -> Self {
        let (request_tx, request_rx) = channel();
        let (response_tx, response_rx) = channel();

        let instance = Self {
            node: Default::default(),
            network,
            examples: Vec::new(),
            analysis: Analysis::from_opening(opening, komi),
            request_tx,
            response_rx,
            batch,
        };

        let node = instance.node.clone();
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

        instance.request_batch(game);

        instance
    }

    pub fn debug(&self, limit: Option<usize>) -> String {
        self.node.lock().unwrap().debug(limit)
    }

    /// Do a batch of rollouts.
    pub fn rollout(&mut self, game: &Game<N>) {
        self.request_batch(game);
        self.consume_batch();
    }

    /// Pick a move to play and also play it.
    pub fn pick_move(&mut self, game: &Game<N>, exploitation: bool) -> Turn<N> {
        let turn = self.node.lock().unwrap().pick_move(exploitation);
        self.play_move(game, &turn);
        turn
    }

    /// Update the search tree, analysis, and create an example.
    pub fn play_move(&mut self, game: &Game<N>, turn: &Turn<N>) {
        // rollout stale paths
        // necessary to update policies accordingly
        // TODO: avoid rolling out nodes that are going to be discarded
        self.consume_batch();

        let mut node = self.node.lock().unwrap();

        // save example
        self.examples.push(IncompleteExample {
            game: game.clone(),
            policy: node.improved_policy(),
        });

        self.analysis.update(&node, turn.clone());

        *node = std::mem::take(node.deref_mut()).play(turn);

        // refill queue
        let mut game = game.clone();
        game.play(turn.clone()).unwrap();
        self.request_batch(&game);
    }

    /// Complete collected examples with the game result and return them.
    /// The examples in the Player will be empty after this method is used.
    pub fn get_examples(&mut self, result: GameResult) -> Vec<Example<N>> {
        let white_result = match result {
            GameResult::Winner {
                colour: Colour::White,
                ..
            } => 1.,
            GameResult::Winner {
                colour: Colour::Black,
                ..
            } => -1.,
            GameResult::Draw { .. } => 0.,
            GameResult::Ongoing { .. } => unreachable!("cannot complete examples with ongoing game"),
        };
        std::mem::take(&mut self.examples)
            .into_iter()
            .map(|ex| {
                let perspective = if ex.game.to_move == Colour::White {
                    white_result
                } else {
                    -white_result
                };
                ex.complete(perspective)
            })
            .collect()
    }

    /// Get the analysis of the game
    pub fn get_analysis(&mut self) -> Analysis<N> {
        std::mem::take(&mut self.analysis)
    }

    /// Apply dirichlet noise to the top node
    pub fn apply_dirichlet(&mut self, game: &Game<N>, alpha: f32, ratio: f32) {
        let mut node = self.node.lock().unwrap();
        node.rollout(game.clone(), self.network);
        node.apply_dirichlet(alpha, ratio);
    }
}
