use super::core::{Color, ColorCastleRights, PieceKind, Piece};
use super::{Line, MoveUnmove};
use super::board::{Board};
use super::move_generation::{generate_moves};

pub fn static_evaluation(board: &Board) -> f32 {
    let mut evaluation = 0.0;
    for (piece, _) in board.piece_list.iter() {
        evaluation += piece.value();
    }
    return evaluation;
}

#[derive(Clone, Copy, Debug)]
pub struct DynamicEvaluatorStatistics {
    pub node_count: u64,
    pub duration: std::time::Duration,
}

impl DynamicEvaluatorStatistics {
    fn create() -> DynamicEvaluatorStatistics {
        DynamicEvaluatorStatistics {
            node_count: 0,
            duration: std::time::Duration::new(0, 0),
        }
    }
}

pub trait DynamicEvaluator {
    fn create(max_depth: u32) -> Self where Self: Sized;
    fn evaluate(&mut self, board: &mut Board) -> f32;
    fn get_best_line(&self) -> &Line;
    fn get_statistics(&self) -> DynamicEvaluatorStatistics;
}

pub struct MinimaxEvaluator {
    statistics: DynamicEvaluatorStatistics,
    best_line: Line,
    max_depth: u32,
}

impl MinimaxEvaluator {
    pub fn minimax(&mut self, board: &mut Board, depth: u32, neg: f32) -> (f32, Line) {
        self.statistics.node_count += 1;

        if depth == self.max_depth {
            return (static_evaluation(&board), Line::empty());
        }

        let moves = generate_moves(&board);
        if moves.is_empty() {
            return (static_evaluation(&board), Line::empty());
        }

        let mut best_line = None;
        let mut best_move_evaluation = None;

        for m in moves.iter() {
            let mut move_unmove = MoveUnmove::apply_move(board, m);
            let (mut evaluation, line) = self.minimax(board, depth + 1, neg * -1.0);
            evaluation *= neg;
            move_unmove.revert_move(board);

            if best_move_evaluation.is_none() || evaluation > best_move_evaluation.unwrap() {
                best_move_evaluation = Some(evaluation);
                best_line = Some(line);
                best_line.as_mut().and_then(|line| {line.push_front(m); return Some(line);} );
            }
        }

        return (best_move_evaluation.unwrap() * neg, best_line.unwrap());
    }
}

impl DynamicEvaluator for MinimaxEvaluator {
    fn create(max_depth: u32) -> MinimaxEvaluator {
        MinimaxEvaluator { statistics: DynamicEvaluatorStatistics::create(), best_line: Line::empty(), max_depth }
    }

    fn evaluate(&mut self, board: &mut Board) -> f32 {
        self.best_line.moves.clear();

        let neg = match board.side {
            Color::White => 1.0,
            Color::Black => -1.0
        };

        let stopwatch = std::time::Instant::now();
        let (evaluation, line) = self.minimax(board, 0, neg);
        self.best_line = line;
        self.statistics.duration += stopwatch.elapsed();

        return evaluation;
    }

    fn get_best_line(&self) -> &Line {
        &self.best_line
    }

    fn get_statistics(&self) -> DynamicEvaluatorStatistics {
        self.statistics
    }
}

pub struct AlphaBetaEvaluator {
    statistics: DynamicEvaluatorStatistics,
    best_line: Line,
    max_depth: u32,
}

impl AlphaBetaEvaluator {
    fn alpha_beta_min(&mut self, board: &mut Board, alpha: f32, mut beta: f32, depth: u32) -> f32 {
        self.statistics.node_count += 1;
        if depth == self.max_depth {
            return static_evaluation(&board);
        }

        let mut moves = generate_moves(&board);
        if moves.is_empty() {
            return static_evaluation(&board);
        }

        let mut best_move_evaluation = None;

        for m in moves.iter() {
            let mut move_unmove = MoveUnmove::apply_move(board, m);
            let evaluation = self.alpha_beta_max(board, alpha, beta, depth + 1);
            move_unmove.revert_move(board);

            if evaluation <= alpha {
                return evaluation;
            }

            if evaluation < beta {
                beta = evaluation;
            }

            if best_move_evaluation == None || evaluation < best_move_evaluation.unwrap() {
                best_move_evaluation = Some(evaluation);
            }
        }

        return best_move_evaluation.unwrap();
    }

    fn alpha_beta_max(&mut self, board: &mut Board, mut alpha: f32, beta: f32, depth: u32) -> f32 {
        self.statistics.node_count += 1;
        if depth == self.max_depth {
            return static_evaluation(&board);
        }

        let mut moves = generate_moves(&board);
        if moves.is_empty() {
            return static_evaluation(&board);
        }

        let mut best_move_evaluation = None;

        for m in moves.iter() {
            let mut move_unmove = MoveUnmove::apply_move(board, m);
            let evaluation = self.alpha_beta_min(board, alpha, beta, depth + 1);
            move_unmove.revert_move(board);

            if evaluation >= beta {
                return evaluation;
            }

            if evaluation > alpha {
                alpha = evaluation;
            }

            if best_move_evaluation == None || evaluation > best_move_evaluation.unwrap() {
                best_move_evaluation = Some(evaluation);
            }
        }

        return best_move_evaluation.unwrap();
    }
}

impl DynamicEvaluator for AlphaBetaEvaluator {
    fn create(max_depth: u32) -> AlphaBetaEvaluator {
        AlphaBetaEvaluator { statistics: DynamicEvaluatorStatistics::create(), best_line: Line::empty(), max_depth }
    }

    fn evaluate(&mut self, board: &mut Board) -> f32 {
        self.best_line.moves.clear();

        let stopwatch = std::time::Instant::now();
        let evaluation = match board.side {
            Color::White => self.alpha_beta_max(board, num_traits::float::Float::min_value(), num_traits::float::Float::max_value(), 0),
            Color::Black => self.alpha_beta_min(board, num_traits::float::Float::min_value(), num_traits::float::Float::max_value(), 0)
        };
        self.statistics.duration += stopwatch.elapsed();

        return evaluation;
    }

    fn get_best_line(&self) -> &Line {
        &self.best_line
    }

    fn get_statistics(&self) -> DynamicEvaluatorStatistics {
        self.statistics
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn dynamic_evaluator_basic<DynamicEvaluatorT: DynamicEvaluator>() {
        // Just a white pawn
        let mut board = Board::create_empty();
        let mut evaluator = DynamicEvaluatorT::create(3);
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 1));
        assert_eq!(evaluator.evaluate(&mut board), 1.0);

        // Just a black pawn
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::Black).at(0, 6));
        let mut evaluator = DynamicEvaluatorT::create(3);
        assert_eq!(evaluator.evaluate(&mut board), -1.0);

        // A white pawn that can capture a black pawn
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 1),
            PieceKind::Pawn.colored(Color::Black).at(1, 2));
        let mut evaluator = DynamicEvaluatorT::create(3);
        assert_eq!(evaluator.evaluate(&mut board), 1.0);

        // A black pawn that can capture a white pawn
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 2),
            PieceKind::Pawn.colored(Color::Black).at(1, 3));
        let mut evaluator = DynamicEvaluatorT::create(3);
        assert_eq!(evaluator.evaluate(&mut board), -1.0);

        // A white pawn that can capture a black pawn and another black pawn
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 1),
            PieceKind::Pawn.colored(Color::Black).at(1, 2),
            PieceKind::Pawn.colored(Color::Black).at(3, 2));
        let mut evaluator = DynamicEvaluatorT::create(3);
        assert_eq!(evaluator.evaluate(&mut board), 0.0);

        // A white pawn that will be captured by a black pawn after it moves
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 4),
            PieceKind::Pawn.colored(Color::Black).at(1, 6));
        let mut evaluator = DynamicEvaluatorT::create(3);
        assert_eq!(evaluator.evaluate(&mut board), -1.0);

        // A white pawn that will capture a black pawn after the black pawn moves
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 3),
            PieceKind::Pawn.colored(Color::Black).at(1, 5));
        let mut evaluator = DynamicEvaluatorT::create(3);
        assert_eq!(evaluator.evaluate(&mut board), 1.0);

        // A white pawn that will be captured by a black pawn after a couple of moves
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 2),
            PieceKind::Pawn.colored(Color::Black).at(1, 5), );
        let mut evaluator = DynamicEvaluatorT::create(10);
        assert_eq!(evaluator.evaluate(&mut board), -1.0);

        // ...
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 3),
            PieceKind::Pawn.colored(Color::White).at(1, 5),
            PieceKind::Pawn.colored(Color::Black).at(0, 6), );
        let mut evaluator = DynamicEvaluatorT::create(10);
        assert_eq!(evaluator.evaluate(&mut board), -1.0);
    }

    #[test]
    fn minimax_basic() {
        dynamic_evaluator_basic::<MinimaxEvaluator>();
    }

    #[test]
    fn alpha_beta_basic() {
        dynamic_evaluator_basic::<AlphaBetaEvaluator>();
    }

    #[test]
    fn static_evaluation_basic() {
        let mut board = Board::create_empty();

        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 1));
        assert_eq!(static_evaluation(&board), 1.0);

        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 1),
            PieceKind::Pawn.colored(Color::Black).at(0, 2),
            PieceKind::Pawn.colored(Color::Black).at(0, 3));
        assert_eq!(static_evaluation(&board), -1.0);
    }
}