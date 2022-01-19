use std::collections::HashMap;

use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng,
};
use tak::{
    game::{Game, GameResult},
    turn::Turn,
};

use crate::{network::Network, turn_map::LUT};

const EXPLORATION: f32 = 0.5;

fn upper_confidence_bound<const N: usize>(parent: &Node<N>, child: &Node<N>) -> f32 {
    // U(s, a) = Q(s, a) + c * P(s, a) * sqrt(sum_b(N(s, b))) / (1 + N(s, a))
    child.expected_reward
        + EXPLORATION * child.policy * (parent.visited_count as f32).sqrt()
            / (1.0 + child.visited_count as f32)
}

#[derive(Clone, Debug, Default)]
pub struct Node<const N: usize> {
    policy: f32,
    expected_reward: f32,
    visited_count: u32,
    children: Option<HashMap<Turn<N>, Node<N>>>,
}

impl<const N: usize> Node<N>
where
    Turn<N>: LUT,
{
    pub fn init(policy: f32) -> Self {
        Node {
            policy,
            expected_reward: 0.0,
            visited_count: 0,
            children: None,
        }
    }

    // TODO perspective of other player? is it correctly flipped?
    // is the best move picked for each player (maximizer vs minimizer)?
    pub fn rollout(&mut self, mut game: Game<N>, network: &Network<N>) -> f32 {
        self.visited_count += 1;
        let result = game.winner();
        if !matches!(result, GameResult::Ongoing) {
            return match result {
                GameResult::Winner(winner) => {
                    if winner == game.to_move {
                        1.
                    } else {
                        -1.
                    }
                }
                GameResult::Draw => 0.,
                GameResult::Ongoing => unreachable!(),
            };
        }

        // if it is the first time we are vising this node
        // initialize all children
        if self.children.is_none() {
            // use the neural network to get initial policy for children
            // and eval for this board
            let (policy, eval) = network.predict(&game, false);
            let policy: Vec<_> = policy.exp().into(); // TODO undoing log?
            let eval: f32 = eval.into();

            let mut children = HashMap::new();

            let turns = game.move_gen();
            for turn in turns {
                let move_index = turn.turn_map();
                children.insert(turn, Node::init(policy[move_index]));
            }

            self.expected_reward = eval;
            self.children = Some(children);
            return -eval;
        }

        // otherwise we have been at this node before
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
        let eval = next_node.rollout(game, network);
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

    pub fn pick_move(&self) -> Turn<N> {
        let improved_policy = self.improved_policy();
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
