use super::core::*;
use super::board::*;
use super::move_::*;

// Add a move by x_delta, y_delta to the moves if the target square is on board and is unoccupied
// or can be captured. Return whether the target square was unoccupied.
fn probe_move(board: &Board, piece: &Piece, current_square: &Square, x_delta: i8, y_delta: i8, moves: &mut Vec<Move>) -> bool {
    let target_square = current_square.delta(x_delta, y_delta);
    if !target_square.is_on_board() {
        return false;
    }

    let target_piece = board.piece_at(target_square);

    return match target_piece {
        Some(target_piece) => {
            if target_piece.color == piece.color {
                false
            } else {
                moves.push(Move::from_to_capture(board, piece.kind, *current_square, target_square, (target_piece, target_square)));
                false
            }
        }
        None => {
            moves.push(Move::from_to(board, piece.kind, *current_square, target_square));
            true
        }
    };
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
fn generate_pawn_move(board: &Board, piece: &Piece, from: &Square, to: &Square, capture: &Option<PieceOnBoard>, moves: &mut Vec<Move>) {
    if to.rank() as u8 == piece.color.promotion_rank() {
        for promotion in &[PieceKind::Knight, PieceKind::Bishop, PieceKind::Rook, PieceKind::Queen] {
            if let Some(capture) = capture {
                moves.push(Move::promotion_capture(board, *from, *to, *capture, *promotion));
            } else {
                moves.push(Move::promotion(board, *from, *to, *promotion));
            }
        }
    } else {
        if let Some(capture) = capture {
            moves.push(Move::from_to_capture(board, PieceKind::Pawn, *from, *to, *capture));
        } else {
            moves.push(Move::from_to(board, PieceKind::Pawn, *from, *to));
        }
    }
}

pub fn generate_moves(board: &Board) -> Vec<Move> {
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
                    generate_pawn_move(board, piece, square, &square.delta(0, forward), &None, &mut moves);

                    if square.rank() == home_rank && !board.has_piece_at(square.delta(0, forward * 2)) && square.delta(0, forward * 2).is_on_board() {
                        moves.push(Move::from_to_en_passant(board, *square, square.delta(0, forward * 2), square.delta(0, forward)));
                    }
                }

                // Generate capture moves
                for file_delta in [-1 as i8, 1 as i8].iter() {
                    let target_piece = board.piece_at(square.delta(*file_delta, forward));

                    if target_piece.is_some() {
                        let target_piece = target_piece.unwrap();
                        if target_piece.color != piece.color {
                            generate_pawn_move(board, piece, square, &square.delta(*file_delta, forward), &Some((target_piece, square.delta(*file_delta, forward))), &mut moves);
                        }
                    }

                    if board.en_passant.is_some() && board.en_passant.unwrap() == square.delta(*file_delta, forward) {
                        let en_passant_piece = board.piece_at(square.delta(*file_delta, 0)).unwrap();
                        moves.push(Move::from_to_capture(board, piece.kind, *square, square.delta(*file_delta, forward), (en_passant_piece, square.delta(*file_delta, 0))));
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

                // Generate King side castle
                if board.castle_rights.get_rights(piece.color).test(Castle::KingSide) {
                    if !board.has_piece_at(Square::at(5, piece.color.back_rank() as i8)) &&
                        !board.has_piece_at(Square::at(6, piece.color.back_rank() as i8)) {
                        moves.push(Move::castle(board, piece.color, Castle::KingSide));
                    }
                }

                // Generate Queen side castle
                if board.castle_rights.get_rights(piece.color).test(Castle::QueenSide) {
                    if !board.has_piece_at(Square::at(3, piece.color.back_rank() as i8)) &&
                        !board.has_piece_at(Square::at(2, piece.color.back_rank() as i8)) &&
                        !board.has_piece_at(Square::at(1, piece.color.back_rank() as i8)) {
                        moves.push(Move::castle(board, piece.color, Castle::QueenSide));
                    }
                }
            }
            PieceKind::Knight => {
                for (x_delta, y_delta) in [(-2, -1), (-1, -2), (1, -2), (2, -1), (2, 1), (1, 2), (-1, 2), (-2, 1)].iter() {
                    probe_move(board, piece, square, *x_delta as i8, *y_delta as i8, &mut moves);
                }
            }
            PieceKind::Dummy => {}
        }
    }

    return moves;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::*;

    #[test]
    fn pawn_moves() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::Black).at(0, 6),
            PieceKind::Pawn.colored(Color::White).at(2, 1),
            PieceKind::Pawn.colored(Color::White).at(3, 2));

        let expected_moves = vec!(
            TestMove::from_to(&board, Square::at(2, 1), Square::at(2, 2)),
            TestMove::from_to_en_passant(&board, Square::at(2, 1), Square::at(2, 3), Square::at(2, 2)),
            TestMove::from_to(&board, Square::at(3, 2), Square::at(3, 3)),
        );
        assert_eq!(generate_moves(&board), expected_moves);

        board.side = Color::Black;
        let expected_moves = vec!(
            TestMove::from_to(&board, Square::at(0, 6), Square::at(0, 5)),
            TestMove::from_to_en_passant(&board, Square::at(0, 6), Square::at(0, 4), Square::at(0, 5)),
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn pawn_moves_blocked() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::Black).at(0, 6),
            PieceKind::Dummy.colored(Color::White).at(0, 4),
            PieceKind::Pawn.colored(Color::Black).at(5, 3),
            PieceKind::Dummy.colored(Color::White).at(5, 2),
            PieceKind::Pawn.colored(Color::White).at(2, 1),
            PieceKind::Dummy.colored(Color::White).at(2, 2),
            PieceKind::Pawn.colored(Color::White).at(3, 1),
            PieceKind::Dummy.colored(Color::White).at(3, 3));

        let expected_moves = vec!(
            TestMove::from_to(&board, Square::at(3, 1), Square::at(3, 2))
        );
        assert_eq!(generate_moves(&board), expected_moves);

        board.side = Color::Black;
        let expected_moves = vec!(
            TestMove::from_to(&board, Square::at(0, 6), Square::at(0, 5))
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn pawn_moves_capture() {
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::Black).at(0, 6),
            PieceKind::Pawn.colored(Color::White).at(0, 5),
            PieceKind::Pawn.colored(Color::White).at(1, 5), );
        let expected_moves = vec!(
            TestMove::from_to_capture(&board, Square::at(0, 6), Square::at(1, 5), PieceKind::Pawn.colored(Color::White).at(1, 5)),
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn pawn_moves_en_passant() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(1, 4),
            PieceKind::Pawn.colored(Color::Black).at(2, 4),
            PieceKind::Pawn.colored(Color::Black).at(4, 3),
            PieceKind::Pawn.colored(Color::White).at(5, 3),
            PieceKind::Pawn.colored(Color::Black).at(7, 3), );

        board.en_passant = Some(Square::at(2, 5));
        let mut expected_moves = vec!(
            TestMove::from_to(&board, Square::at(1, 4), Square::at(1, 5)),
            TestMove::from_to_capture(&board, Square::at(1, 4), Square::at(2, 5), PieceKind::Pawn.colored(Color::Black).at(2, 4)),
            TestMove::from_to(&board, Square::at(5, 3), Square::at(5, 4))
        );
        for mut move_ in expected_moves.iter_mut() {
            move_.en_passant_before = board.en_passant;
        }
        assert_eq!(generate_moves(&board), expected_moves);

        board.side = Color::Black;
        board.en_passant = Some(Square::at(5, 2));
        let mut expected_moves = vec!(
            TestMove::from_to(&board, Square::at(2, 4), Square::at(2, 3)),
            TestMove::from_to(&board, Square::at(4, 3), Square::at(4, 2)),
            TestMove::from_to_capture(&board, Square::at(4, 3), Square::at(5, 2), PieceKind::Pawn.colored(Color::White).at(5, 3)),
            TestMove::from_to(&board, Square::at(7, 3), Square::at(7, 2)),
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
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(1, 6),
            PieceKind::Pawn.colored(Color::Black).at(2, 7), );
        let expected_moves = vec!(
            TestMove::promotion(&board, Square::at(1, 6), Square::at(1, 7), PieceKind::Knight),
            TestMove::promotion(&board, Square::at(1, 6), Square::at(1, 7), PieceKind::Bishop),
            TestMove::promotion(&board, Square::at(1, 6), Square::at(1, 7), PieceKind::Rook),
            TestMove::promotion(&board, Square::at(1, 6), Square::at(1, 7), PieceKind::Queen),
            TestMove::promotion_capture(&board, Square::at(1, 6), Square::at(2, 7), PieceKind::Pawn.colored(Color::Black).at(2, 7), PieceKind::Knight),
            TestMove::promotion_capture(&board, Square::at(1, 6), Square::at(2, 7), PieceKind::Pawn.colored(Color::Black).at(2, 7), PieceKind::Bishop),
            TestMove::promotion_capture(&board, Square::at(1, 6), Square::at(2, 7), PieceKind::Pawn.colored(Color::Black).at(2, 7), PieceKind::Rook),
            TestMove::promotion_capture(&board, Square::at(1, 6), Square::at(2, 7), PieceKind::Pawn.colored(Color::Black).at(2, 7), PieceKind::Queen),
        );
        assert_eq!(generate_moves(&board), expected_moves);

        // Black pawn that can promote
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::Black).at(1, 1),
            PieceKind::Pawn.colored(Color::White).at(2, 0), );
        let expected_moves = vec!(
            TestMove::promotion(&board, Square::at(1, 1), Square::at(1, 0), PieceKind::Knight),
            TestMove::promotion(&board, Square::at(1, 1), Square::at(1, 0), PieceKind::Bishop),
            TestMove::promotion(&board, Square::at(1, 1), Square::at(1, 0), PieceKind::Rook),
            TestMove::promotion(&board, Square::at(1, 1), Square::at(1, 0), PieceKind::Queen),
            TestMove::promotion_capture(&board, Square::at(1, 1), Square::at(2, 0), PieceKind::Pawn.colored(Color::White).at(2, 0), PieceKind::Knight),
            TestMove::promotion_capture(&board, Square::at(1, 1), Square::at(2, 0), PieceKind::Pawn.colored(Color::White).at(2, 0), PieceKind::Bishop),
            TestMove::promotion_capture(&board, Square::at(1, 1), Square::at(2, 0), PieceKind::Pawn.colored(Color::White).at(2, 0), PieceKind::Rook),
            TestMove::promotion_capture(&board, Square::at(1, 1), Square::at(2, 0), PieceKind::Pawn.colored(Color::White).at(2, 0), PieceKind::Queen),
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn rook_moves() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Rook.colored(Color::White).at(3, 3),
            PieceKind::Dummy.colored(Color::White).at(3, 5),
            PieceKind::Pawn.colored(Color::Black).at(1, 3), );

        let expected_moves = vec!(
            TestMove::from_to(&board, Square::at(3, 3), Square::at(4, 3)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(5, 3)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(6, 3)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(7, 3)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(2, 3)),
            TestMove::from_to_capture(&board, Square::at(3, 3), Square::at(1, 3), PieceKind::Pawn.colored(Color::Black).at(1, 3)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(3, 4)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(3, 2)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(3, 1)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(3, 0))
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn bishop_moves() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Bishop.colored(Color::White).at(3, 3),
            PieceKind::Dummy.colored(Color::White).at(1, 1),
            PieceKind::Pawn.colored(Color::Black).at(1, 5), );

        let expected_moves = vec!(
            TestMove::from_to(&board, Square::at(3, 3), Square::at(4, 4)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(5, 5)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(6, 6)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(7, 7)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(2, 4)),
            TestMove::from_to_capture(&board, Square::at(3, 3), Square::at(1, 5), PieceKind::Pawn.colored(Color::Black).at(1, 5)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(2, 2)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(4, 2)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(5, 1)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(6, 0)),
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn queen_moves() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Queen.colored(Color::White).at(3, 3),
            PieceKind::Dummy.colored(Color::White).at(1, 1),
            PieceKind::Pawn.colored(Color::Black).at(1, 5),
            PieceKind::Dummy.colored(Color::White).at(3, 5),
            PieceKind::Pawn.colored(Color::Black).at(1, 3), );

        let expected_moves = vec!(
            TestMove::from_to(&board, Square::at(3, 3), Square::at(4, 3)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(5, 3)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(6, 3)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(7, 3)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(2, 3)),
            TestMove::from_to_capture(&board, Square::at(3, 3), Square::at(1, 3), PieceKind::Pawn.colored(Color::Black).at(1, 3)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(3, 4)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(3, 2)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(3, 1)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(3, 0)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(4, 4)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(5, 5)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(6, 6)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(7, 7)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(2, 4)),
            TestMove::from_to_capture(&board, Square::at(3, 3), Square::at(1, 5), PieceKind::Pawn.colored(Color::Black).at(1, 5)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(2, 2)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(4, 2)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(5, 1)),
            TestMove::from_to(&board, Square::at(3, 3), Square::at(6, 0)),
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn king_basic_moves() {
        // Freestanding King
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::King.colored(Color::White).at(3, 2), );
        let expected_moves = vec!(
            TestMove::from_to(&board, Square::at(3, 2), Square::at(4, 2)),
            TestMove::from_to(&board, Square::at(3, 2), Square::at(2, 2)),
            TestMove::from_to(&board, Square::at(3, 2), Square::at(3, 3)),
            TestMove::from_to(&board, Square::at(3, 2), Square::at(3, 1)),
            TestMove::from_to(&board, Square::at(3, 2), Square::at(4, 3)),
            TestMove::from_to(&board, Square::at(3, 2), Square::at(2, 3)),
            TestMove::from_to(&board, Square::at(3, 2), Square::at(2, 1)),
            TestMove::from_to(&board, Square::at(3, 2), Square::at(4, 1))
        );
        assert_eq!(generate_moves(&board), expected_moves);

        // Blocked and capturing king at the edge of the board
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::King.colored(Color::White).at(3, 0),
            PieceKind::Dummy.colored(Color::White).at(4, 0),
            PieceKind::Pawn.colored(Color::Black).at(2, 1));
        let expected_moves = vec!(
            TestMove::from_to(&board, Square::at(3, 0), Square::at(2, 0)),
            TestMove::from_to(&board, Square::at(3, 0), Square::at(3, 1)),
            TestMove::from_to(&board, Square::at(3, 0), Square::at(4, 1)),
            TestMove::from_to_capture(&board, Square::at(3, 0), Square::at(2, 1), PieceKind::Pawn.colored(Color::Black).at(2, 1))
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn king_castling_moves() {
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::King.colored(Color::White).at(4, 0),
            PieceKind::Rook.colored(Color::White).at(0, 0),
            PieceKind::Rook.colored(Color::White).at(7, 0));

        // No castle rights, no castle
        board.castle_rights = BoardCastleRights::none();
        assert!(!generate_moves(&board).contains(&TestMove::castle(&board, Color::White, Castle::KingSide)));
        assert!(!generate_moves(&board).contains(&TestMove::castle(&board, Color::White, Castle::QueenSide)));

        // Castle only where rights are granted
        board.castle_rights = BoardCastleRights::none();
        board.castle_rights.white.king_side = true;
        assert!(generate_moves(&board).contains(&TestMove::castle(&board, Color::White, Castle::KingSide)));
        assert!(!generate_moves(&board).contains(&TestMove::castle(&board, Color::White, Castle::QueenSide)));
        board.castle_rights.white.queen_side = true;
        assert!(generate_moves(&board).contains(&TestMove::castle(&board, Color::White, Castle::KingSide)));
        assert!(generate_moves(&board).contains(&TestMove::castle(&board, Color::White, Castle::QueenSide)));

        // If all castle rights for both side are granted, then castle
        board.castle_rights = BoardCastleRights::all();
        assert!(generate_moves(&board).contains(&TestMove::castle(&board, Color::White, Castle::KingSide)));
        assert!(generate_moves(&board).contains(&TestMove::castle(&board, Color::White, Castle::QueenSide)));
    }

    #[test]
    fn king_castling_moves_blocked() {
        let mut original_board = Board::create_empty();
        original_board.piece_list = vec!(
            PieceKind::King.colored(Color::Black).at(4, 7),
            PieceKind::Rook.colored(Color::Black).at(0, 7),
            PieceKind::Rook.colored(Color::Black).at(7, 7));
        original_board.side = Color::Black;
        original_board.castle_rights = BoardCastleRights::all();

        // No blockers added yet, we can still castle
        let mut board = original_board.clone();
        assert!(generate_moves(&board).contains(&TestMove::castle(&board, Color::Black, Castle::KingSide)));
        assert!(generate_moves(&board).contains(&TestMove::castle(&board, Color::Black, Castle::QueenSide)));

        // Blocker on the queen side, not on the king side
        let mut board = original_board.clone();
        board.piece_list.push(PieceKind::Dummy.colored(Color::Black).at(1, 7));
        assert!(generate_moves(&board).contains(&TestMove::castle(&board, Color::Black, Castle::KingSide)));
        assert!(!generate_moves(&board).contains(&TestMove::castle(&board, Color::Black, Castle::QueenSide)));

        // Blocker on the king side, not on the queen side
        let mut board = original_board.clone();
        board.piece_list.push(PieceKind::Dummy.colored(Color::White).at(5, 7));
        assert!(!generate_moves(&board).contains(&TestMove::castle(&board, Color::Black, Castle::KingSide)));
        assert!(generate_moves(&board).contains(&TestMove::castle(&board, Color::Black, Castle::QueenSide)));
    }

    #[test]
    fn knight_moves() {
        // Freestanding and capturing knight
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Knight.colored(Color::White).at(3, 4),
            PieceKind::Pawn.colored(Color::Black).at(4, 3),
            PieceKind::Pawn.colored(Color::Black).at(4, 4),
            PieceKind::Pawn.colored(Color::Black).at(5, 3));
        let expected_moves = vec!(
            TestMove::from_to(&board, Square::at(3, 4), Square::at(1, 3)),
            TestMove::from_to(&board, Square::at(3, 4), Square::at(2, 2)),
            TestMove::from_to(&board, Square::at(3, 4), Square::at(4, 2)),
            TestMove::from_to_capture(&board, Square::at(3, 4), Square::at(5, 3), PieceKind::Pawn.colored(Color::Black).at(5, 3)),
            TestMove::from_to(&board, Square::at(3, 4), Square::at(5, 5)),
            TestMove::from_to(&board, Square::at(3, 4), Square::at(4, 6)),
            TestMove::from_to(&board, Square::at(3, 4), Square::at(2, 6)),
            TestMove::from_to(&board, Square::at(3, 4), Square::at(1, 5))
        );
        assert_eq!(generate_moves(&board), expected_moves);

        // Blocked knight at the edge of the board
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Knight.colored(Color::White).at(0, 7),
            PieceKind::Dummy.colored(Color::White).at(1, 5));
        let expected_moves = vec!(
            TestMove::from_to(&board, Square::at(0, 7), Square::at(2, 6))
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }
}