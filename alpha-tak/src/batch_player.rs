use tak::*;

use crate::{
    analysis::Analysis,
    example::{Example, IncompleteExample},
    model::network::Network,
    search::{node::Node, turn_map::Lut},
};

pub struct BatchPlayer<'a, const N: usize> {
    node: Node<N>,
    network: &'a Network<N>,
    examples: Vec<IncompleteExample<N>>,
    analysis: Analysis<N>,
}

impl<'a, const N: usize> BatchPlayer<'a, N>
where
    Turn<N>: Lut,
{
    pub fn new(network: &'a Network<N>, opening: Vec<Turn<N>>, komi: i32) -> Self {
        Self {
            node: Node::default(),
            network,
            examples: Vec::new(),
            analysis: Analysis::from_opening(opening, komi),
        }
    }

    pub fn debug(&self, limit: Option<usize>) -> String {
        self.node.debug(limit)
    }

    /// Do some amount of rollouts.
    pub fn rollout(&mut self, game: &Game<N>, amount: usize) {
        let (paths, games): (Vec<_>, Vec<_>) = (0..amount)
            .filter_map(|_| {
                let mut path = vec![];
                let mut game = game.clone();
                if self.node.virtual_rollout(&mut game, &mut path) == GameResult::Ongoing {
                    Some((path, game))
                } else {
                    None
                }
            })
            .unzip();

        let (policy_vecs, evals) = if games.is_empty() {
            Default::default()
        } else {
            self.network.policy_eval_batch(games.as_slice())
        };

        policy_vecs
            .into_iter()
            .zip(evals)
            .zip(paths)
            .for_each(|(result, path)| {
                self.node.devirtualize_path(&mut path.into_iter(), &result);
            });
    }

    /// Pick a move to play and also play it.
    pub fn pick_move(&mut self, game: &Game<N>, exploitation: bool) -> Turn<N> {
        let turn = self.node.pick_move(exploitation);
        self.play_move(game, &turn);
        turn
    }

    /// Update the search tree, analysis, and create an example.
    pub fn play_move(&mut self, game: &Game<N>, turn: &Turn<N>) {
        self.node.rollout(game.clone(), self.network); // at least one rollout
        self.save_example(game.clone());
        self.analysis.update(&self.node, turn.clone());

        let node = std::mem::take(&mut self.node);
        self.node = node.play(turn);
    }

    fn save_example(&mut self, game: Game<N>) {
        self.examples.push(IncompleteExample {
            game,
            policy: self.node.improved_policy(),
        })
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
        self.rollout(game, 1);
        self.node.apply_dirichlet(alpha, ratio);
    }
}
