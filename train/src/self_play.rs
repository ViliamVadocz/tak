use alpha_tak::{Example, Network, Player};
use rand::{prelude::SliceRandom, thread_rng};
use tak::*;

const SELF_PLAY_GAMES: u32 = 500;
const BATCH_SIZE: u32 = 64;
const ROLLOUTS: u32 = 15;

const NOISE_ALPHA: f32 = 0.2;
const NOISE_RATIO: f32 = 0.5;
const NOISE_PLIES: u16 = 50;

const RANDOM_PLIES: u32 = 2;
const EXPLOIT_PLIES: u16 = 30;

pub fn self_play<const N: usize, NET: Network<N>>(network: &NET) -> Vec<Example<N>> {
    let mut examples = Vec::new();

    let mut example_file = File::create(format!("{EXAMPLE_DIR}/{}.data", sys_time())).unwrap();

    // TODO parallel batches, create new kind of player?
    let mut rng = thread_rng();
    for i in 0..SELF_PLAY_GAMES {
        println!("self_play game {i}/{SELF_PLAY_GAMES}");
        let mut game = Game::with_komi(2);
        let mut player = Player::new(network, BATCH_SIZE, true, false, &game);

        // Do random opening.
        for _ in 0..RANDOM_PLIES {
            let my_move = *game.possible_moves().choose(&mut rng).unwrap();
            player.play_move(&game, my_move, false);
            game.play(my_move).unwrap();
        }

        while game.result() == GameResult::Ongoing {
            if game.ply < NOISE_PLIES {
                player.add_noise(NOISE_ALPHA, NOISE_RATIO);
            }
            for _ in 0..ROLLOUTS {
                player.rollout(&game);
            }
            let my_move = player.pick_move(game.ply >= EXPLOIT_PLIES);
            player.play_move(&game, my_move, true);
            game.play(my_move).unwrap();
        }
        println!("{:?} in {} plies", game.result(), game.ply);

        // Save examples as we go to a file.
        let new_examples = player.get_examples(game.result());
        for example in &new_examples {
            writeln!(example_file, "{example}").unwrap();
        }
        example_file.flush().unwrap();
        // Save examples to output vector.
        examples.extend(new_examples.into_iter());
    }

    examples
}
