pub mod board;
pub mod core;
pub mod evaluation;
pub mod move_;
pub mod move_generation;
mod test_util;

use board::*;
use crate::core::*;
use evaluation::*;
use move_::*;
use move_generation::*;

use num_traits::Float;

pub struct Line {
    pub moves: Vec<Move>
}

impl Line {
    pub fn new() -> Line {
        Line{moves: Vec::new()}
    }

    pub fn from_moves(moves: Vec<Move>) -> Line {
        Line{moves: moves}
    }

    pub fn to_string(&self) -> String {
        self.moves.iter().map(|m| m.long_algebraic()).collect::<Vec<String>>().join(" ")
    }
}

pub struct MoveUnmove {
    board_before: Board,
    move_: Move,
}

impl MoveUnmove {
    pub fn apply_move(board: &mut Board, move_: &Move) -> MoveUnmove {
        let move_unmove = MoveUnmove {
            board_before: board.clone(),
            move_: *move_,
        };
        board.apply_move(*move_);
        return move_unmove;
    }

    pub fn revert_move(&mut self, board: &mut Board) {
        board.revert_move(self.move_);

        // if !board.semantic_eq(&self.board_before) {
        //     panic!("Board mismatch after {:?}\n{:?}\nvs\n{:?}", self.move_, self.board_before, board);
        // }
    }
}

pub fn play(board: &mut Board) {
    let mut num_moves = 0;

    let max_depth = 0;

    loop {
        let mut evaluator = MinimaxEvaluator::create();
        let d = evaluator.evaluate(board, max_depth);
        println!("{:?}'s turn, static evaluation is {}, dynamic evaluation is {}", board.side, static_evaluation(&board), d);
        board.print();

        if board.is_game_over() {
            println!("Game is over");
            break;
        }

        let mut moves = generate_moves(board);
        println!("{} moves to choose from", moves.len());

        let mut best_move = Option::None;
        let mut best_move_evaluation = Float::min_value();

        let neg = match board.side {
            Color::White => 1.0,
            Color::Black => -1.0
        };

        let mut evaluator = MinimaxEvaluator::create();

        for move_ in moves.iter() {
            let mut move_unmove = MoveUnmove::apply_move(board, move_);
            let evaluation = evaluator.evaluate(board, max_depth) * neg;
            move_unmove.revert_move(board);

            //println!("Evaluating {:?} with {}", move_, evaluation);
            if evaluation > best_move_evaluation {
                best_move = Some(move_);
                best_move_evaluation = evaluation;
            }
        }

        let nodes_per_second = evaluator.get_statistics().node_count as f32 / evaluator.get_statistics().duration.as_secs_f32();
        let best_move = best_move.unwrap();

        println!("Chose move {:?} with an evaluation of {}, evaluated {} nodes at {} nodes/s", best_move, best_move_evaluation * neg, evaluator.get_statistics().node_count, nodes_per_second);
        println!("Line: {}", evaluator.get_best_line().to_string());

        board.apply_move(*best_move);

        num_moves += 1;
        if num_moves > 50 {
            println!("Too many moves, aborting game");
            break;
        }

        println!();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::*;

    #[test]
    fn line_to_string() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 1),
            PieceKind::Pawn.colored(Color::White).at(0, 6)
        );

        let mut moves = Vec::new();
        moves.push(TestMove::from_to(&board, Square::at(0, 1), Square::at(0, 3)));
        moves.push(TestMove::from_to(&board, Square::at(0, 6), Square::at(0, 5)));

        let mut line = Line::from_moves(moves);

        assert_eq!(line.to_string(), "a1-a3 a6-a5");
    }
}