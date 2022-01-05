pub mod board;
pub mod colour;
pub mod game;
pub mod pos;
pub mod tile;
pub mod turn;

pub type StrResult<T> = Result<T, &'static str>;

#[cfg(test)]
mod tests {
    use crate::{
        game::{Game, GameResult},
        StrResult,
    };

    #[test]
    fn always_last_move() -> StrResult<()> {
        let mut game = Game::<6>::default();
        while let GameResult::Ongoing = game.winner() {
            if game.ply > 1000 {
                break;
            }
            let mut moves = game.move_gen();
            game.play(moves.pop().unwrap())?;
        }
        Ok(())
    }

    #[test]
    fn always_first_move() -> StrResult<()> {
        let mut game = Game::<6>::default();
        while let GameResult::Ongoing = game.winner() {
            if game.ply > 1000 {
                break;
            }
            let mut moves = game.move_gen();
            game.play(moves.swap_remove(0))?;
        }
        Ok(())
    }

    fn perf_count<const N: usize>(game: Game<N>, depth: usize) -> usize {
        if depth == 0 || !matches!(game.winner(), GameResult::Ongoing) {
            1
        } else if depth == 1 {
            game.move_gen().into_iter().len()
        } else {
            game.move_gen()
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
    fn perft_numbers() {
        assert_eq!(perf_count(Game::<5>::default(), 0), 1);
        assert_eq!(perf_count(Game::<5>::default(), 1), 25);
        assert_eq!(perf_count(Game::<5>::default(), 2), 600);
        assert_eq!(perf_count(Game::<5>::default(), 3), 43_320);
        assert_eq!(perf_count(Game::<5>::default(), 4), 2_999_784); // FIXME
    }
}
