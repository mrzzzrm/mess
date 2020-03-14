use super::core::*;
use super::board::*;
use super::move_::*;

pub struct TestMove {}

impl TestMove {
    // Wrapper around Move::from_to() that performs a lookup of the PieceKind
    pub fn from_to(board: &Board, from: Square, to: Square) -> Move {
        Move::from_to(board, board.piece_at(from).unwrap().kind, from, to)
    }

    pub fn castle(board: &Board, color: Color, castle: Castle) -> Move {
        Move::castle(board, color, castle)
    }

    pub fn promotion(board: &Board, from: Square, to: Square, promotion: PieceKind) -> Move {
        Move::promotion(board, from, to, promotion)
    }

    pub fn promotion_capture(board: &Board, from: Square, to: Square, capture: PieceOnBoard, promotion: PieceKind) -> Move {
        Move::promotion_capture(board, from, to, capture, promotion)
    }

    pub fn from_to_en_passant(board: &Board, from: Square, to: Square, en_passant: Square) -> Move {
        Move::from_to_en_passant(board, from, to, en_passant)
    }

    // Wrapper around Move::from_to_capture() that performs a lookup of the PieceKind
    pub fn from_to_capture(board: &Board, from: Square, to: Square, capture: PieceOnBoard) -> Move {
        Move::from_to_capture(board, board.piece_at(from).unwrap().kind, from, to, capture)
    }
}