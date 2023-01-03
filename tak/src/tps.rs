use std::num::NonZeroUsize;

use takparse::{Color, ExtendedSquare, Piece, Stack as TpsStack, Tps};

use crate::{reserves::Reserves, Game};

impl<const N: usize, const HALF_KOMI: i8> From<Game<N, HALF_KOMI>> for Tps {
    fn from(game: Game<N, HALF_KOMI>) -> Self {
        let mut board: Vec<_> = game
            .board
            .iter()
            .map(|row| {
                row.map(|stack| match stack.top() {
                    None => ExtendedSquare::EmptySquares(1),
                    Some((piece, _color)) => {
                        ExtendedSquare::Stack(TpsStack::new(piece, stack.colors()))
                    }
                })
                .collect()
            })
            .collect();
        board.reverse();

        unsafe {
            Self::new_unchecked(
                board,
                game.to_move,
                NonZeroUsize::new(1 + usize::from(game.ply) / 2).unwrap_unchecked(),
            )
        }
    }
}

impl<const N: usize, const HALF_KOMI: i8> From<Tps> for Game<N, HALF_KOMI>
where
    Reserves<N>: Default,
{
    fn from(tps: Tps) -> Self {
        let board = tps.board().collect();

        // Figure out how many reserves each player has left.
        let Reserves {
            stones: mut white_stones,
            caps: mut white_caps,
        } = Reserves::<N>::default();
        let Reserves {
            stones: mut black_stones,
            caps: mut black_caps,
        } = Reserves::<N>::default();

        for stack in tps.board().flatten() {
            if stack.top() == Piece::Cap {
                match stack.colors().last() {
                    Some(Color::White) => {
                        white_stones += 1;
                        white_caps -= 1;
                    }
                    Some(Color::Black) => {
                        black_stones += 1;
                        black_caps -= 1;
                    }
                    None => {}
                }
            }
            for color in stack.colors() {
                match color {
                    Color::White => white_stones -= 1,
                    Color::Black => black_stones -= 1,
                }
            }
        }

        Self {
            board,
            to_move: tps.color(),
            ply: tps.ply().try_into().unwrap_or_default(),
            white_reserves: Reserves {
                stones: white_stones,
                caps: white_caps,
            },
            black_reserves: Reserves {
                stones: black_stones,
                caps: black_caps,
            },
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use takparse::Tps;

    use crate::{reserves::Reserves, Game, GameResult, PlayError};

    #[test]
    fn complicated_board() {
        let game = Game::<6, 0>::from_ptn_moves(&[
            "e1", "f2", "Sb5", "Cd6", "d3", "d4", "Sc1", "c3", "Ca6", "f6", "b1", "Sb4", "b3",
            "b2", "d5", "e1>", "d3>", "b2<", "Se2", "f4", "f2-", "c3-", "e4", "Sa5", "c3", "c5",
            "b5>", "a2-", "Sb5", "e6", "2c5-11", "d6>", "d5<", "b2", "b3-", "b3", "e3+", "e6>",
            "a4", "Sf5", "d6", "e6-", "f1+", "d4<", "d3", "d4", "b2>", "e3", "2e4+11", "a1>",
            "2c3>11", "Sc6", "d3-", "e4", "d5", "a2", "d5-", "a2+", "2c2+11", "c2", "d1", "c3>",
            "3c4-", "2d3-11", "Sa2", "c4", "2d2<11", "Sd2", "d3", "b3-", "f2+", "b3", "a1", "e4+",
            "d5", "2e5<11", "2d4>", "2b2>", "d5-", "d2+", "e4+", "d2", "c3<", "c3<", "e2<", "c2+",
            "c2<", "e2", "d5>", "c3<", "b2>", "d5", "d4>", "d5+", "c2<", "d5", "b2-", "d5>", "c2+",
            "b3>", "2d2<", "d2", "3c2+21", "d4", "e4<", "d5", "c2",
        ]);

        let tps: Tps = game.into();
        assert_eq!(
            tps.to_string(),
            "1C,x,2S,12,1,22C/2S,1S,12,2,2112,2S/1,2S,21S,21,2,2/2,212,21222,12S,21S,1/1S,2,1,2,2,\
             x/1,121,1S,12,x,2 2 54"
        );
    }

    fn tps_consistency<const N: usize>(seed: usize) -> Result<(), PlayError>
    where
        Reserves<N>: Default,
    {
        let mut game = Game::<N, 0>::default();
        let mut moves = Vec::new();
        while game.result() == GameResult::Ongoing {
            moves.clear();
            game.possible_moves(&mut moves);
            let my_move = moves[seed % moves.len()];

            println!("{my_move}");
            game.play(my_move)?;

            let tps: Tps = game.into();
            println!("{tps}");
            let tps_game: Game<N, 0> = tps.into();

            assert_eq!(game.board, tps_game.board, "board does not equal");
            assert_eq!(game.to_move, tps_game.to_move, "to_move does not equal");
            assert_eq!(game.ply, tps_game.ply, "ply does not equal");
            assert_eq!(
                game.white_reserves, tps_game.white_reserves,
                "white reserves do not equal"
            );
            assert_eq!(
                game.black_reserves, tps_game.black_reserves,
                "black reserves do not equal"
            );
        }
        Ok(())
    }

    macro_rules! tps_consistency_seeded {
        [$($name:ident $seed:literal),*] => {
            $(
                #[test]
                fn $name() {
                    tps_consistency::<3>($seed).unwrap();
                    tps_consistency::<4>($seed).unwrap();
                    tps_consistency::<5>($seed).unwrap();
                    tps_consistency::<6>($seed).unwrap();
                    tps_consistency::<7>($seed).unwrap();
                    tps_consistency::<8>($seed).unwrap();
                }
            )*
        };
    }

    tps_consistency_seeded![
        tps_consistency_5915587277 5_915_587_277,
        tps_consistency_1500450271 1_500_450_271,
        tps_consistency_3267000013 3_267_000_013,
        tps_consistency_5754853343 5_754_853_343,
        tps_consistency_4093082899 4_093_082_899,
        tps_consistency_9576890767 9_576_890_767,
        tps_consistency_3628273133 3_628_273_133,
        tps_consistency_2860486313 2_860_486_313,
        tps_consistency_5463458053 5_463_458_053,
        tps_consistency_3367900313 3_367_900_313
    ];
}
