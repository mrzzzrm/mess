use super::core::*;
use super::board::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece_kind: PieceKind,
    pub capture: Option<PieceOnBoard>,
    pub en_passant_before: Option<Square>,
    pub en_passant_after: Option<Square>,

    pub castle_rights_before: BoardCastleRights,

    pub castle: Option<Castle>,
    pub promotion: Option<PieceKind>,
}

impl Move {
    pub fn castle(board: &Board, color: Color, castle: Castle) -> Move {
        let file = match castle {
            Castle::KingSide => 6,
            Castle::QueenSide => 2,
        };

        let rank = color.back_rank();

        let mut m = Move::from_to(board, PieceKind::King, Square::at(4, rank), Square::at(file, rank));
        m.castle = Some(castle);

        return m;
    }

    pub fn promotion(board: &Board, from: Square, to: Square, promotion: PieceKind) -> Move {
        let mut m = Move::from_to(board, PieceKind::Pawn, from, to);
        m.promotion = Some(promotion);
        return m;
    }

    pub fn promotion_capture(board: &Board, from: Square, to: Square, capture: PieceOnBoard, promotion: PieceKind) -> Move {
        let mut m = Move::from_to(board, PieceKind::Pawn, from, to);
        m.capture = Some(capture);
        m.promotion = Some(promotion);
        return m;
    }

    pub fn from_to(board: &Board, piece_kind: PieceKind, from: Square, to: Square) -> Move {
        Move {
            piece_kind,
            from,
            to,
            capture: None,
            en_passant_before: board.en_passant,
            en_passant_after: None,
            castle_rights_before: board.castle_rights,
            castle: None,
            promotion: None,
        }
    }

    // Create a Move that creates an en-passant square
    pub fn from_to_en_passant(board: &Board, from: Square, to: Square, en_passant: Square) -> Move {
        let mut m = Move::from_to(board, PieceKind::Pawn, from, to);
        m.en_passant_after = Some(en_passant);
        return m;
    }

    pub fn from_to_capture(board: &Board, piece_kind: PieceKind, from: Square, to: Square, capture: PieceOnBoard) -> Move {
        let mut m = Move::from_to(board, piece_kind, from, to);
        m.capture = Some(capture);
        return m;
    }

    pub fn castle_rights_after(&self, side: Color) -> BoardCastleRights {
        let mut rights = self.castle_rights_before;
        let other_side = side.switch();

        match self.piece_kind {
            PieceKind::King => {
                rights.set_rights(side, &ColorCastleRights::none());
            }
            PieceKind::Rook => {
                if self.from == Square::at(7, side.back_rank()) {
                    rights.get_rights_mut(side).king_side = false;
                }
                if self.from == Square::at(0, side.back_rank()) {
                    rights.get_rights_mut(side).queen_side = false;
                }
            }
            _ => {}
        }

        if let Some(capture) = self.capture {
            if capture.1 == Square::at(7, other_side.back_rank()) {
                rights.get_rights_mut(other_side).king_side = false;
            }
            if capture.1 == Square::at(0, other_side.back_rank()) {
                rights.get_rights_mut(other_side).queen_side = false;
            }
        }

        return rights;
    }

    // Create the move a Rook makes during castling
    pub fn rook_castle(board: &Board, castle: Castle, rank: i8) -> Move {
        assert!(rank == 0 || rank == 7);

        return match castle {
            Castle::KingSide => {
                Move::from_to(board, PieceKind::Rook, Square::at(7, rank), Square::at(5, rank))
            }
            Castle::QueenSide => {
                Move::from_to(board, PieceKind::Rook, Square::at(0, rank), Square::at(3, rank))
            }
        };
    }

    pub fn long_algebraic(&self) -> String {
        format!("{}{}{}", self.from.algebraic(), "-", self.to.algebraic())
    }
}
