use std::{error::Error, fmt::Display, str::FromStr};

use tak::{takparse::Tps, *};
use tch::Tensor;

use crate::{
    repr::{game_repr, output_size, possible_moves},
    search::move_index,
};

#[derive(Debug)]
pub struct IncompleteExample<const N: usize> {
    pub game: Game<N>,
    pub policy: Vec<(Move, u32)>,
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
    pub policy: Vec<(Move, u32)>,
    pub result: f32,
}

impl<const N: usize> Example<N> {
    fn empty_pi() -> [Vec<f32>; 8] {
        if N == 5 {
            [
                vec![0.; possible_moves(N)],
                vec![0.; possible_moves(N)],
                vec![0.; possible_moves(N)],
                vec![0.; possible_moves(N)],
                vec![0.; possible_moves(N)],
                vec![0.; possible_moves(N)],
                vec![0.; possible_moves(N)],
                vec![0.; possible_moves(N)],
            ]
        } else {
            [
                vec![0.; output_size(N)],
                vec![0.; output_size(N)],
                vec![0.; output_size(N)],
                vec![0.; output_size(N)],
                vec![0.; output_size(N)],
                vec![0.; output_size(N)],
                vec![0.; output_size(N)],
                vec![0.; output_size(N)],
            ]
        }
    }

    pub fn to_tensors(&self) -> Vec<(Tensor, Tensor, f32)> {
        let mut pi = Self::empty_pi();
        let total = self.policy.iter().map(|(_, c)| c).sum::<u32>() as f32;
        for (m, value) in &self.policy {
            for (i, symm) in Symmetry::<N>::symmetries(*m).into_iter().enumerate() {
                pi[i][move_index(&symm, N)] = *value as f32 / total;
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

impl<const N: usize> Display for Example<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{};{};{};{};{};{};{};{}",
            Tps::from(self.game.clone()),
            self.game.white_stones,
            self.game.white_caps,
            self.game.black_stones,
            self.game.black_caps,
            self.game.half_komi,
            self.result,
            self.policy
                .iter()
                .map(|(mov, visits)| format!("{mov}:{visits}"))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl<const N: usize> FromStr for Example<N> {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.trim().split(';');

        let tps = iter.next().ok_or("missing tps")?.parse::<Tps>()?;
        let mut game: Game<N> = tps.into();

        game.white_stones = iter.next().ok_or("missing white stones")?.parse()?;
        game.white_caps = iter.next().ok_or("missing white caps")?.parse()?;
        game.black_stones = iter.next().ok_or("missing black stones")?.parse()?;
        game.black_caps = iter.next().ok_or("missing black caps")?.parse()?;
        game.half_komi = iter.next().ok_or("missing half komi")?.parse()?;

        let result = iter.next().ok_or("missing result")?.parse()?;

        fn parse_pair(pair: &str) -> Result<(Move, u32), Box<dyn Error>> {
            let (move_str, visit_str) = pair.split_once(':').ok_or("pair has missing delimiter")?;
            Ok((move_str.parse()?, visit_str.parse()?))
        }

        let policy = iter
            .next()
            .ok_or("missing policy")?
            .split(',')
            .map(parse_pair)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Example { game, result, policy })
    }
}
