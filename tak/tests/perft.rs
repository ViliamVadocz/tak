use tak::*;

fn perf_count<const N: usize>(game: &Game<N>, depth: usize) -> usize {
    if depth == 0 || game.result() != GameResult::Ongoing {
        1
    } else if depth == 1 {
        game.possible_moves().len()
    } else {
        game.possible_moves()
            .into_iter()
            .map(|m| {
                let mut clone = game.clone();
                clone.play(m).unwrap();
                perf_count(&clone, depth - 1)
            })
            .sum()
    }
}

#[test]
fn move_stack_perft() -> Result<(), PlayError> {
    let game = Game::<5>::from_ptn_moves(&["d3", "c3", "c4", "1d3<", "1c4-", "Sc4"])?;
    assert_eq!(perf_count(&game, 1), 87);
    assert_eq!(perf_count(&game, 2), 6_155);
    assert_eq!(perf_count(&game, 3), 461_800);
    Ok(())
}

#[test]
fn respect_carry_limit_perft() -> Result<(), PlayError> {
    let game = Game::<5>::from_ptn_moves(&[
        "c2", "c3", "d3", "b3", "c4", "1c2+", "1d3<", "1b3>", "1c4-", "Cc2", "a1", "1c2+", "a2",
    ])?;
    assert_eq!(perf_count(&game, 1), 104);
    assert_eq!(perf_count(&game, 2), 7_743);
    assert_eq!(perf_count(&game, 3), 592_645);
    Ok(())
}

#[test]
fn suicide_perft() -> Result<(), PlayError> {
    let game = Game::<5>::from_ptn_moves(&[
        "c4", "c2", "d2", "c3", "b2", "d3", "1d2+", "b3", "d2", "b4", "1c2+", "1b3>", "2d3<", "1c4-", "d4",
        "5c3<23", "c2", "c4", "1d4<", "d3", "1d2+", "1c3+", "Cc3", "2c4>", "1c3<", "d2", "c3", "1d2+",
        "1c3+", "1b4>", "2b3>11", "3c4-12", "d2", "c4", "b4", "c5", "1b3>", "1c4<", "3c3-", "e5", "e2",
    ])?;
    assert_eq!(perf_count(&game, 1), 85);
    assert_eq!(perf_count(&game, 2), 11_206);
    assert_eq!(perf_count(&game, 3), 957_000);
    Ok(())
}

#[test]
fn endgame_perft() -> Result<(), PlayError> {
    let game = Game::<5>::from_ptn_moves(&[
        "a5", "b4", "c3", "d2", "e1", "d1", "c2", "d3", "c1", "d4", "d5", "c4", "c5", "b3", "b2", "a2",
        "Sb1", "a3", "Ce4", "Cb5", "a4", "a1", "e5", "e3", "c3<", "Sc3", "c1>", "c1", "2d1+", "c3-", "c3",
        "a3>", "a3", "d1", "e4<", "2c2>", "c2", "e2", "b2+", "b2",
    ])?;
    assert_eq!(perf_count(&game, 1), 65);
    assert_eq!(perf_count(&game, 2), 4_072);
    assert_eq!(perf_count(&game, 3), 272_031);
    assert_eq!(perf_count(&game, 4), 16_642_760);
    Ok(())
}

#[test]
fn reserves_perft() -> Result<(), PlayError> {
    let game = Game::<5>::from_ptn_moves(&[
        "a1", "b1", "c1", "d1", "e1", "e2", "d2", "c2", "b2", "a2", "a3", "b3", "c3", "d3", "e3", "a4", "b4",
        "c4", "d4", "e4", "a5", "a4-", "b4-", "c4-", "d4-", "e4-", "a4", "b4", "c4", "d4", "e4", "2a3>",
        "c4>", "2e3<", "a3", "4b3-", "b3", "c4", "e3", "d5", "d2<", "d2", "2d4-", "d4", "c5", "b5", "2c2>",
        "d1+", "c2", "e2+", "d1", "e2", "c5<", "c5", "e4<", "Se4", "2b5-", "e4-", "a3-",
    ])?;
    assert_eq!(perf_count(&game, 1), 152);
    assert_eq!(perf_count(&game, 2), 15_356);
    assert_eq!(perf_count(&game, 3), 1_961_479);
    // assert_eq!(perf_count(&game, 4), 197_434_816);
    Ok(())
}

#[test]
fn perft_5() {
    assert_eq!(perf_count(&Game::<5>::default(), 0), 1);
    assert_eq!(perf_count(&Game::<5>::default(), 1), 25);
    assert_eq!(perf_count(&Game::<5>::default(), 2), 600);
    assert_eq!(perf_count(&Game::<5>::default(), 3), 43_320);
    assert_eq!(perf_count(&Game::<5>::default(), 4), 2_999_784);
}

#[test]
fn perft_6() {
    assert_eq!(perf_count(&Game::<6>::default(), 0), 1);
    assert_eq!(perf_count(&Game::<6>::default(), 1), 36);
    assert_eq!(perf_count(&Game::<6>::default(), 2), 1_260);
    assert_eq!(perf_count(&Game::<6>::default(), 3), 132_720);
    assert_eq!(perf_count(&Game::<6>::default(), 4), 13_586_048);
    // assert_eq!(perf_count(&Game::<6>::default(), 5), 1_253_506_520);
}
