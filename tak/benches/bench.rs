use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use tak::{perf_count, Game};
use takparse::Tps;

const EARLY_TPS: &str = "2,2,2,2,x2/x2,1S,2,x2/x2,1C,2,x2/1,1,1,12C,1,x/x3,1,2,x/x5,1 1 10";

const MIDDLE_TPS: &str =
    "2,2,21S,2,x,1/x,2,1,2,2,1/x,1,x,122S,221C,1/1,1,1,2,112C,2/x2,1,1,x,2/x2,1,x,1212,1 1 24";

const ENDGAME_TPS: &str = "1,1,1S,121,2,1/2,12,2,1,2,1/2,x,2,221C,12C,1/x,212,222221S,21,2,2/1,21,\
                           21,2,2S,1/2,1,1,2,1,2 1 36";

fn perft(c: &mut Criterion) {
    c.bench_function("perft 5x5 depth 4", |b| {
        let game = Game::<5>::default();
        b.iter(|| perf_count(game, black_box(4)))
    });
    c.bench_function("perft 6x6 depth 4", |b| {
        let game = Game::<6>::default();
        b.iter(|| perf_count(game, black_box(4)))
    });
    c.bench_function("perft 7x7 depth 4", |b| {
        let game = Game::<7>::default();
        b.iter(|| perf_count(game, black_box(4)))
    });
}

fn move_gen(c: &mut Criterion) {
    c.bench_function("move_gen early game", |b| {
        let game: Game<6> = EARLY_TPS.parse::<Tps>().unwrap().into();
        let moves = Vec::new();
        b.iter_batched_ref(
            || moves.clone(),
            |moves| black_box(game).possible_moves(moves),
            BatchSize::SmallInput,
        )
    });
    c.bench_function("move_gen middle game", |b| {
        let game: Game<6> = MIDDLE_TPS.parse::<Tps>().unwrap().into();
        let moves = Vec::new();
        b.iter_batched_ref(
            || moves.clone(),
            |moves| black_box(game).possible_moves(moves),
            BatchSize::SmallInput,
        )
    });
    c.bench_function("move_gen endgame", |b| {
        let game: Game<6> = ENDGAME_TPS.parse::<Tps>().unwrap().into();
        let moves = Vec::new();
        b.iter_batched_ref(
            || moves.clone(),
            |moves| black_box(game).possible_moves(moves),
            BatchSize::SmallInput,
        )
    });
}

fn making_moves(c: &mut Criterion) {
    c.bench_function("making a spread", |b| {
        let game: Game<6> = ENDGAME_TPS.parse::<Tps>().unwrap().into();
        let mov = "5c3>212".parse().unwrap();
        b.iter_batched(
            || game,
            |mut g| g.play(black_box(mov)).unwrap(),
            BatchSize::SmallInput,
        )
    });
}

fn canonical(c: &mut Criterion) {
    c.bench_function("canonical early", |b| {
        let game: Game<6> = EARLY_TPS.parse::<Tps>().unwrap().into();
        b.iter(|| black_box(game).canonical())
    });
    c.bench_function("canonical middle", |b| {
        let game: Game<6> = MIDDLE_TPS.parse::<Tps>().unwrap().into();
        b.iter(|| black_box(game).canonical())
    });
    c.bench_function("canonical endgame", |b| {
        let game: Game<6> = ENDGAME_TPS.parse::<Tps>().unwrap().into();
        b.iter(|| black_box(game).canonical())
    });
}

criterion_group!(benches, perft, move_gen, making_moves, canonical);
criterion_main!(benches);
