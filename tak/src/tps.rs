use std::num::NonZeroUsize;

use takparse::{ExtendedSquare, Stack, Tps};

use crate::{Board, Game, Tile};

impl<const N: usize> From<Game<N>> for Tps {
    fn from(game: Game<N>) -> Self {
        let board = game
            .board
            .data
            .into_iter()
            .rev()
            .map(|row| {
                row.into_iter()
                    .map(|tile| {
                        if tile.is_empty() {
                            ExtendedSquare::EmptySquares(1)
                        } else {
                            ExtendedSquare::Stack(Stack::new(tile.piece, tile.stack.into_iter()))
                        }
                    })
                    .collect()
            })
            .collect();

        unsafe {
            Tps::new_unchecked(
                board,
                game.to_move,
                NonZeroUsize::new(1 + game.ply as usize / 2).unwrap(),
            )
        }
    }
}

impl<const N: usize> From<Tps> for Game<N> {
    fn from(tps: Tps) -> Game<N> {
        let mut data = tps
            .board_2d()
            .map(|row| {
                row.map(|square| {
                    if let Some(stack) = square {
                        Tile {
                            piece: stack.top(),
                            stack: stack.colors().collect(),
                        }
                    } else {
                        Tile::default()
                    }
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
            })
            .collect::<Vec<_>>();
        data.reverse();
        let board = Board {
            data: data.try_into().unwrap(),
        };
        Game {
            board,
            to_move: tps.color(),
            ply: tps.ply() as u16,
            ..Default::default()
        }
    }
}
