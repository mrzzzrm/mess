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
    fn token(&self) -> char {
        match self {
            Color::Black => 'W',
            Color::White => 'B'
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

impl Move {
    fn from_to(from: Square, to: Square) -> Move {
        Move {from, to, capture: None}
    }

    fn from_to_capture(from: Square, to: Square, capture: PieceOnBoard) -> Move {
        Move {from, to, capture: Some(capture)}
    }
}

type PieceOnBoard = (Piece, Square);

#[derive(Debug)]
struct Board {
    piece_list: Vec<PieceOnBoard>,
    side: Color,
    en_passant_square: Option<Square>
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

    fn has_piece_at(&self, square: Square) -> bool {
        self.piece_list.iter().position(|(_, square2)| square == *square2).is_some()
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

    fn print(&self) {
        print!("  ",);
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

fn init_board() -> Board {
    Board {
        piece_list: Vec::new(),
        side: Color::White,
        en_passant_square: None
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
                    moves.push(Move::from_to(*square, square.delta(0, forward)));

                    if square.rank() == home_rank && !board.has_piece_at(square.delta(0, forward * 2)) && square.delta(0, forward * 2).is_on_board() {
                        moves.push(Move::from_to(*square, square.delta(0, forward * 2)));
                    }
                }

                // Generate capture moves
                for file_delta in [-1 as i8, 1 as i8].iter() {
                    let target_piece = board.piece_at(square.delta(*file_delta, forward));

                    if target_piece.is_some() {
                        let target_piece = target_piece.unwrap();
                        if target_piece.color != piece.color {
                            moves.push(Move::from_to_capture(*square, square.delta(*file_delta, forward), (target_piece, square.delta(*file_delta, forward))));
                        }
                    }

                    if board.en_passant_square.is_some() && board.en_passant_square.unwrap() == square.delta(*file_delta, forward) {
                        let en_passant_piece = board.piece_at(square.delta(*file_delta, 0)).unwrap();
                        moves.push(Move::from_to_capture(*square, square.delta(*file_delta, forward), (en_passant_piece, square.delta(*file_delta, 0))));
                    }
                }

            }
            _ => {
              println!("Unexpected piece {:?} at {:?}", piece, square);
            }
        }
    }

    return moves;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pawn_moves() {
        let mut board = init_board();
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::Black).at(0, 6));
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::White).at(2, 1));
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::White).at(3, 2));

        let expected_moves = vec!(
            Move::from_to(Square::at(2, 1), Square::at(2, 2)),
            Move::from_to(Square::at(2, 1), Square::at(2, 3)),
            Move::from_to(Square::at(3, 2), Square::at(3, 3)),
        );
        assert_eq!(generate_moves(&board), expected_moves);

        board.side = Color::Black;
        let expected_moves = vec!(
            Move::from_to(Square::at(0, 6), Square::at(0, 5)),
            Move::from_to(Square::at(0, 6), Square::at(0, 4)),
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn pawn_moves_blocked() {
        let mut board = init_board();
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::Black).at(0, 6));
        board.piece_list.push(Piece::create(PieceKind::Dummy, Color::White).at(0, 4));
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::Black).at(5, 3));
        board.piece_list.push(Piece::create(PieceKind::Dummy, Color::White).at(5, 2));
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::White).at(2, 1));
        board.piece_list.push(Piece::create(PieceKind::Dummy, Color::White).at(2, 2));
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::White).at(3, 1));
        board.piece_list.push(Piece::create(PieceKind::Dummy, Color::White).at(3, 3));

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
        let mut board = init_board();
        board.side = Color::Black;
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::Black).at(0, 6));
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::White).at(0, 5));
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::White).at(1, 5));
        let expected_moves = vec!(
            Move::from_to_capture(Square::at(0, 6), Square::at(1, 5), Piece::create(PieceKind::Pawn, Color::White).at(1, 5)),
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }

    #[test]
    fn pawn_moves_en_passant() {
        let mut board = init_board();
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::White).at(1, 4));
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::Black).at(2, 4));
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::Black).at(4, 3));
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::White).at(5, 3));
        board.piece_list.push(Piece::create(PieceKind::Pawn, Color::Black).at(7, 3));

        board.en_passant_square = Some(Square::at(2, 5));
        let expected_moves = vec!(
            Move::from_to(Square::at(1, 4), Square::at(1, 5)),
            Move::from_to_capture(Square::at(1, 4), Square::at(2, 5), Piece::create(PieceKind::Pawn, Color::Black).at(2, 4)),
            Move::from_to(Square::at(5, 3), Square::at(5, 4))
        );
        assert_eq!(generate_moves(&board), expected_moves);

        board.side = Color::Black;
        board.en_passant_square = Some(Square::at(5, 2));
        let expected_moves = vec!(
            Move::from_to(Square::at(2, 4), Square::at(2, 3)),
            Move::from_to(Square::at(4, 3), Square::at(4, 2)),
            Move::from_to_capture(Square::at(4, 3), Square::at(5, 2), Piece::create(PieceKind::Pawn, Color::White).at(5, 3)),
            Move::from_to(Square::at(7, 3), Square::at(7, 2)),
        );
        assert_eq!(generate_moves(&board), expected_moves);
    }
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

    board.print();

    let moves = generate_moves(&board);
    println!("Moves {:?}", moves);

    let s_evaluation = static_evaluation(&board);
    println!("Evaluation {:?}", s_evaluation);
    let d_evaluation = minimax(&mut board, 1, -1.0);
    println!("Evaluation {:?}", d_evaluation);
}
