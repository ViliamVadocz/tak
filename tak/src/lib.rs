pub mod board;
pub mod colour;
pub mod game;
pub mod pos;
pub mod tile;
pub mod turn;

pub type StrResult<T> = Result<T, &'static str>;

#[cfg(test)]
mod tests {
    use arrayvec::ArrayVec;

    use crate::{
        colour::Colour,
        game::{Game, GameResult},
        pos::Pos,
        tile::{Piece, Shape},
        turn::Turn,
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
            game.move_gen().len()
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

    fn place<const N: usize>(x: usize, y: usize, colour: Colour, shape: Shape) -> Turn<N> {
        Turn::Place {
            pos: Pos { x, y },
            piece: Piece { colour, shape },
        }
    }

    fn move_<const N: usize>(
        x: usize,
        y: usize,
        colour: Colour,
        shape: Shape,
        x2: usize,
        y2: usize,
    ) -> Turn<N> {
        let mut drops = ArrayVec::new();
        drops.push((Pos { x: x2, y: y2 }, Piece { colour, shape }));
        Turn::Move {
            pos: Pos { x, y },
            drops,
        }
    }

    #[test]
    fn correct_move() -> StrResult<()> {
        let mut game = Game::<5>::default();
        game.play(place(2, 2, Colour::White, Shape::Flat))?;
        game.play(place(3, 2, Colour::Black, Shape::Flat))?;
        game.play(place(2, 1, Colour::White, Shape::Flat))?;
        game.play(move_(3, 2, Colour::Black, Shape::Flat, 2, 2))?;
        game.play(move_(2, 1, Colour::White, Shape::Flat, 2, 2))?;
        game.play(place(2, 1, Colour::Black, Shape::Wall))?;

        assert_eq!(perf_count(game.clone(), 1), 87);
        assert_eq!(perf_count(game.clone(), 2), 6155);
        assert_eq!(perf_count(game.clone(), 3), 461_800);
        Ok(())
    }

    #[test]
    fn perft_numbers_5() {
        assert_eq!(perf_count(Game::<5>::default(), 0), 1);
        assert_eq!(perf_count(Game::<5>::default(), 1), 25);
        assert_eq!(perf_count(Game::<5>::default(), 2), 600);
        assert_eq!(perf_count(Game::<5>::default(), 3), 43_320);
        assert_eq!(perf_count(Game::<5>::default(), 4), 2_999_784);
    }
}
