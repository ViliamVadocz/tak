use std::collections::HashMap;

use tak::*;

use super::{node::Node, turn_map::Lut};
use crate::{agent::Agent, config::CONTEMPT};

impl<const N: usize> Node<N>
where
    Turn<N>: Lut,
{
    pub fn rollout<A: Agent<N>>(&mut self, game: Game<N>, agent: &A) -> f32 {
        self.visited_count += 1;

        // cache game result
        if self.result.is_none() {
            self.result = Some(game.winner());
            self.expected_reward = match self.result {
                Some(GameResult::Winner { colour: winner, .. }) => {
                    if winner == game.to_move {
                        // means that the previous player played a losing move
                        -1.
                    } else {
                        1.
                    }
                }
                Some(GameResult::Draw { .. }) => -CONTEMPT,
                _ => 0.,
            };
        }
        if let Some(GameResult::Winner { .. }) = self.result {
            return -self.expected_reward;
        } else if let Some(GameResult::Draw { .. }) = self.result {
            return 0.;
        }

        // if it is the first time we are vising this node
        // initialize all children
        if self.children.is_none() {
            return self.expand_node(game, agent);
        }
        // otherwise we have been at this node before
        self.rollout_next(game, agent)
    }

    fn expand_node<A: Agent<N>>(&mut self, game: Game<N>, agent: &A) -> f32 {
        // use the neural network to get initial policy for children
        // and eval for this board
        let (policy, eval) = agent.policy_and_eval(&game);

        let mut children = HashMap::new();

        let turns = game.possible_turns();
        for turn in turns {
            let move_index = turn.turn_map();
            children.insert(turn, Node::init(policy[move_index]));
        }

        self.expected_reward = -eval;
        self.children = Some(children);
        eval
    }

    fn rollout_next<A: Agent<N>>(&mut self, mut game: Game<N>, agent: &A) -> f32 {
        // pick which node to rollout
        let mut children = self.children.take().unwrap();
        let (turn, next_node) = children
            .iter_mut()
            .max_by(|(_, a), (_, b)| {
                self.upper_confidence_bound(a)
                    .partial_cmp(&self.upper_confidence_bound(b))
                    .expect("tried comparing nan")
            })
            .unwrap();

        // rollout next node
        game.play(turn.clone()).unwrap();
        let eval = next_node.rollout(game, agent);
        self.children = Some(children);

        // take the mean of the expected reward and eval
        self.expected_reward =
            ((self.visited_count - 1) as f32 * self.expected_reward + eval) / (self.visited_count as f32);

        -eval
    }
}
