use std::collections::HashMap;

use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng,
};
use tak::{
    game::{Game, GameResult},
    turn::Turn,
};

use crate::{agent::Agent, turn_map::Lut};

const EXPLORATION: f32 = 1.0;

fn upper_confidence_bound<const N: usize>(parent: &Node<N>, child: &Node<N>) -> f32 {
    // U(s, a) = Q(s, a) + c * P(s, a) * sqrt(sum_b(N(s, b))) / (1 + N(s, a))
    child.expected_reward
        + EXPLORATION
            * child.policy
            * ((parent.visited_count as f32).sqrt() / (1.0 + child.visited_count as f32))
}

#[derive(Clone, Debug, Default)]
pub struct Node<const N: usize> {
    result: Option<GameResult>,
    policy: f32,
    expected_reward: f32,
    visited_count: u32,
    children: Option<HashMap<Turn<N>, Node<N>>>,
}

impl<const N: usize> Node<N>
where
    Turn<N>: Lut,
{
    pub fn init(policy: f32) -> Self {
        Node {
            policy,
            ..Default::default()
        }
    }

    pub fn rollout<A: Agent<N>>(&mut self, game: Game<N>, agent: &A) -> f32 {
        self.visited_count += 1;

        // cache game result
        if self.result.is_none() {
            self.result = Some(game.winner());
        }
        match self.result {
            Some(GameResult::Winner(winner)) => {
                return {
                    if winner == game.to_move {
                        // means that the previous player played a losing move
                        -1.
                    } else {
                        1.
                    }
                };
            }
            Some(GameResult::Draw) => return 0.,
            _ => {}
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
        -eval
    }

    fn rollout_next<A: Agent<N>>(&mut self, mut game: Game<N>, agent: &A) -> f32 {
        // pick which node to rollout
        let mut children = self.children.take().unwrap();
        let (turn, next_node) = children
            .iter_mut()
            .max_by(|(_, a), (_, b)| {
                upper_confidence_bound(self, a)
                    .partial_cmp(&upper_confidence_bound(self, b))
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

    pub fn improved_policy(&self) -> HashMap<Turn<N>, f32> {
        let mut policy = HashMap::new();
        // after many rollouts the visited counts become a better estimate for policy
        for (turn, child) in self.children.as_ref().unwrap() {
            let p = child.visited_count as f32 / self.visited_count as f32;
            policy.insert(turn.clone(), p);
        }
        policy
    }

    #[must_use]
    pub fn play(self, turn: &Turn<N>) -> Node<N> {
        let mut children = self.children.expect("do at least one rollout");
        children.remove(turn).expect("all turns should be in there")
    }

    pub fn pick_move(&self, exploitation: bool) -> Turn<N> {
        let improved_policy = self.improved_policy();

        if exploitation {
            // when exploiting always pick the move with largest policy
            improved_policy
                .into_iter()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("tried comparing nan"))
                .unwrap()
                .0
        } else {
            // split into turns and weights
            let mut turns = vec![];
            let mut weights = vec![];
            for (turn, weight) in improved_policy {
                turns.push(turn);
                weights.push(weight);
            }
            // randomly pick based on weights from improved policy
            let mut rng = thread_rng();
            let distr = WeightedIndex::new(&weights).unwrap();
            let index = distr.sample(&mut rng);
            turns.swap_remove(index)
        }
    }
}
