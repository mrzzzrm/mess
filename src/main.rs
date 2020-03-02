extern crate num_traits;

use num_traits::Float;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Square {
    x: i8,
    y: i8,
}

impl Square {
    fn at(x: i8, y: i8) -> Square {
        Square { x, y }
    }

    fn file(&self) -> i8 {
        self.x
    }

    fn rank(&self) -> i8 {
        self.y
    }

    fn is_on_board(&self) -> bool {
        return self.x >= 0 && self.x < 8 && self.y >= 0 && self.y < 8;
    }

    fn delta(&self, x: i8, y: i8) -> Square {
        Square { x: self.x + x, y: self.y + y }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
    White,
    Black,
}

impl Color {
    fn switch(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White
        }
    }
    fn index(self) -> usize {
        match self {
            Color::White => 0,
            Color::Black => 1
        }
    }
    fn token(&self) -> char {
        match self {
            Color::White => 'B',
            Color::Black => 'W'
        }
    }
    fn evaluation_sign(&self) -> f32 {
        match self {
            Color::White => 1.0_f32,
            Color::Black => -1.0_f32
        }
    }
    fn promotion_rank(&self) -> u8 {
        match self {
            Color::White => 7,
            Color::Black => 0
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    Dummy,
}

impl PieceKind {
    fn value(&self) -> f32 {
        match self {
            PieceKind::Pawn => 1.0,
            PieceKind::Knight => 3.0,
            PieceKind::Bishop => 3.0,
            PieceKind::Rook => 5.0,
            PieceKind::Queen => 9.0,
            PieceKind::King => 200.0,
            PieceKind::Dummy => 0.0
        }
    }

    fn token(&self) -> char {
        match self {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
            PieceKind::Dummy => 'd'
        }
    }

    fn colored(self, color: Color) -> Piece {
        Piece::create(self, color)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Piece {
    kind: PieceKind,
    color: Color,
}

impl Piece {
    fn create(kind: PieceKind, color: Color) -> Piece {
        return Piece { kind, color };
    }
    fn at(&self, file: i8, rank: i8) -> PieceOnBoard {
        return (*self, Square { x: file, y: rank });
    }

    fn value(&self) -> f32 {
        self.kind.value() * self.color.evaluation_sign()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Move {
    from: Square,
    to: Square,
    capture: Option<PieceOnBoard>,
    en_passant_before: Option<Square>,
    en_passant_after: Option<Square>,
    is_castling: bool,
    promotion: Option<PieceKind>,
}

impl Move {
    fn castling(from: Square, to: Square) -> Move {
        Move { from, to, capture: None, en_passant_before: None, en_passant_after: None, is_castling: true, promotion: None }
    }

    fn promotion(from: Square, to: Square, promotion: PieceKind) -> Move {
        Move { from, to, capture: None, en_passant_before: None, en_passant_after: None, is_castling: false, promotion: Some(promotion) }
    }

    fn promotion_capture(from: Square, to: Square, capture: PieceOnBoard, promotion: PieceKind) -> Move {
        Move { from, to, capture: Some(capture), en_passant_before: None, en_passant_after: None, is_castling: false, promotion: Some(promotion) }
    }

    fn from_to(from: Square, to: Square) -> Move {
        Move { from, to, capture: None, en_passant_before: None, en_passant_after: None, is_castling: false, promotion: None }
    }

    fn from_to_en_passant(from: Square, to: Square, en_passant: Square) -> Move {
        Move { from, to, capture: None, en_passant_before: None, en_passant_after: Some(en_passant), is_castling: false, promotion: None }
    }

    fn from_to_capture(from: Square, to: Square, capture: PieceOnBoard) -> Move {
        Move { from, to, capture: Some(capture), en_passant_before: None, en_passant_after: None, is_castling: false, promotion: None }
    }
}

type PieceOnBoard = (Piece, Square);

#[derive(Debug, PartialEq)]
struct Board {
    piece_list: Vec<PieceOnBoard>,
    side: Color,
    en_passant: Option<Square>,
    castle_right_king_side: [bool; 2],
    castle_right_queen_side: [bool; 2],
}

impl Board {
    fn create_empty() -> Board {
        Board {
            piece_list: Vec::new(),
            side: Color::White,
            en_passant: None,
            castle_right_king_side: [true, true],
            castle_right_queen_side: [true, true],
        }
    }

    fn create_populated() -> Board {
        let mut board = Board::create_empty();
        for x in 0..8 {
            board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(x, 1));
            board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(x, 6));
        }
        board.piece_list.push(PieceKind::Rook.colored(Color::White).at(0, 0));
        board.piece_list.push(PieceKind::Rook.colored(Color::White).at(7, 0));
        board.piece_list.push(PieceKind::Rook.colored(Color::Black).at(0, 7));
        board.piece_list.push(PieceKind::Rook.colored(Color::Black).at(7, 7));
        board.piece_list.push(PieceKind::Knight.colored(Color::White).at(1, 0));
        board.piece_list.push(PieceKind::Knight.colored(Color::White).at(6, 0));
        board.piece_list.push(PieceKind::Knight.colored(Color::Black).at(1, 7));
        board.piece_list.push(PieceKind::Knight.colored(Color::Black).at(6, 7));
        board.piece_list.push(PieceKind::Bishop.colored(Color::White).at(2, 0));
        board.piece_list.push(PieceKind::Bishop.colored(Color::White).at(5, 0));
        board.piece_list.push(PieceKind::Bishop.colored(Color::Black).at(2, 7));
        board.piece_list.push(PieceKind::Bishop.colored(Color::Black).at(5, 7));
        board.piece_list.push(PieceKind::Queen.colored(Color::White).at(3, 0));
        board.piece_list.push(PieceKind::King.colored(Color::White).at(4, 0));
        board.piece_list.push(PieceKind::Queen.colored(Color::Black).at(3, 7));
        board.piece_list.push(PieceKind::King.colored(Color::Black).at(4, 7));

        return board;
    }

    fn piece_at(&self, square: Square) -> Option<Piece> {
        for (piece, square2) in self.piece_list.iter() {
            if square == *square2 {
                return Some(*piece);
            }
        }
        return None;
    }

    fn has_piece_at(&self, square: Square) -> bool {
        self.piece_list.iter().position(|(_, square2)| square == *square2).is_some()
    }

    fn apply_move(&mut self, m: Move) {
        assert_eq!(m.en_passant_before, self.en_passant);
g
        self.side = self.side.switch();
        self.en_passant = m.en_passant_after;

        if let Some(capture) = m.capture {
            let pos = self.piece_list.iter().position(|&x| x.1 == capture.1).unwrap();
            self.piece_list.remove(pos);
        }

        for t in self.piece_list.iter_mut() {
            if t.1 == m.from {
                t.1 = m.to;
                if let Some(promotion) = m.promotion {
                    t.0.kind = promotion;
                }

                return;
            }
        }

        panic!("{:?} not found", m.from);
    }

    fn revert_move(&mut self, m: Move) {
        self.side = self.side.switch();
        self.en_passant = m.en_passant_before;

        for t in self.piece_list.iter_mut() {
            if t.1 == m.to {
                if let Some(promotion) = m.promotion {
                    t.0.kind = PieceKind::Pawn;
                }

                t.1 = m.from;
            }
        }

        if let Some(capture) = m.capture {
            self.piece_list.push(capture);
        }

    }

    fn is_game_over(&self) -> bool {
        generate_moves(self).is_empty()
    }

    fn print(&self) {
        print!("  ", );
        for file in 0..8 {
            print!("{} ", file);
        }
        println!();

        for rank in 0..8 {
            print!("{} ", rank);
            for file in 0..8 {
                let piece = self.piece_at(Square::at(file, rank));

                let mut token = match piece {
                    Some(piece) => piece.kind.token(),
                    None => '.'
                };

                if piece.is_some() && piece.unwrap().color == Color::White {
                    token = token.to_uppercase().to_string().chars().nth(0).unwrap();
                }

                print!("{} ", token.to_string());
            }
            println!();
        }
    }
}

fn static_evaluation(board: &Board) -> f32 {
    let mut evaluation = 0.0;
    for (piece, _) in board.piece_list.iter() {
        evaluation += piece.value();
    }
    return evaluation;
}

#[derive(Clone, Copy, Debug)]
struct DynamicEvaluatorStatistics {
    node_count: u64,
    duration: std::time::Duration,
}

impl DynamicEvaluatorStatistics {
    fn create() -> DynamicEvaluatorStatistics {
        DynamicEvaluatorStatistics {
            node_count: 0,
            duration: std::time::Duration::new(0, 0),
        }
    }
}

trait DynamicEvaluator {
    fn evaluate(&mut self, board: &mut Board, depth: u32) -> f32;
    fn get_statistics(&self) -> DynamicEvaluatorStatistics;
}

struct MinimaxEvaluator {
    statistics: DynamicEvaluatorStatistics
}

impl MinimaxEvaluator {
    fn create() -> MinimaxEvaluator {
        MinimaxEvaluator { statistics: DynamicEvaluatorStatistics::create() }
    }

    fn minimax(&mut self, board: &mut Board, depth: u32, neg: f32) -> f32 {
        self.statistics.node_count += 1;

        if depth == 0 {
            return static_evaluation(&board);
        }

        let mut moves = generate_moves(&board);
        if moves.is_empty() {
            return static_evaluation(&board);
        }

        let mut best_move_evaluation = None;

        for m in moves.iter() {
            board.apply_move(*m);

            let evaluation = self.minimax(board, depth - 1, neg * -1.0) * neg;

            board.revert_move(*m);

            if best_move_evaluation == None || evaluation > best_move_evaluation.unwrap() {
                best_move_evaluation = Some(evaluation);
            }
        }

        return best_move_evaluation.unwrap() * neg;
    }
}

impl DynamicEvaluator for MinimaxEvaluator {
    fn evaluate(&mut self, board: &mut Board, depth: u32) -> f32 {
        let neg = match board.side {
            Color::White => 1.0,
            Color::Black => -1.0
        };

        let stopwatch = std::time::Instant::now();
        let evaluation = self.minimax(board, depth, neg) * neg;
        self.statistics.duration += stopwatch.elapsed();

        return evaluation;
    }

    fn get_statistics(&self) -> DynamicEvaluatorStatistics {
        self.statistics
    }
}

// Add a move by x_delta, y_delta to the moves if the target square is on board and is unoccupied
// or can be captured. Return whether the target square was unoccupied.
fn probe_move(board: &Board, piece: &Piece, current_square: &Square, x_delta: i8, y_delta: i8, moves: &mut Vec<Move>) -> bool {
    let target_square = current_square.delta(x_delta, y_delta);
    if !target_square.is_on_board() {
        return false;
    }

    let target_piece = board.piece_at(target_square);

    match target_piece {
        Some(target_piece) => {
            if target_piece.color == piece.color {
                return false;
            } else {
                moves.push(Move::from_to_capture(*current_square, target_square, (target_piece, target_square)));
                return false;
            }
        }
        None => {
            moves.push(Move::from_to(*current_square, target_square));
            return true;
        }
    }
}

// Generate moves for the "directional" pieces Bishop, Rook and Queen.
fn generate_directional_moves(board: &Board, piece: &Piece, current_square: &Square, x_delta: i8, y_delta: i8, moves: &mut Vec<Move>) {
    let mut step_idx = 1;
    loop {
        if !probe_move(board, piece, current_square, x_delta * step_idx, y_delta * step_idx, moves) {
            break;
        }
        step_idx += 1;
    }
}

// Generate either a normal or a promotion move, depending on which rank the pawn is headed to
fn generate_pawn_move(piece: &Piece, from: &Square, to: &Square, capture: &Option<PieceOnBoard>, moves: &mut Vec<Move>) {
    if to.rank() as u8 == piece.color.promotion_rank() {
        for promotion in &[PieceKind::Knight, PieceKind::Bishop, PieceKind::Rook, PieceKind::Queen] {
            if let Some(capture) = capture {
                moves.push(Move::promotion_capture(*from, *to, *capture, *promotion));
            } else {
                moves.push(Move::promotion(*from, *to, *promotion));
            }
        }
    } else {
        if let Some(capture) = capture {
            moves.push(Move::from_to_capture(*from, *to, *capture));
        } else {
            moves.push(Move::from_to(*from, *to));
        }
    }
}

fn generate_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();

    for (piece, square) in board.piece_list.iter() {
        if piece.color != board.side {
            continue;
        }

        match piece.kind {
            PieceKind::Pawn => {
                let forward = match piece.color {
                    Color::White => 1,
                    Color::Black => -1
                };

                let home_rank = match piece.color {
                    Color::White => 1,
                    Color::Black => 6
                };

                if !board.has_piece_at(square.delta(0, forward)) && square.delta(0, forward).is_on_board() {
                    generate_pawn_move(piece, square, &square.delta(0, forward), &None, &mut moves);

                    if square.rank() == home_rank && !board.has_piece_at(square.delta(0, forward * 2)) && square.delta(0, forward * 2).is_on_board() {
                        moves.push(Move::from_to_en_passant(*square, square.delta(0, forward * 2), square.delta(0, forward)));
                    }
                }

                // Generate capture moves
                for file_delta in [-1 as i8, 1 as i8].iter() {
                    let target_piece = board.piece_at(square.delta(*file_delta, forward));

                    if target_piece.is_some() {
                        let target_piece = target_piece.unwrap();
                        if target_piece.color != piece.color {
                            generate_pawn_move(piece, square, &square.delta(*file_delta, forward), &Some((target_piece, square.delta(*file_delta, forward))), &mut moves);
                        }
                    }

                    if board.en_passant.is_some() && board.en_passant.unwrap() == square.delta(*file_delta, forward) {
                        let en_passant_piece = board.piece_at(square.delta(*file_delta, 0)).unwrap();
                        moves.push(Move::from_to_capture(*square, square.delta(*file_delta, forward), (en_passant_piece, square.delta(*file_delta, 0))));
                    }
                }
            }
            PieceKind::Rook => {
                for (x_delta, y_delta) in [(1, 0), (-1, 0), (0, 1), (0, -1)].iter() {
                    generate_directional_moves(board, piece, square, *x_delta as i8, *y_delta as i8, &mut moves);
                }
            }
            PieceKind::Bishop => {
                for (x_delta, y_delta) in [(1, 1), (-1, 1), (-1, -1), (1, -1)].iter() {
                    generate_directional_moves(board, piece, square, *x_delta as i8, *y_delta as i8, &mut moves);
                }
            }
            PieceKind::Queen => {
                for (x_delta, y_delta) in [(1, 0), (-1, 0), (0, 1), (0, -1)].iter() {
                    generate_directional_moves(board, piece, square, *x_delta as i8, *y_delta as i8, &mut moves);
                }
                for (x_delta, y_delta) in [(1, 1), (-1, 1), (-1, -1), (1, -1)].iter() {
                    generate_directional_moves(board, piece, square, *x_delta as i8, *y_delta as i8, &mut moves);
                }
            }
            PieceKind::King => {
                for (x_delta, y_delta) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (-1, -1), (1, -1)].iter() {
                    probe_move(board, piece, square, *x_delta as i8, *y_delta as i8, &mut moves);
                }
                //
                // // Generate castling
                // if board.castle_rights[piece.color.index()] {
                //
                // }
            }
            PieceKind::Knight => {
                for (x_delta, y_delta) in [(-2, -1), (-1, -2), (1, -2), (2, -1), (2, 1), (1, 2), (-1, 2), (-2, 1)].iter() {
                    probe_move(board, piece, square, *x_delta as i8, *y_delta as i8, &mut moves);
                }
            }
            PieceKind::Dummy => {}
        }
    }

    // Preserve the current en passant square
    if let Some(en_passant) = board.en_passant {
        for mut move_ in moves.iter_mut() {
            move_.en_passant_before = Some(en_passant);
        }
    }

    // Preserve the current castling rights
    // move_.castle_right_king_side_before = board.castle_right_king_side;
    // move_.castle_right_queen_side_before = board.castle_right_queen_side;

    return moves;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pawn_moves() {
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(0, 6));
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(2, 1));
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(3, 2));

        let expected_moves = vec!(
            Move::from_to(Square::at(2, 1), Square::at(2, 2)),
            Move::from_to_en_passant(Square::at(2, 1), Square::at(2, 3), Square::at(2, 2)),
            Move::from_to(Square::at(3, 2), Square::at(3, 3)),
        );
        assert_eq!(generate_moves(&board), expected_moves);

        board.side = Color::Black;
        let expected_moves = vec!(
            Move::from_to(Square::at(0, 6), Square::at(0, 5)),
            Move::from_to_en_passant(Square::at(0, 6), Square::at(0, 4), Square::at(0, 5)),
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn pawn_moves_blocked() {
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(0, 6));
        board.piece_list.push(PieceKind::Dummy.colored(Color::White).at(0, 4));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(5, 3));
        board.piece_list.push(PieceKind::Dummy.colored(Color::White).at(5, 2));
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(2, 1));
        board.piece_list.push(PieceKind::Dummy.colored(Color::White).at(2, 2));
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(3, 1));
        board.piece_list.push(PieceKind::Dummy.colored(Color::White).at(3, 3));

        let expected_moves = vec!(
            Move::from_to(Square::at(3, 1), Square::at(3, 2))
        );
        assert_eq!(generate_moves(&board), expected_moves);

        board.side = Color::Black;
        let expected_moves = vec!(
            Move::from_to(Square::at(0, 6), Square::at(0, 5))
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn pawn_moves_capture() {
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(0, 6));
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 5));
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(1, 5));
        let expected_moves = vec!(
            Move::from_to_capture(Square::at(0, 6), Square::at(1, 5), PieceKind::Pawn.colored(Color::White).at(1, 5)),
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn pawn_moves_en_passant() {
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(1, 4));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(2, 4));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(4, 3));
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(5, 3));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(7, 3));

        board.en_passant = Some(Square::at(2, 5));
        let mut expected_moves = vec!(
            Move::from_to(Square::at(1, 4), Square::at(1, 5)),
            Move::from_to_capture(Square::at(1, 4), Square::at(2, 5), PieceKind::Pawn.colored(Color::Black).at(2, 4)),
            Move::from_to(Square::at(5, 3), Square::at(5, 4))
        );
        for mut move_ in expected_moves.iter_mut() {
            move_.en_passant_before = board.en_passant;
        }
        assert_eq!(generate_moves(&board), expected_moves);

        board.side = Color::Black;
        board.en_passant = Some(Square::at(5, 2));
        let mut expected_moves = vec!(
            Move::from_to(Square::at(2, 4), Square::at(2, 3)),
            Move::from_to(Square::at(4, 3), Square::at(4, 2)),
            Move::from_to_capture(Square::at(4, 3), Square::at(5, 2), PieceKind::Pawn.colored(Color::White).at(5, 3)),
            Move::from_to(Square::at(7, 3), Square::at(7, 2)),
        );
        for mut move_ in expected_moves.iter_mut() {
            move_.en_passant_before = board.en_passant;
        }
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn pawn_moves_promotion() {
        // White pawn that can promote
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(1, 6));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(2, 7));
        let expected_moves = vec!(
            Move::promotion(Square::at(1, 6), Square::at(1, 7), PieceKind::Knight),
            Move::promotion(Square::at(1, 6), Square::at(1, 7), PieceKind::Bishop),
            Move::promotion(Square::at(1, 6), Square::at(1, 7), PieceKind::Rook),
            Move::promotion(Square::at(1, 6), Square::at(1, 7), PieceKind::Queen),
            Move::promotion_capture(Square::at(1, 6), Square::at(2, 7), PieceKind::Pawn.colored(Color::Black).at(2, 7), PieceKind::Knight),
            Move::promotion_capture(Square::at(1, 6), Square::at(2, 7), PieceKind::Pawn.colored(Color::Black).at(2, 7), PieceKind::Bishop),
            Move::promotion_capture(Square::at(1, 6), Square::at(2, 7), PieceKind::Pawn.colored(Color::Black).at(2, 7), PieceKind::Rook),
            Move::promotion_capture(Square::at(1, 6), Square::at(2, 7), PieceKind::Pawn.colored(Color::Black).at(2, 7), PieceKind::Queen),
        );
        assert_eq!(generate_moves(&board), expected_moves);

        // Black pawn that can promote
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 1));
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(2, 0));
        let expected_moves = vec!(
            Move::promotion(Square::at(1, 1), Square::at(1, 0), PieceKind::Knight),
            Move::promotion(Square::at(1, 1), Square::at(1, 0), PieceKind::Bishop),
            Move::promotion(Square::at(1, 1), Square::at(1, 0), PieceKind::Rook),
            Move::promotion(Square::at(1, 1), Square::at(1, 0), PieceKind::Queen),
            Move::promotion_capture(Square::at(1, 1), Square::at(2, 0), PieceKind::Pawn.colored(Color::White).at(2, 0), PieceKind::Knight),
            Move::promotion_capture(Square::at(1, 1), Square::at(2, 0), PieceKind::Pawn.colored(Color::White).at(2, 0), PieceKind::Bishop),
            Move::promotion_capture(Square::at(1, 1), Square::at(2, 0), PieceKind::Pawn.colored(Color::White).at(2, 0), PieceKind::Rook),
            Move::promotion_capture(Square::at(1, 1), Square::at(2, 0), PieceKind::Pawn.colored(Color::White).at(2, 0), PieceKind::Queen),
        );
        assert_eq!(generate_moves(&board), expected_moves);

    }

    #[test]
    fn rook_moves() {
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Rook.colored(Color::White).at(3, 3));
        board.piece_list.push(PieceKind::Dummy.colored(Color::White).at(3, 5));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 3));

        let expected_moves = vec!(
            Move::from_to(Square::at(3, 3), Square::at(4, 3)),
            Move::from_to(Square::at(3, 3), Square::at(5, 3)),
            Move::from_to(Square::at(3, 3), Square::at(6, 3)),
            Move::from_to(Square::at(3, 3), Square::at(7, 3)),
            Move::from_to(Square::at(3, 3), Square::at(2, 3)),
            Move::from_to_capture(Square::at(3, 3), Square::at(1, 3), PieceKind::Pawn.colored(Color::Black).at(1, 3)),
            Move::from_to(Square::at(3, 3), Square::at(3, 4)),
            Move::from_to(Square::at(3, 3), Square::at(3, 2)),
            Move::from_to(Square::at(3, 3), Square::at(3, 1)),
            Move::from_to(Square::at(3, 3), Square::at(3, 0))
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn bishop_moves() {
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Bishop.colored(Color::White).at(3, 3));
        board.piece_list.push(PieceKind::Dummy.colored(Color::White).at(1, 1));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 5));

        let expected_moves = vec!(
            Move::from_to(Square::at(3, 3), Square::at(4, 4)),
            Move::from_to(Square::at(3, 3), Square::at(5, 5)),
            Move::from_to(Square::at(3, 3), Square::at(6, 6)),
            Move::from_to(Square::at(3, 3), Square::at(7, 7)),
            Move::from_to(Square::at(3, 3), Square::at(2, 4)),
            Move::from_to_capture(Square::at(3, 3), Square::at(1, 5), PieceKind::Pawn.colored(Color::Black).at(1, 5)),
            Move::from_to(Square::at(3, 3), Square::at(2, 2)),
            Move::from_to(Square::at(3, 3), Square::at(4, 2)),
            Move::from_to(Square::at(3, 3), Square::at(5, 1)),
            Move::from_to(Square::at(3, 3), Square::at(6, 0)),
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn queen_moves() {
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Queen.colored(Color::White).at(3, 3));
        board.piece_list.push(PieceKind::Dummy.colored(Color::White).at(1, 1));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 5));
        board.piece_list.push(PieceKind::Dummy.colored(Color::White).at(3, 5));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 3));

        let expected_moves = vec!(
            Move::from_to(Square::at(3, 3), Square::at(4, 3)),
            Move::from_to(Square::at(3, 3), Square::at(5, 3)),
            Move::from_to(Square::at(3, 3), Square::at(6, 3)),
            Move::from_to(Square::at(3, 3), Square::at(7, 3)),
            Move::from_to(Square::at(3, 3), Square::at(2, 3)),
            Move::from_to_capture(Square::at(3, 3), Square::at(1, 3), PieceKind::Pawn.colored(Color::Black).at(1, 3)),
            Move::from_to(Square::at(3, 3), Square::at(3, 4)),
            Move::from_to(Square::at(3, 3), Square::at(3, 2)),
            Move::from_to(Square::at(3, 3), Square::at(3, 1)),
            Move::from_to(Square::at(3, 3), Square::at(3, 0)),
            Move::from_to(Square::at(3, 3), Square::at(4, 4)),
            Move::from_to(Square::at(3, 3), Square::at(5, 5)),
            Move::from_to(Square::at(3, 3), Square::at(6, 6)),
            Move::from_to(Square::at(3, 3), Square::at(7, 7)),
            Move::from_to(Square::at(3, 3), Square::at(2, 4)),
            Move::from_to_capture(Square::at(3, 3), Square::at(1, 5), PieceKind::Pawn.colored(Color::Black).at(1, 5)),
            Move::from_to(Square::at(3, 3), Square::at(2, 2)),
            Move::from_to(Square::at(3, 3), Square::at(4, 2)),
            Move::from_to(Square::at(3, 3), Square::at(5, 1)),
            Move::from_to(Square::at(3, 3), Square::at(6, 0)),
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn king_moves() {
        // Freestanding King
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::King.colored(Color::White).at(3, 2));
        let expected_moves = vec!(
            Move::from_to(Square::at(3, 2), Square::at(4, 2)),
            Move::from_to(Square::at(3, 2), Square::at(2, 2)),
            Move::from_to(Square::at(3, 2), Square::at(3, 3)),
            Move::from_to(Square::at(3, 2), Square::at(3, 1)),
            Move::from_to(Square::at(3, 2), Square::at(4, 3)),
            Move::from_to(Square::at(3, 2), Square::at(2, 3)),
            Move::from_to(Square::at(3, 2), Square::at(2, 1)),
            Move::from_to(Square::at(3, 2), Square::at(4, 1))
        );
        assert_eq!(generate_moves(&board), expected_moves);

        // Blocked and capturing king at the edge of the board
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::King.colored(Color::White).at(3, 0));
        board.piece_list.push(PieceKind::Dummy.colored(Color::White).at(4, 0));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(2, 1));
        let expected_moves = vec!(
            Move::from_to(Square::at(3, 0), Square::at(2, 0)),
            Move::from_to(Square::at(3, 0), Square::at(3, 1)),
            Move::from_to(Square::at(3, 0), Square::at(4, 1)),
            Move::from_to_capture(Square::at(3, 0), Square::at(2, 1), PieceKind::Pawn.colored(Color::Black).at(2, 1))
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn knight_moves() {
        // Freestanding and capturing knight
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Knight.colored(Color::White).at(3, 4));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(4, 3));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(4, 4));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(5, 3));
        let expected_moves = vec!(
            Move::from_to(Square::at(3, 4), Square::at(1, 3)),
            Move::from_to(Square::at(3, 4), Square::at(2, 2)),
            Move::from_to(Square::at(3, 4), Square::at(4, 2)),
            Move::from_to_capture(Square::at(3, 4), Square::at(5, 3), PieceKind::Pawn.colored(Color::Black).at(5, 3)),
            Move::from_to(Square::at(3, 4), Square::at(5, 5)),
            Move::from_to(Square::at(3, 4), Square::at(4, 6)),
            Move::from_to(Square::at(3, 4), Square::at(2, 6)),
            Move::from_to(Square::at(3, 4), Square::at(1, 5))
        );
        assert_eq!(generate_moves(&board), expected_moves);

        // Blocked knight at the edge of the board
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Knight.colored(Color::White).at(0, 7));
        board.piece_list.push(PieceKind::Dummy.colored(Color::White).at(1, 5));
        let expected_moves = vec!(
            Move::from_to(Square::at(0, 7), Square::at(2, 6))
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn board_apply_and_revert_move() {
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 1));
        let mut move_ = Move::from_to(Square::at(0, 1), Square::at(0, 2));

        // Apply the move
        board.apply_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.side = Color::Black;
        expected_board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 2));
        assert_eq!(board, expected_board);

        // Revert the move
        board.revert_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 1));
        assert_eq!(board, expected_board);
    }

    #[test]
    fn board_apply_and_revert_move_with_capture() {
        let mut board = Board::create_empty();
        let mut move_ = Move::from_to_capture(Square::at(0, 1), Square::at(1, 2), PieceKind::Pawn.colored(Color::Black).at(1, 2));

        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 1));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 2));

        // Apply the move
        board.apply_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.side = Color::Black;
        expected_board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(1, 2));

        assert_eq!(board, expected_board);

        // Revert the move
        board.revert_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 1));
        expected_board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 2));

        assert_eq!(board, expected_board);
    }

    #[test]
    fn board_apply_and_revert_move_with_en_passant_square() {
        let mut board = Board::create_empty();
        let mut move_ = Move::from_to(Square::at(2, 4), Square::at(2, 5));
        move_.en_passant_before = Some(Square::at(4, 2));

        board.en_passant = Some(Square::at(4, 2));
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(2, 4));

        // Apply the move
        board.apply_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.side = Color::Black;
        expected_board.en_passant = None;
        expected_board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(2, 5));

        assert_eq!(board, expected_board);

        // Revert the move
        board.revert_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.en_passant = Some(Square::at(4, 2));
        expected_board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(2, 4));

        assert_eq!(board, expected_board);
    }

    #[test]
    fn board_apply_and_revert_move_with_en_passant_capture() {
        let mut board = Board::create_empty();
        let mut move_ = Move::from_to_capture(Square::at(2, 4), Square::at(1, 5), PieceKind::Pawn.colored(Color::Black).at(1, 4));

        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 4));
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(2, 4));

        // Apply the move
        board.apply_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.side = Color::Black;
        expected_board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(1, 5));

        assert_eq!(board, expected_board);

        // Revert the move
        board.revert_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(2, 4));
        expected_board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 4));

        assert_eq!(board, expected_board);
    }

    #[test]
    fn board_apply_and_revert_move_with_promotion() {
        let mut board = Board::create_empty();
        let mut move_ = Move::promotion(Square::at(1, 6), Square::at(1, 7), PieceKind::Bishop);

        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(1, 6));

        // Apply the move
        board.apply_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.side = Color::Black;
        expected_board.piece_list.push(PieceKind::Bishop.colored(Color::White).at(1, 7));

        assert_eq!(board, expected_board);

        // Revert the move
        board.revert_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(1, 6));

        assert_eq!(board, expected_board);
    }

    #[test]
    fn board_apply_and_revert_move_with_capture_and_promotion() {
        let mut board = Board::create_empty();
        let mut move_ = Move::promotion_capture(Square::at(1, 6), Square::at(2, 7), PieceKind::Pawn.colored(Color::Black).at(2, 7), PieceKind::Bishop);

        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(1, 6));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(2, 7));

        // Apply the move
        board.apply_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.side = Color::Black;
        expected_board.piece_list.push(PieceKind::Bishop.colored(Color::White).at(2, 7));

        assert_eq!(board, expected_board);

        // Revert the move
        board.revert_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(1, 6));
        expected_board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(2, 7));

        assert_eq!(board, expected_board);
    }

    #[test]
    fn static_evaluation_basic() {
        let mut board = Board::create_empty();

        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 1));
        assert_eq!(static_evaluation(&board), 1.0);

        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(0, 2));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(0, 3));
        assert_eq!(static_evaluation(&board), -1.0);
    }

    #[test]
    fn minimax_basic() {
        let minimax = |board: &mut Board, depth: u32, neg: f32| {
            MinimaxEvaluator::create().minimax(board, depth, neg)
        };

        // Just a white pawn
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 1));
        assert_eq!(minimax(&mut board, 3, 1.0), 1.0);

        // Just a black pawn
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(0, 6));
        assert_eq!(minimax(&mut board, 3, -1.0), -1.0);

        // A white pawn that can capture a black pawn
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 1));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 2));
        assert_eq!(minimax(&mut board, 3, 1.0), 1.0);

        // A black pawn that can capture a white pawn
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 2));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 3));
        assert_eq!(minimax(&mut board, 3, -1.0), -1.0);

        // A white pawn that can capture a black pawn and another black pawn
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 1));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 2));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(3, 2));
        assert_eq!(minimax(&mut board, 3, 1.0), 0.0);

        // A white pawn that will be capture by a black pawn after it moves
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 4));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 6));
        assert_eq!(minimax(&mut board, 3, 1.0), -1.0);

        // A white pawn that will capture a black pawn after the black pawn moves
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 3));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 5));
        assert_eq!(minimax(&mut board, 3, -1.0), 1.0);

        // A white pawn that will be captured by a black pawn after a couple of moves
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 2));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(1, 5));
        assert_eq!(minimax(&mut board, 10, -1.0), -1.0);

        // ...
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(0, 3));
        board.piece_list.push(PieceKind::Pawn.colored(Color::White).at(1, 5));
        board.piece_list.push(PieceKind::Pawn.colored(Color::Black).at(0, 6));
        assert_eq!(minimax(&mut board, 10, -1.0), -1.0);
    }
}

fn play(board: &mut Board) {
    let mut num_moves = 0;

    let max_depth = 5;

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
            board.apply_move(*move_);
            let evaluation = evaluator.evaluate(board, max_depth) * neg;
            board.revert_move(*move_);

            //println!("Evaluating {:?} with {}", move_, evaluation * neg);
            if evaluation > best_move_evaluation {
                best_move = Some(move_);
                best_move_evaluation = evaluation;
            }
        }

        let nodes_per_second = evaluator.statistics.node_count as f32 / evaluator.statistics.duration.as_secs_f32();
        let best_move = best_move.unwrap();

        println!("Chose move {:?} with an evaluation of {}, evaluated {} nodes/s", best_move, best_move_evaluation * neg, nodes_per_second);

        board.apply_move(*best_move);

        num_moves += 1;
        if num_moves > 50 {
            println!("Too many moves, aborting game");
            break;
        }

        println!();
    }
}

fn main() {
    let mut board = Board::create_populated();
    play(&mut board);
}
