use takparse::{Direction, Move, MoveKind, Square};

use crate::{board::Board, game::Game};

pub trait Symmetry<const N: usize>: Sized {
    fn symmetries(self) -> [Self; 8];
}

impl<const N: usize> Symmetry<N> for Square {
    fn symmetries(self) -> [Self; 8] {
        let n = N as u8;
        [
            self,
            self.rotate(n),
            self.rotate(n).rotate(n),
            self.rotate(n).rotate(n).rotate(n),
            self.mirror(n),
            self.mirror(n).rotate(n),
            self.mirror(n).rotate(n).rotate(n),
            self.mirror(n).rotate(n).rotate(n).rotate(n),
        ]
    }
}

impl<const N: usize> Symmetry<N> for Direction {
    fn symmetries(self) -> [Self; 8] {
        [
            self,
            self.rotate(),
            self.rotate().rotate(),
            self.rotate().rotate().rotate(),
            self.mirror(),
            self.mirror().rotate(),
            self.mirror().rotate().rotate(),
            self.mirror().rotate().rotate().rotate(),
        ]
    }
}

impl<const N: usize> Symmetry<N> for Move {
    fn symmetries(self) -> [Self; 8] {
        let square = self.square();
        let kind = self.kind();
        match kind {
            MoveKind::Place(_) => {
                Symmetry::<N>::symmetries(square).map(|square| Self::new(square, kind))
            }
            MoveKind::Spread(direction, pattern) => zip(
                Symmetry::<N>::symmetries(square),
                Symmetry::<N>::symmetries(direction),
            )
            .map(|(square, direction)| Self::new(square, MoveKind::Spread(direction, pattern))),
        }
    }
}

impl<const N: usize> Symmetry<N> for Board<N> {
    fn symmetries(self) -> [Self; 8] {
        let mut boards = [self; 8];
        for (x, row) in self.iter().enumerate() {
            for (y, &stack) in row.enumerate() {
                let square = Square::new(x as u8, y as u8);
                for (i, sym) in Symmetry::<N>::symmetries(square)
                    .into_iter()
                    .enumerate()
                    .skip(1)
                {
                    // Unwrap is sound because the square is guaranteed to be on the board.
                    unsafe {
                        *boards[i].get_mut(sym).unwrap_unchecked() = stack;
                    }
                }
            }
        }
        boards
    }
}

impl<const N: usize, const HALF_KOMI: i8> Symmetry<N> for Game<N, HALF_KOMI> {
    fn symmetries(self) -> [Self; 8] {
        let mut games = [self; 8];
        for (i, board) in self.board.symmetries().into_iter().enumerate().skip(1) {
            games[i].board = board;
        }
        games
    }
}

#[inline]
fn zip<const N: usize, A: Copy, B: Copy>(a: [A; N], b: [B; N]) -> [(A, B); N] {
    let mut i = 0;
    [(); N].map(|()| {
        let r = (a[i], b[i]);
        i += 1;
        r
    })
}

impl<const N: usize, const HALF_KOMI: i8> Game<N, HALF_KOMI> {
    pub fn canonical(mut self) -> (usize, Self) {
        let (i, board) = self
            .board
            .symmetries()
            .into_iter()
            .enumerate()
            .min_by_key(|(_, board)| *board)
            .unwrap();
        self.board = board;
        (i, self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{reserves::Reserves, symm::Symmetry, Game, GameResult, PlayError};

    fn symmetrical_boards<const N: usize>(seed: usize) -> Result<(), PlayError>
    where
        Reserves<N>: Default,
    {
        let [mut g0, mut g1, mut g2, mut g3, mut g4, mut g5, mut g6, mut g7] =
            Game::<N, 0>::default().symmetries();
        let mut moves = Vec::new();
        while matches!(g0.result(), GameResult::Ongoing) {
            moves.clear();
            g0.possible_moves(&mut moves);
            let my_move = moves[seed % moves.len()];
            let [t0, t1, t2, t3, t4, t5, t6, t7] = Symmetry::<N>::symmetries(my_move);
            g0.play(t0)?;
            g1.play(t1)?;
            g2.play(t2)?;
            g3.play(t3)?;
            g4.play(t4)?;
            g5.play(t5)?;
            g6.play(t6)?;
            g7.play(t7)?;
        }
        assert_eq!(g0.result(), g1.result());
        assert_eq!(g1.result(), g2.result());
        assert_eq!(g2.result(), g3.result());
        assert_eq!(g4.result(), g5.result());
        assert_eq!(g5.result(), g6.result());
        assert_eq!(g6.result(), g7.result());
        Ok(())
    }

    macro_rules! symmetrical_boards_seeded {
        [$($name:ident $seed:literal),*] => {
            $(
                #[test]
                fn $name() {
                    symmetrical_boards::<3>($seed).unwrap();
                    symmetrical_boards::<4>($seed).unwrap();
                    symmetrical_boards::<5>($seed).unwrap();
                    symmetrical_boards::<6>($seed).unwrap();
                    symmetrical_boards::<7>($seed).unwrap();
                    symmetrical_boards::<8>($seed).unwrap();
                }
            )*
        };
    }

    symmetrical_boards_seeded![
        symmetrical_boards_5915587277 5_915_587_277,
        symmetrical_boards_1500450271 1_500_450_271,
        symmetrical_boards_3267000013 3_267_000_013,
        symmetrical_boards_5754853343 5_754_853_343,
        symmetrical_boards_4093082899 4_093_082_899,
        symmetrical_boards_9576890767 9_576_890_767,
        symmetrical_boards_3628273133 3_628_273_133,
        symmetrical_boards_2860486313 2_860_486_313,
        symmetrical_boards_5463458053 5_463_458_053,
        symmetrical_boards_3367900313 3_367_900_313
    ];
}
