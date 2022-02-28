use tak::*;

use crate::{
    agent::Agent,
    analysis::Analysis,
    example::{Example, IncompleteExample},
    search::{node::Node, turn_map::Lut},
};

pub struct Player<'a, const N: usize, A: Agent<N>> {
    node: Node<N>,
    agent: &'a A,
    examples: Vec<IncompleteExample<N>>,
    pub analysis: Analysis<N>,
}

impl<'a, const N: usize, A: Agent<N>> Player<'a, N, A>
where
    Turn<N>: Lut,
{
    pub fn new(agent: &'a A, opening: Vec<Turn<N>>) -> Self {
        Player {
            node: Node::default(),
            agent,
            examples: Vec::new(),
            analysis: Analysis::from_opening(opening),
        }
    }

    /// Do some amount of rollouts.
    pub fn rollout(&mut self, game: &Game<N>, amount: usize) {
        for _ in 0..amount {
            self.node.rollout(game.clone(), self.agent);
        }
    }

    /// Pick a move to play.
    pub fn pick_move(&mut self, game: &Game<N>, exploitation: bool) -> Turn<N> {
        let turn = self.node.pick_move(exploitation);
        self.play_move(game, &turn);
        turn
    }

    /// Update the search tree, analysis, and create an example.
    pub fn play_move(&mut self, game: &Game<N>, turn: &Turn<N>) {
        self.node.rollout(game.clone(), self.agent); // at least one rollout
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
    pub fn get_examples(self, result: GameResult) -> Vec<Example<N>> {
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
        self.examples
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
}
