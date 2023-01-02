use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use tak::{perf_count, Game};
use takparse::Tps;

const ENDGAME_TPS: &str = "1,1,1S,121,2,1/2,12,2,1,2,1/2,x,2,221C,12C,1/x,212,222221S,21,2,2/1,21,\
                           21,2,2S,1/2,1,1,2,1,2 1 36";

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("perft 5x5 depth 4", |b| {
        let game = Game::<5>::default();
        b.iter(|| perf_count(game, black_box(4)))
    });
    c.bench_function("move_gen endgame", |b| {
        let game: Game<6> = ENDGAME_TPS.parse::<Tps>().unwrap().into();
        let moves = Vec::with_capacity(200);
        b.iter_batched_ref(
            || moves.clone(),
            |moves| black_box(game).possible_moves(moves),
            BatchSize::SmallInput,
        )
    });
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

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
