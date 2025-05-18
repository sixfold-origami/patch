use std::{str::FromStr, time::Duration};

use chess::Board;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use patch::engine::evaluation::eval_heuristic;

const FENS: [&str; 6] = [
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
    "rnb1k1nr/pp3ppp/2pp1q2/4p3/2BbP3/2N2N2/PPP2PPP/R1BQ1RK1 w kq - 0 1",
    "r1b2rk1/pp1n1p1p/1bp3qp/3p4/4p3/1QP2NNP/PP2BPPK/R4R2 w - - 0 1",
    "4rr1k/pp3p1p/1b2n2p/3p1q2/1Q6/2P1pPPP/PP2B2K/2R2R2 b - - 0 1",
    "4Br1k/pp3p1p/8/2Qp2np/5P2/2P5/PP6/3q3K w - - 0 1",
];

fn criterion_benchmark(c: &mut Criterion) {
    let boards: Vec<_> = FENS
        .into_iter()
        .map(|fen| Board::from_str(fen).unwrap())
        .collect();

    let mut group = c.benchmark_group("evaluation heuristic");
    group
        .sample_size(1000)
        .measurement_time(Duration::from_secs(30));

    for board in boards.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(board), board, |b, board| {
            b.iter(|| eval_heuristic(board));
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
