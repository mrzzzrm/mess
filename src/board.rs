use super::core::*;
use super::move_::*;
use super::move_generation::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Board {
    pub piece_list: Vec<PieceOnBoard>,
    pub side: Color,
    pub en_passant: Option<Square>,
    pub castle_rights: BoardCastleRights,
}

impl Board {
    pub fn create_empty() -> Board {
        Board {
            piece_list: Vec::new(),
            side: Color::White,
            en_passant: None,
            castle_rights: BoardCastleRights::none(),
        }
    }

    pub fn create_populated() -> Board {
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

        board.castle_rights = BoardCastleRights::all();

        return board;
    }

    pub fn create_king_rooks() -> Board {
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Rook.colored(Color::White).at(0, 0));
        board.piece_list.push(PieceKind::Rook.colored(Color::White).at(7, 0));
        board.piece_list.push(PieceKind::Rook.colored(Color::Black).at(0, 7));
        board.piece_list.push(PieceKind::Rook.colored(Color::Black).at(7, 7));
        board.piece_list.push(PieceKind::King.colored(Color::White).at(4, 0));
        board.piece_list.push(PieceKind::King.colored(Color::Black).at(4, 7));

        board.castle_rights = BoardCastleRights::all();

        return board;
    }

    pub fn create_rooks() -> Board {
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Rook.colored(Color::White).at(7, 0));
        board.piece_list.push(PieceKind::Rook.colored(Color::Black).at(7, 7));

        board.castle_rights = BoardCastleRights::none();

        return board;
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        for (piece, square2) in self.piece_list.iter() {
            if square == *square2 {
                return Some(*piece);
            }
        }
        return None;
    }

    pub fn piece_at_mut(&mut self, square: Square) -> &mut PieceOnBoard {
        for piece_on_board in self.piece_list.iter_mut() {
            if square == piece_on_board.1 {
                return piece_on_board;
            }
        }
        panic!("No piece found on {:?}", square);
    }

    pub fn has_piece_at(&self, square: Square) -> bool {
        self.piece_list.iter().position(|(_, square2)| square == *square2).is_some()
    }

    fn apply_move_impl(&mut self, m: Move) {
        assert_eq!(self.piece_at(m.from).unwrap().kind, m.piece_kind);

        if let Some(castle) = m.castle {
            self.apply_move_impl(Move::rook_castle(self, castle, m.from.rank()));
        }

        let piece_on_board = self.piece_at_mut(m.from);
        if let Some(promotion) = m.promotion {
            piece_on_board.0.kind = promotion;
        }
        piece_on_board.1 = m.to;
    }

    pub fn apply_move(&mut self, m: Move) {
        assert_eq!(m.en_passant_before, self.en_passant);

        // Capture the piece on the target square, if any
        if let Some(capture) = m.capture {
            let pos = self.piece_list.iter().position(|&x| x.1 == capture.1).unwrap();
            assert_eq!(capture, self.piece_list[pos]);
            self.piece_list.remove(pos);
        } else {
            assert!(!self.has_piece_at(m.to));
        }

        self.apply_move_impl(m);
        self.en_passant = m.en_passant_after;
        self.castle_rights = m.castle_rights_after(self.side);
        self.side = self.side.switch();
    }

    fn revert_move_impl(&mut self, m: Move) {
        if let Some(promotion) = m.promotion {
            assert_eq!(self.piece_at(m.to).unwrap().kind, promotion);
        } else {
            assert_eq!(self.piece_at(m.to).unwrap().kind, m.piece_kind);
        }

        if let Some(castle) = m.castle {
            self.revert_move_impl(Move::rook_castle(self, castle, m.from.rank()));
        }

        let piece_on_board = self.piece_at_mut(m.to);
        if m.promotion.is_some() {
            piece_on_board.0.kind = PieceKind::Pawn;
        }
        piece_on_board.1 = m.from;
    }

    pub fn revert_move(&mut self, m: Move) {
        self.revert_move_impl(m);

        // Revert capture, if any
        if let Some(capture) = m.capture {
            assert!(!self.has_piece_at(capture.1));
            self.piece_list.push(capture);
        }

        self.side = self.side.switch();
        self.en_passant = m.en_passant_before;
        self.castle_rights = m.castle_rights_before;
    }

    pub fn is_game_over(&self) -> bool {
        generate_moves(self).is_empty()
    }

    pub fn king_square(&self, color: Color) -> Option<Square> {
        for (piece, square) in self.piece_list.iter() {
            if piece.kind == PieceKind::King && piece.color == color {
                return Some(*square);
            }
        }
        None
    }

    pub fn print(&self) {
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

    pub fn semantic_eq(&self, other: &Self) -> bool {
        if self.side != other.side || self.en_passant != other.en_passant || self.castle_rights != other.castle_rights {
            return false;
        }

        let mut self_sorted = self.piece_list.clone();
        self_sorted.sort();
        let mut other_sorted = other.piece_list.clone();
        other_sorted.sort();

        return self_sorted == other_sorted;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::*;

    #[test]
    fn board_apply_and_revert_move() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 1));
        let mut move_ = TestMove::from_to(&board, Square::at(0, 1), Square::at(0, 2));
        let original_board = board.clone();

        // Apply the move
        board.apply_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.side = Color::Black;
        expected_board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 2));
        assert_eq!(board, expected_board);

        // Revert the move
        board.revert_move(move_);
        assert_eq!(board, original_board);
    }

    #[test]
    fn board_apply_and_revert_move_with_capture() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 1),
            PieceKind::Pawn.colored(Color::Black).at(1, 2));
        let original_board = board.clone();

        let mut move_ = TestMove::from_to_capture(&board, Square::at(0, 1), Square::at(1, 2), PieceKind::Pawn.colored(Color::Black).at(1, 2));


        // Apply the move
        board.apply_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.side = Color::Black;
        expected_board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(1, 2));

        assert_eq!(board, expected_board);

        // Revert the move
        board.revert_move(move_);
        assert_eq!(board, original_board);
    }

    #[test]
    fn board_apply_and_revert_move_with_en_passant_square() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(2, 4));
        board.en_passant = Some(Square::at(4, 2));

        let original_board = board.clone();

        let mut move_ = TestMove::from_to(&board, Square::at(2, 4), Square::at(2, 5));

        // Apply the move
        board.apply_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.side = Color::Black;
        expected_board.en_passant = None;
        expected_board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(2, 5));

        assert_eq!(board, expected_board);

        // Revert the move
        board.revert_move(move_);
        assert!(board.semantic_eq(&original_board));
    }

    #[test]
    fn board_apply_and_revert_move_with_en_passant_capture() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::Black).at(1, 4),
            PieceKind::Pawn.colored(Color::White).at(2, 4));
        let original_board = board.clone();

        let mut move_ = TestMove::from_to_capture(&board, Square::at(2, 4), Square::at(1, 5), PieceKind::Pawn.colored(Color::Black).at(1, 4));

        // Apply the move
        board.apply_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.side = Color::Black;
        expected_board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(1, 5));

        assert_eq!(board, expected_board);

        // Revert the move
        board.revert_move(move_);
        assert!(board.semantic_eq(&original_board));
    }

    #[test]
    fn board_apply_and_revert_move_with_promotion() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(1, 6));
        let original_board = board.clone();

        let mut move_ = TestMove::promotion(&board, Square::at(1, 6), Square::at(1, 7), PieceKind::Bishop);

        // Apply the move
        board.apply_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.side = Color::Black;
        expected_board.piece_list = vec!(
            PieceKind::Bishop.colored(Color::White).at(1, 7));

        assert_eq!(board, expected_board);

        // Revert the move
        board.revert_move(move_);
        assert_eq!(board, original_board);
    }

    #[test]
    fn board_apply_and_revert_move_with_capture_and_promotion() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(1, 6),
            PieceKind::Pawn.colored(Color::Black).at(2, 7));
        let original_board = board.clone();

        let mut move_ = TestMove::promotion_capture(&board, Square::at(1, 6), Square::at(2, 7), PieceKind::Pawn.colored(Color::Black).at(2, 7), PieceKind::Bishop);

        // Apply the move
        board.apply_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.side = Color::Black;
        expected_board.piece_list = vec!(
            PieceKind::Bishop.colored(Color::White).at(2, 7));

        assert_eq!(board, expected_board);

        // Revert the move
        board.revert_move(move_);
        assert_eq!(board, original_board);
    }

    #[test]
    fn board_apply_and_revert_king_side_castling() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::King.colored(Color::White).at(4, 0),
            PieceKind::Rook.colored(Color::White).at(0, 0),
            PieceKind::Rook.colored(Color::White).at(7, 0));
        board.castle_rights = BoardCastleRights::all();
        let original_board = board.clone();

        let mut move_ = TestMove::castle(&board, Color::White, Castle::KingSide);

        // Apply the move
        board.apply_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.side = Color::Black;
        expected_board.castle_rights.white = ColorCastleRights::none();
        expected_board.castle_rights.black = ColorCastleRights::all();
        expected_board.piece_list = vec!(
            PieceKind::King.colored(Color::White).at(6, 0),
            PieceKind::Rook.colored(Color::White).at(0, 0),
            PieceKind::Rook.colored(Color::White).at(5, 0));
        assert_eq!(board, expected_board);

        // Revert the move
        board.revert_move(move_);
        assert_eq!(board, original_board);
    }

    #[test]
    fn board_apply_and_revert_queen_side_castling() {
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list = vec!(
            PieceKind::King.colored(Color::Black).at(4, 7),
            PieceKind::Rook.colored(Color::Black).at(0, 7),
            PieceKind::Rook.colored(Color::Black).at(7, 7));
        board.castle_rights = BoardCastleRights::all();
        let original_board = board.clone();

        let mut move_ = TestMove::castle(&board, Color::Black, Castle::QueenSide);

        // Apply the move
        board.apply_move(move_);

        let mut expected_board = Board::create_empty();
        expected_board.castle_rights.white = ColorCastleRights::all();
        expected_board.castle_rights.black = ColorCastleRights::none();
        expected_board.piece_list = vec!(
            PieceKind::King.colored(Color::Black).at(2, 7),
            PieceKind::Rook.colored(Color::Black).at(3, 7),
            PieceKind::Rook.colored(Color::Black).at(7, 7));
        assert_eq!(board, expected_board);

        // Revert the move
        board.revert_move(move_);
        assert_eq!(board, original_board);
    }

    #[test]
    fn board_apply_and_revert_castle_rights_loss_through_normal_move() {
        let mut board = Board::create_empty();
        board.castle_rights = BoardCastleRights::all();
        board.piece_list = vec!(
            PieceKind::Rook.colored(Color::White).at(0, 0),
            PieceKind::King.colored(Color::White).at(4, 0),
            PieceKind::Rook.colored(Color::White).at(7, 0));
        let original_board = board.clone();

        // Moving the king-side Rook looses queen side castle rights
        let move_ = TestMove::from_to(&board, Square::at(0, 0), Square::at(0, 1));

        board.apply_move(move_);
        let mut expected_castle_rights = BoardCastleRights::all();
        expected_castle_rights.white.queen_side = false;
        assert_eq!(board.castle_rights, expected_castle_rights);

        board.revert_move(move_);
        let mut expected_castle_rights = BoardCastleRights::all();
        assert_eq!(board.castle_rights, expected_castle_rights);

        // Moving the queen-side Rook looses king side castle rights
        let move_ = TestMove::from_to(&board, Square::at(7, 0), Square::at(7, 1));

        board.apply_move(move_);
        let mut expected_castle_rights = BoardCastleRights::all();
        expected_castle_rights.white.king_side = false;
        assert_eq!(board.castle_rights, expected_castle_rights);

        board.revert_move(move_);
        let mut expected_castle_rights = BoardCastleRights::all();
        assert_eq!(board.castle_rights, expected_castle_rights);

        // Moving the King looses castle rights on both sides
        let move_ = TestMove::from_to(&board, Square::at(4, 0), Square::at(3, 1));

        board.apply_move(move_);
        let mut expected_castle_rights = BoardCastleRights::all();
        expected_castle_rights.white = ColorCastleRights::none();
        assert_eq!(board.castle_rights, expected_castle_rights);

        board.revert_move(move_);
        let mut expected_castle_rights = BoardCastleRights::all();
        assert_eq!(board.castle_rights, expected_castle_rights);
    }

    #[test]
    fn board_apply_and_revert_castle_rights_loss_through_capture() {
        let mut board = Board::create_empty();
        board.castle_rights = BoardCastleRights::all();
        board.piece_list = vec!(
            PieceKind::Rook.colored(Color::Black).at(0, 7),
            PieceKind::King.colored(Color::Black).at(4, 7),
            PieceKind::Rook.colored(Color::Black).at(7, 7),
            PieceKind::Pawn.colored(Color::White).at(1, 6),
            PieceKind::Pawn.colored(Color::White).at(6, 6));
        let original_board = board.clone();

        // Moving the king-side Rook looses queen side castle rights
        let queen_side_capture = TestMove::from_to_capture(&board, Square::at(1, 6), Square::at(0, 7), PieceKind::Rook.colored(Color::Black).at(0, 7));

        board.apply_move(queen_side_capture);
        let mut expected_castle_rights = BoardCastleRights::all();
        expected_castle_rights.black.queen_side = false;
        assert_eq!(board.castle_rights, expected_castle_rights);

        board.revert_move(queen_side_capture);
        let mut expected_castle_rights = BoardCastleRights::all();
        assert_eq!(board.castle_rights, expected_castle_rights);

        // Moving the queen-side Rook looses king side castle rights
        let queen_side_capture = TestMove::from_to_capture(&board, Square::at(6, 6), Square::at(7, 7), PieceKind::Rook.colored(Color::Black).at(7, 7));

        board.apply_move(queen_side_capture);
        let mut expected_castle_rights = BoardCastleRights::all();
        expected_castle_rights.black.king_side = false;
        assert_eq!(board.castle_rights, expected_castle_rights);

        board.revert_move(queen_side_capture);
        let mut expected_castle_rights = BoardCastleRights::all();
        assert_eq!(board.castle_rights, expected_castle_rights);
    }

    #[test]
    fn board_apply_and_revert_no_castle_rights() {
        // Test that with no castle rights to begin with, reverting a move that would loose castle
        // rights doesn't accidentally grant them.

        let mut board = Board::create_empty();
        board.castle_rights = BoardCastleRights::none();
        board.piece_list = vec!(
            PieceKind::Rook.colored(Color::White).at(0, 0),
            PieceKind::King.colored(Color::White).at(4, 0),
            PieceKind::Rook.colored(Color::White).at(7, 0));

        // Moving the king-side Rook
        let move_ = TestMove::from_to(&board, Square::at(0, 0), Square::at(0, 1));
        board.apply_move(move_);
        assert_eq!(board.castle_rights, BoardCastleRights::none());

        board.revert_move(move_);
        assert_eq!(board.castle_rights, BoardCastleRights::none());

        // Moving the queen-side Rook looses king side castle rights
        let move_ = TestMove::from_to(&board, Square::at(7, 0), Square::at(7, 1));
        board.apply_move(move_);
        assert_eq!(board.castle_rights, BoardCastleRights::none());

        board.revert_move(move_);
        assert_eq!(board.castle_rights, BoardCastleRights::none());

        // Moving the King looses castle rights on both sides
        let move_ = TestMove::from_to(&board, Square::at(4, 0), Square::at(3, 1));
        board.apply_move(move_);
        assert_eq!(board.castle_rights, BoardCastleRights::none());

        board.revert_move(move_);
        assert_eq!(board.castle_rights, BoardCastleRights::none());
    }
}