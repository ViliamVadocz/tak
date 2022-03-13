use alpha_tak::{
    config::{MAX_EXAMPLES, N, WIN_RATE_THRESHOLD},
    example::{save_examples, Example},
    model::network::Network,
    sys_time,
};

use crate::{pit::pit, self_play::self_play, EXAMPLE_DIR, MODEL_DIR};

pub fn training_loop(mut network: Network<N>, mut examples: Vec<Example<N>>) -> ! {
    loop {
        if !examples.is_empty() {
            let new_network = {
                let mut nn = copy(&network);
                nn.train(&examples);
                nn
            };

            println!("pitting two networks against each other");
            let (results, more_examples) = pit(&new_network, &network);
            save_examples(&more_examples, format!("{EXAMPLE_DIR}/pit_{}.data", sys_time()));
            examples.extend(more_examples.into_iter());

            println!("{:?}", results);
            if results.win_rate() > WIN_RATE_THRESHOLD {
                network = new_network;
                println!("saving model");
                network.save(format!("{MODEL_DIR}/{}.model", sys_time())).unwrap();
            }
        }

        // do self-play to get new examples
        println!("starting self-play");
        let new_examples = self_play(&network);
        save_examples(&new_examples, format!("{EXAMPLE_DIR}/{}.data", sys_time()));

        // keep only the latest MAX_EXAMPLES examples
        examples.extend(new_examples.into_iter());
        if examples.len() > MAX_EXAMPLES {
            examples.reverse();
            examples.truncate(MAX_EXAMPLES);
            examples.reverse();
        }
    }
}

fn copy<const N: usize>(network: &Network<N>) -> Network<N> {
    // copy network values by file (ugly but works)
    let mut dir = std::env::temp_dir();
    dir.push("model");
    network.save(&dir).unwrap();
    Network::<N>::load(&dir).unwrap()
}
