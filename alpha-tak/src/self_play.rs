use rand::random;
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

const SELF_PLAY_GAMES: u32 = 100;
const ROLLOUTS_PER_MOVE: u32 = 100;
const PIT_GAMES: u32 = 50;
const WIN_RATE_THRESHOLD: f64 = 0.55;

fn self_play<const N: usize>(network: &Network<N>) -> Vec<Example<N>>
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: LUT,
{
    let mut examples = Vec::new();

    // run multiple games against self
    for i in 0..SELF_PLAY_GAMES {
        println!("self_play game: {}", i);
        let mut game_examples = Vec::new();
        // TODO add komi?
        let mut game = Game::default();
        game.opening(random()).unwrap();

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
        println!("{:?} in {} plies\n{}", game.winner(), game.ply, game.board);
        // complete examples
        let result = match game.winner() {
            GameResult::Winner(Colour::White) => 1.,
            GameResult::Winner(Colour::Black) => -1.,
            GameResult::Draw => 0.,
            GameResult::Ongoing => unreachable!(),
        };
        // fill in examples with result
        examples.extend(game_examples.into_iter().enumerate().flat_map(|(ply, ex)| {
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
        // (self.wins as f64 + self.draws as f64 / 2.) / (self.wins + self.draws +
        // self.losses) as f64
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
        game.opening(random()).unwrap();

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
        let game_result = game.winner();
        match game_result {
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
        println!(
            "{:?} as {:?} [{}/{}/{}]\n{}",
            game_result, my_colour, wins, draws, losses, game.board
        );
    }

    PitResult { wins, draws, losses }
}

pub fn play_until_better<const N: usize>(network: Network<N>) -> Network<N>
where
    [[Option<Tile>; N]; N]: Default,
    Turn<N>: LUT,
{
    println!("starting a new iteration of self-play");
    // copy network values by file (UGLY)
    let mut dir = std::env::temp_dir();
    dir.push("model");
    network.save(&dir).unwrap();
    let mut new_network = Network::<N>::load(&dir).unwrap();

    loop {
        let examples = self_play(&network);
        new_network.train(&examples);
        let results = pit(&new_network, &network);
        println!("{:?}", results);
        if results.win_rate() > WIN_RATE_THRESHOLD {
            return new_network;
        }
    }
}
