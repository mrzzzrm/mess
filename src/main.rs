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
            Color::Black => Color::White,
            Color::White => Color::Black
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
        }
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
        return ( *self, Square { x: file, y: rank } );
    }
}

#[derive(Debug, PartialEq)]
struct Move {
    from: Square,
    to: Square,
    capture: Option<PieceOnBoard>,
}

type PieceOnBoard = (Piece, Square);

#[derive(Debug)]
struct Board {
    piece_list: Vec<PieceOnBoard>,
    side: Color,
}

impl Board {
    fn piece_at(&self, square: Square) -> Option<Piece> {
        for (piece, square2) in self.piece_list.iter() {
            if square == *square2 {
                return Some(*piece);
            }
        }
        return None;
    }

    fn apply_move(&mut self, m: &Move) {
        if m.capture.is_some() {
            let pos = self.piece_list.iter().position(|&x| x.1 == m.to).unwrap();
            self.piece_list.remove(pos);
        }

        for (piece, mut square) in self.piece_list.iter_mut() {
            if square == m.from {
                square = m.to;
            }
        }

        self.side = self.side.switch();
    }

    fn revert_move(&mut self, m: &Move) {
        for (piece, mut square) in self.piece_list.iter_mut() {
            if square == m.to {
                square = m.from;
            }
        }

        if m.capture.is_some() {
            let capture = m.capture.unwrap();
            self.piece_list.push(capture);
        }

        self.side = self.side.switch();
    }
}

fn init_board() -> Board {
    Board {
        piece_list: Vec::new(),
        side: Color::White,
    }
}

fn static_evaluation(board: &Board) -> f32 {
    let mut evaluation = 0.0;
    for (piece, _) in board.piece_list.iter() {
        evaluation += piece.kind.value();
    }
    return evaluation;
}

fn minimax(board: &mut Board, depth: u32, neg: f32) -> f32 {
    if depth == 0 {
        return static_evaluation(&board);
    }

    let moves = generate_moves(&board);
    if moves.is_empty() {
        return 0.0;
    }

    let mut best_move_evaluation = None;

    for m in moves.iter() {
        board.apply_move(m);
        let evaluation = minimax(board, depth - 1, neg * -1.0) * neg;
        board.revert_move(m);

        if best_move_evaluation == None || evaluation > best_move_evaluation.unwrap() {
            best_move_evaluation = Some(evaluation);
        }
    }

    return best_move_evaluation.unwrap() * neg;
}

fn generate_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();

    for (piece, square) in board.piece_list.iter() {
        if piece.color != board.side {
            continue;
        }



//        match piece.piece.kind {
//            PieceType::BlackPawn => {
//                moves.push(Move { from: piece.square64, to: move64(piece.square64, 0, -1), capture: None });
//                if rank64(piece.square64) == 6 {
//                    moves.push(Move { from: piece.square64, to: move64(piece.square64, 0, -2), capture: None });
//                }
//
//                // Generate capture moves
//                for delta_file in [-1 as i8, 1 as i8].iter() {
//                    let target_piece = board.piece_at(move64(piece.square64, *delta_file, -1));
//                    match target_piece {
//                        Some(target_piece) => {
//                            if target_piece.piece_type.color() == Color::White {
//                                moves.push(Move { from: piece.square64, to: move64(piece.square64, *delta_file, -1), capture: Some(target_piece.piece_type) });
//                            }
//                        }
//                        None => {}
//                    }
//                }
//            }
//            _ => {
//                println!("Unexpected piece {:?}", piece);
//            }
//        }
    }

    return moves;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pawn_moves() {
        let mut board = init_board();
        board.side = Color::Black;
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::Black).at(0, 6));
        let expected_moves = vec!(
            Move { from: Square::at(0, 6), to: Square::at(0, 5), capture: None },
            Move { from: Square::at(0, 6), to: Square::at(0, 4), capture: None },
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

//    #[test]
//    fn pawn_moves_blocked() {
//        let mut board = init_board();
//        board.side = Color::Black;
//        board.pieces.push(Piece { piece_type: PieceType::BlackPawn, square64: square64(0, 6) });
//        board.pieces.push(Piece { piece_type: PieceType::BlackPawn, square64: square64(0, 4) });
//        board.pieces.push(Piece { piece_type: PieceType::BlackPawn, square64: square64(1, 5) });
//        let expected_moves = vec!(
//            Move { from: square64(0, 6), to: square64(0, 5), capture: None }
//        );
//        assert_eq!(generate_moves(&board), expected_moves);
//    }
//
//    #[test]
//    fn pawn_moves_capture() {
//        let mut board = init_board();
//        board.side = Color::Black;
//        board.pieces.push(Piece { piece_type: PieceType::BlackPawn, square64: square64(1, 6) });
//        board.pieces.push(Piece { piece_type: PieceType::WhitePawn, square64: square64(0, 5) });
//        board.pieces.push(Piece { piece_type: PieceType::WhitePawn, square64: square64(2, 5) });
//        let expected_moves = vec!(
//            Move { from: square64(1, 6), to: square64(1, 5), capture: None },
//            Move { from: square64(1, 6), to: square64(1, 4), capture: None },
//            Move { from: square64(1, 6), to: square64(0, 5), capture: Some(PieceType::WhitePawn) },
//            Move { from: square64(1, 6), to: square64(2, 5), capture: Some(PieceType::WhitePawn) }
//        );
//        assert_eq!(generate_moves(&board), expected_moves);
//    }
}

fn main() {
    let v = vec!((1, 2), (5, 3));

    for (a, b) in v {
        println!("{:?}, {:?}", a, b);
    }


    println!("Hello, world!");
    let mut board = init_board();
    board.side = Color::Black;

    board.piece_list.push(Piece::create(PieceKind::Pawn, Color::Black).at(0, 6));
    board.piece_list.push(Piece::create(PieceKind::Pawn, Color::White).at(1, 5));
    println!("Board {:?}", board);

    let moves = generate_moves(&board);
    println!("Moves {:?}", moves);

    let s_evaluation = static_evaluation(&board);
    println!("Evaluation {:?}", s_evaluation);
    let d_evaluation = minimax(&mut board, 1, -1.0);
    println!("Evaluation {:?}", d_evaluation);
}
