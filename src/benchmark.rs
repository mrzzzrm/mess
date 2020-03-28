#[macro_use]
extern crate bencher;

use mess::board::*;
use mess::move_generation::*;
use mess::MoveUnmove;
use mess::evaluation::{MinimaxEvaluator, DynamicEvaluator, AlphaBetaEvaluator};

fn generate_moves_rec(board: &mut Board, depth: u32) {
    if depth == 0 {
        return;
    }
    let moves = generate_moves(board);
    for m in moves.iter() {
        let mut move_unmove = MoveUnmove::apply_move(board, &m);
        generate_moves_rec(board, depth - 1);
        move_unmove.revert_move(board);
    }
}

fn bench_move_generation(b: &mut bencher::Bencher) {
    let mut board = Board::create_populated();

    b.iter(|| {
        generate_moves_rec(&mut board, 3);
    });
}

fn bench_minimax(b: &mut bencher::Bencher) {
    let mut board = Board::create_populated();

    b.iter(|| {
        let mut evaluator = MinimaxEvaluator::create(4);
        evaluator.evaluate(&mut board);
    });
}

fn bench_alphabeta(b: &mut bencher::Bencher) {
    let mut board = Board::create_populated();gggdaHallo wie geht es euch allen?

    b.iter(|| {
        let mut evaluator = AlphaBetaEvaluator::create(4 );
        evaluator.evaluate(&mut board);
    });
}

benchmark_group!(benches, bench_move_generation, bench_minimax, bench_alphabeta);
benchmark_main!(benches);