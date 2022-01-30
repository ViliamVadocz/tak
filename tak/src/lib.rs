#![feature(array_zip)]

#[macro_use]
extern crate lazy_static;

pub mod board;
pub mod colour;
pub mod direction;
pub mod game;
pub mod pos;
pub mod ptn;
pub mod symm;
pub mod tile;
pub mod turn;

pub type StrResult<T> = Result<T, String>;

#[cfg(test)]
mod tests {
    use crate::{
        colour::Colour,
        game::{Game, GameResult},
        ptn::FromPTN,
        StrResult,
    };

    #[test]
    fn always_last_move() -> StrResult<()> {
        let mut game = Game::<6>::default();
        while let GameResult::Ongoing = game.winner() {
            let mut moves = game.possible_turns();
            println!("{}", game.board);
            let tried = format!("{:?}", moves.last().unwrap());
            if game.play(moves.pop().unwrap()).is_err() {
                panic!("{}", tried)
            };
        }
        Ok(())
    }

    #[test]
    fn always_first_move() -> StrResult<()> {
        let mut game = Game::<6>::default();
        while let GameResult::Ongoing = game.winner() {
            let mut moves = game.possible_turns();
            game.play(moves.swap_remove(0))?;
        }
        Ok(())
    }

    fn perf_count<const N: usize>(game: Game<N>, depth: usize) -> usize {
        if depth == 0 || !matches!(game.winner(), GameResult::Ongoing) {
            1
        } else if depth == 1 {
            game.possible_turns().len()
        } else {
            game.possible_turns()
                .into_iter()
                .map(|turn| {
                    let mut clone = game.clone();
                    clone.play(turn).unwrap();
                    perf_count(clone, depth - 1)
                })
                .sum()
        }
    }

    #[test]
    fn position1_perft() -> StrResult<()> {
        let mut game = Game::<5>::default();
        game.play_ptn_moves(&["d3", "c3", "c4", "1d3<", "1c4-", "Sc4"])?;
        assert_eq!(perf_count(game.clone(), 1), 87);
        assert_eq!(perf_count(game.clone(), 2), 6_155);
        assert_eq!(perf_count(game, 3), 461_800);
        Ok(())
    }

    #[test]
    fn position2_perft() -> StrResult<()> {
        let mut game = Game::<5>::default();
        game.play_ptn_moves(&[
            "c2", "c3", "d3", "b3", "c4", "1c2+", "1d3<", "1b3>", "1c4-", "Cc2", "a1", "1c2+", "a2",
        ])?;
        assert_eq!(perf_count(game.clone(), 1), 104);
        assert_eq!(perf_count(game.clone(), 2), 7_743);
        assert_eq!(perf_count(game, 3), 592_645);
        Ok(())
    }

    #[test]
    fn position3_perft() -> StrResult<()> {
        let mut game = Game::<5>::default();
        game.play_ptn_moves(&[
            "c4", "c2", "d2", "c3", "b2", "d3", "1d2+", "b3", "d2", "b4", "1c2+", "1b3>", "2d3<", "1c4-",
            "d4", "5c3<23", "c2", "c4", "1d4<", "d3", "1d2+", "1c3+", "Cc3", "2c4>", "1c3<", "d2", "c3",
            "1d2+", "1c3+", "1b4>", "2b3>11", "3c4-12", "d2", "c4", "b4", "c5", "1b3>", "1c4<", "3c3-", "e5",
            "e2",
        ])?;
        assert_eq!(perf_count(game.clone(), 1), 85);
        assert_eq!(perf_count(game.clone(), 2), 11_206);
        assert_eq!(perf_count(game, 3), 957_000);
        Ok(())
    }

    #[test]
    fn perft_5() {
        assert_eq!(perf_count(Game::<5>::default(), 0), 1);
        assert_eq!(perf_count(Game::<5>::default(), 1), 25);
        assert_eq!(perf_count(Game::<5>::default(), 2), 600);
        assert_eq!(perf_count(Game::<5>::default(), 3), 43_320);
        assert_eq!(perf_count(Game::<5>::default(), 4), 2_999_784);
    }

    #[test]
    fn perft_6() {
        assert_eq!(perf_count(Game::<6>::default(), 0), 1);
        assert_eq!(perf_count(Game::<6>::default(), 1), 36);
        assert_eq!(perf_count(Game::<6>::default(), 2), 1_260);
        assert_eq!(perf_count(Game::<6>::default(), 3), 132_720);
        assert_eq!(perf_count(Game::<6>::default(), 4), 13_586_048);
        // assert_eq!(perf_count(Game::<6>::default(), 5), 1_253_506_520);
    }

    #[test]
    fn double_road_correct_win() -> StrResult<()> {
        let game = Game::<6>::from_ptn(
            "1. a4 a3
            2. b3 b4
            3. c3 c4
            4. d3 d4
            5. d3+ e4
            6. e3 f4
            7. f3 Cb5
            8. d4-",
        )?;
        assert_eq!(game.winner(), GameResult::Winner(Colour::White));
        Ok(())
    }

    #[test]
    fn flat_win() -> StrResult<()> {
        let game = Game::<3>::from_ptn(
            "1. a3 c1
            2. c2 c3
            3. b3 b2
            4. b1 a1
            5. a2 F-0",
        )?;
        assert_eq!(game.winner(), GameResult::Winner(Colour::White));
        Ok(())
    }
}
