use std::{
    fs::{create_dir_all, File},
    io::Write,
};

use alpha_tak::{
    agent::Agent,
    analysis::Analysis,
    config::{
        DIRICHLET_NOISE,
        KOMI,
        N,
        NOISE_PLIES,
        NOISE_RATIO,
        ROLLOUTS_PER_MOVE,
        SELF_PLAY_GAMES,
        TEMPERATURE_PLIES,
    },
    example::Example,
    model::network::Network,
    player::Player,
    sys_time,
    threadpool::thread_pool,
};
use tak::*;

use crate::GAME_DIR;

pub fn self_play(network: &Network<N>) -> Vec<Example<N>> {
    const WORKERS: usize = 128;

    let outputs = thread_pool::<N, WORKERS, _, _>(network, SELF_PLAY_GAMES, self_play_game);
    let mut examples = Vec::new();
    let mut analyses = Vec::new();
    for output in outputs {
        examples.extend(output.0.into_iter());
        analyses.push(output.1);
    }

    // TODO Do some opening analysis on the analyses
    let time = sys_time();
    if create_dir_all(format!("{GAME_DIR}/{time}")).is_ok() {
        for (i, analysis) in analyses.into_iter().enumerate() {
            if let Ok(mut file) = File::create(format!("{GAME_DIR}/{time}/{i}.ptn")) {
                file.write_all(analysis.to_ptn().as_bytes()).unwrap();
            }
        }
    }

    examples
}

fn self_play_game<A: Agent<N>>(agent: &A, _index: usize) -> (Vec<Example<N>>, Analysis<N>) {
    let mut game = Game::with_komi(KOMI);

    // TODO proper opening book using index
    let opening = vec![
        Turn::Place {
            pos: Pos { x: 0, y: 0 },
            shape: Shape::Flat,
        },
        Turn::Place {
            pos: Pos {
                x: 4,
                y: if rand::random() { 0 } else { 4 },
            },
            shape: Shape::Flat,
        },
    ];
    for turn in opening.clone() {
        game.play(turn).unwrap()
    }

    let mut player = Player::new(agent, opening, game.komi);

    while matches!(game.winner(), GameResult::Ongoing) {
        if game.ply < NOISE_PLIES {
            player.apply_dirichlet(&game, DIRICHLET_NOISE, NOISE_RATIO);
        }
        player.rollout(&game, ROLLOUTS_PER_MOVE);
        let turn = player.pick_move(&game, game.ply > TEMPERATURE_PLIES);
        game.play(turn).unwrap();
    }

    (player.get_examples(game.winner()), player.get_analysis())
}
