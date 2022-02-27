use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{Read, Write},
};

use tak::*;
use tch::Tensor;

use crate::{
    repr::{game_repr, moves_dims},
    search::turn_map::Lut,
    sys_time,
};

#[derive(Debug)]
pub struct IncompleteExample<const N: usize> {
    pub game: Game<N>,
    pub policy: HashMap<Turn<N>, u32>,
}

impl<const N: usize> IncompleteExample<N> {
    #[must_use]
    pub fn complete(self, result: f32) -> Example<N> {
        Example {
            game: self.game,
            policy: self.policy,
            result,
        }
    }
}

#[derive(Debug)]
pub struct Example<const N: usize> {
    pub game: Game<N>,
    pub policy: HashMap<Turn<N>, u32>,
    pub result: f32,
}

impl<const N: usize> Example<N>
where
    Turn<N>: Lut,
    [[Option<Tile>; N]; N]: Default,
{
    pub fn to_tensors(&self) -> Vec<(Tensor, Tensor, f32)> {
        let mut pi = [
            vec![0.; moves_dims(N)],
            vec![0.; moves_dims(N)],
            vec![0.; moves_dims(N)],
            vec![0.; moves_dims(N)],
            vec![0.; moves_dims(N)],
            vec![0.; moves_dims(N)],
            vec![0.; moves_dims(N)],
            vec![0.; moves_dims(N)],
        ];
        let total = self.policy.iter().map(|(_, c)| c).sum::<u32>() as f32;
        for (turn, &value) in self.policy.iter() {
            for (i, symm) in turn.clone().symmetries().into_iter().enumerate() {
                pi[i][symm.turn_map()] = value as f32 / total;
            }
        }

        self.game
            .clone()
            .symmetries()
            .into_iter()
            .enumerate()
            .map(|(i, game)| (game_repr(&game), Tensor::of_slice(&pi[i]), self.result))
            .collect()
    }
}

pub fn save_examples<const N: usize>(examples: &[Example<N>]) {
    if let Ok(mut file) = File::create(format!("examples/{}.data", sys_time())) {
        let out = examples
            .iter()
            .map(|example| {
                format!(
                    "{};{};{}\n",
                    example.game.to_tps(),
                    example.result,
                    example
                        .policy
                        .iter()
                        .map(|(turn, visits)| format!("{} {visits},", turn.to_ptn()))
                        .collect::<String>()
                )
            })
            .collect::<String>();
        file.write_all(out.as_bytes()).unwrap();
    }
}

// TODO clean this up
pub fn load_examples<const N: usize>(path: &str) -> Result<Vec<Example<N>>, Box<dyn Error>>
where
    [[Option<Tile>; N]; N]: Default,
{
    let mut file = File::open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;

    s.split_terminator('\n')
        .map(|example| {
            let mut chunks = example.split(';');
            let mut tps = chunks.next().expect("missing board").split(' ');

            // TODO put this ugly code into different functions, clean it up a bit
            // MOVE IT TO FromTPS for Game
            let board = Board::from_tps(tps.next().expect("missing board")).unwrap();
            let to_move = Colour::from_ptn(tps.next().expect("missing to_move")).unwrap();
            let ply = (tps.next().expect("missing move number").parse::<u64>().unwrap() - 1) * 2
                + match to_move {
                    Colour::White => 0,
                    Colour::Black => 1,
                };
            let mut white_reserves = tps.next().expect("missing white reserves").split('/');
            let white_stones = white_reserves.next().unwrap()[1..].parse().unwrap();
            let white_caps = white_reserves.next().unwrap().replace(')', "").parse().unwrap();
            let mut black_reserves = tps.next().expect("missing black reserves").split('/');
            let black_stones = black_reserves.next().unwrap()[1..].parse().unwrap();
            let black_caps = black_reserves.next().unwrap().replace(')', "").parse().unwrap();
            let komi = tps.next().expect("missing komi").parse::<i32>().unwrap();

            let game = Game {
                board,
                to_move,
                white_caps,
                black_caps,
                white_stones,
                black_stones,
                ply,
                komi,
            };

            let result = chunks
                .next()
                .expect("missing result")
                .parse::<f32>()
                .expect("game result cannot be parsed");

            let mut policy = HashMap::new();
            for line in chunks.next().expect("missing turns").split_terminator(',') {
                let mut words = line.split(' ');
                let turn = Turn::from_ptn(words.next().expect("missing turn")).expect("invalid turn");
                let visited = words
                    .next()
                    .expect("missing visited count")
                    .parse::<u32>()
                    .expect("invalid visited count");
                policy.insert(turn, visited);
            }

            Ok(Example { game, policy, result })
        })
        .collect()
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use tak::*;
    use test::Bencher;

    use super::Example;

    #[bench]
    fn to_tensors_bench(b: &mut Bencher) {
        let game = Game::<5>::from_ptn(
            "
            1. a1 e1
            2. c3 Cd3
            3. d4 c4
            4. c2 d2
            5. b4 c5
        ",
        )
        .unwrap();
        let policy = game
            .possible_turns()
            .into_iter()
            .map(|t| (t, 1))
            .collect::<HashMap<Turn<5>, u32>>();
        let example = Example {
            game,
            policy,
            result: 1.0,
        };
        b.iter(|| example.to_tensors())
    }
}
