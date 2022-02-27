use tak::*;

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
    assert_eq!(game.winner(), GameResult::Winner {
        colour: Colour::White,
        road: true
    });
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
    assert_eq!(game.winner(), GameResult::Winner {
        colour: Colour::White,
        road: false
    });
    Ok(())
}

#[test]
fn road_win() -> StrResult<()> {
    let game = Game::<5>::from_ptn(
        "1. d2 a5
        2. b4 d3
        3. Cc3 Cc2
        4. b2 b1
        5. b3 a1
        6. c4 c1
        7. e2 e3",
    )?;
    assert_eq!(game.winner(), GameResult::Winner {
        colour: Colour::Black,
        road: true
    });
    Ok(())
}

#[test]
fn road_beats_flats() -> StrResult<()> {
    let mut game = Game::<5>::from_ptn(
        "[TPS \"2,1,1,1,2/x,1,11211112C,221C,21/1212,x,2,x2/x,1112S,x,2,1/221S,2121,x,2,x 1 42\"]
        [Komi \"2\"]",
    )?;
    game.play(Turn::from_ptn("a4")?)?;
    assert_eq!(game.winner(), GameResult::Winner {
        colour: Colour::White,
        road: true
    });
    Ok(())
}
