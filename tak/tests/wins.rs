use tak::*;
use takparse::Color;

#[test]
fn double_road_correct_win() -> StrResult<()> {
    let game = Game::<6>::from_ptn_moves(&[
        "a4", "a3", "b3", "b4", "c3", "c4", "d3", "d4", "d3+", "e4", "e3", "f4", "f3", "Cb5", "d4-",
    ])?;
    assert_eq!(game.result, GameResult::Winner {
        color: Color::White,
        road: true
    });
    Ok(())
}

#[test]
fn flat_win() -> StrResult<()> {
    let game = Game::<3>::from_ptn_moves(&["a3", "c1", "c2", "c3", "b3", "b2", "b1", "a1", "a2"])?;
    assert_eq!(game.result, GameResult::Winner {
        color: Color::White,
        road: false
    });
    Ok(())
}

#[test]
fn road_win() -> StrResult<()> {
    let game = Game::<5>::from_ptn_moves(&[
        "d2", "a5", "b4", "d3", "Cc3", "Cc2", "b2", "b1", "b3", "a1", "c4", "c1", "e2", "e3",
    ])?;
    assert_eq!(game.result, GameResult::Winner {
        color: Color::Black,
        road: true
    });
    Ok(())
}

#[test]
fn road_beats_flats() -> StrResult<()> {
    let game = Game::<3>::from_ptn_moves(&["a1", "c1", "c2", "a2", "Sa3", "b1", "Sb3", "b2", "c3"])?;
    assert_eq!(game.result, GameResult::Winner {
        color: Color::White,
        road: true
    });
    Ok(())
}
