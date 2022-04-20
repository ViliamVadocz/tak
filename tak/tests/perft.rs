use tak::*;

fn perf_count<const N: usize>(game: Game<N>, depth: usize) -> usize {
    if depth == 0 || game.result != GameResult::Ongoing {
        1
    } else if depth == 1 {
        game.possible_moves().len()
    } else {
        game.possible_moves()
            .into_iter()
            .map(|m| {
                let mut clone = game.clone();
                clone.play(m).unwrap();
                perf_count(clone, depth - 1)
            })
            .sum()
    }
}

#[test]
fn position1_perft() -> StrResult<()> {
    let game = Game::<5>::from_ptn_moves(&["d3", "c3", "c4", "1d3<", "1c4-", "Sc4"])?;
    assert_eq!(perf_count(game.clone(), 1), 87);
    assert_eq!(perf_count(game.clone(), 2), 6_155);
    assert_eq!(perf_count(game, 3), 461_800);
    Ok(())
}

#[test]
fn position2_perft() -> StrResult<()> {
    let game = Game::<5>::from_ptn_moves(&[
        "c2", "c3", "d3", "b3", "c4", "1c2+", "1d3<", "1b3>", "1c4-", "Cc2", "a1", "1c2+", "a2",
    ])?;
    assert_eq!(perf_count(game.clone(), 1), 104);
    assert_eq!(perf_count(game.clone(), 2), 7_743);
    assert_eq!(perf_count(game, 3), 592_645);
    Ok(())
}

#[test]
fn position3_perft() -> StrResult<()> {
    let game = Game::<5>::from_ptn_moves(&[
        "c4", "c2", "d2", "c3", "b2", "d3", "1d2+", "b3", "d2", "b4", "1c2+", "1b3>", "2d3<", "1c4-", "d4",
        "5c3<23", "c2", "c4", "1d4<", "d3", "1d2+", "1c3+", "Cc3", "2c4>", "1c3<", "d2", "c3", "1d2+",
        "1c3+", "1b4>", "2b3>11", "3c4-12", "d2", "c4", "b4", "c5", "1b3>", "1c4<", "3c3-", "e5", "e2",
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
