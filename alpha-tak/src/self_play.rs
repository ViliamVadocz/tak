use tak::{
    colour::Colour,
    game::{Game, GameResult},
    tile::Tile,
    turn::Turn,
};

use crate::{
    example::{Example, IncompleteExample},
    mcts::Node,
    network::Network,
    turn_map::LUT,
};

const GAMES_PER_BATCH: u32 = 240;
const ROLLOUTS_PER_MOVE: u32 = 100;
const PIT_GAMES: u32 = 240;
const WIN_RATE_THRESHOLD: f64 = 0.55;

fn self_play<const N: usize>(network: &Network<N>) -> Vec<Example<N>>
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: LUT,
{
    let mut examples = Vec::new();

    // run multiple games against self
    for i in 0..GAMES_PER_BATCH {
        println!("self_play game: {}", i);
        let mut game_examples = Vec::new();
        // TODO add komi?
        let mut game = Game::default();
        game.opening(i as usize).unwrap();

        let mut node = Node::default();
        // play game
        while matches!(game.winner(), GameResult::Ongoing) {
            for _ in 0..ROLLOUTS_PER_MOVE {
                node.rollout(game.clone(), network);
            }
            game_examples.push(IncompleteExample {
                game: game.clone(),
                policy: node.improved_policy(),
            });
            let turn = node.pick_move();
            node = node.play(&turn);
            game.play(turn).unwrap();
        }
        println!("{:?}\n{}", game.winner(), game.board);
        // complete examples
        let result = match game.winner() {
            GameResult::Winner(Colour::White) => 1.,
            GameResult::Winner(Colour::Black) => -1.,
            GameResult::Draw => 0.,
            GameResult::Ongoing => unreachable!(),
        };
        // fill in examples with result
        examples.extend(game_examples.into_iter().enumerate().map(|(ply, ex)| {
            let perspective = if ply % 2 == 0 { result } else { -result };
            ex.complete(perspective)
        }));
    }
    examples
}

#[derive(Debug)]
struct PitResult {
    wins: u32,
    #[allow(dead_code)]
    draws: u32,
    losses: u32,
}

impl PitResult {
    pub fn win_rate(&self) -> f64 {
        self.wins as f64 / (self.wins + self.losses) as f64
    }
}

/// pits two networks against each other
/// counts wins and losses of the new network
fn pit<const N: usize>(new: &Network<N>, old: &Network<N>) -> PitResult
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: LUT,
{
    println!("pitting two networks against each other");
    let mut wins = 0;
    let mut draws = 0;
    let mut losses = 0;

    for i in 0..PIT_GAMES {
        println!("pit game: {}", i);
        // TODO add komi?
        let mut game = Game::default();
        game.opening(i as usize / 2).unwrap();

        let my_colour = if i % 2 == 0 { Colour::White } else { Colour::Black };
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
                my_node.pick_move()
            } else {
                for _ in 0..ROLLOUTS_PER_MOVE {
                    opp_node.rollout(game.clone(), old);
                }
                opp_node.pick_move()
            };
            my_node = my_node.play(&turn);
            opp_node = opp_node.play(&turn);
            game.play(turn).unwrap();
        }
        println!("{:?}\n{}", game.winner(), game.board);
        match game.winner() {
            GameResult::Winner(winner) => {
                if winner == my_colour {
                    wins += 1;
                } else {
                    losses += 1;
                }
            }
            GameResult::Draw => draws += 1,
            GameResult::Ongoing => unreachable!(),
        }
    }

    PitResult { wins, draws, losses }
}

pub fn play_until_better<const N: usize>(network: Network<N>) -> Network<N>
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: LUT,
{
    loop {
        println!("starting a new iteration of self-play");
        let examples = self_play(&network);
        let mut new_network = Network::<N>::default(); // each time start fresh?
        new_network.train(examples);
        let results = pit(&new_network, &network);
        println!("{:?}", results);
        if results.win_rate() > WIN_RATE_THRESHOLD {
            return new_network;
        } else {
            println!("discarding changes");
        }
    }
}
