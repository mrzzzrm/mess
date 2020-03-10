extern crate num_traits;

use num_traits::Float;
use std::time::{Duration, Instant};
use std::string::String;

#[derive(Clone, Copy, Ord, Eq, PartialOrd, PartialEq)]
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

    fn algebraic(&self) -> String {
        assert!(self.is_on_board());
        format!("{}{}", ('a' as u8 + self.x as u8) as char, self.y)
    }
}

impl std::fmt::Debug for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.file(), self.rank())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Castle {
    KingSide,
    QueenSide,
}

#[derive(Clone, Copy, Debug, Ord, Eq, PartialOrd, PartialEq)]
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
    fn back_rank(&self) -> i8 {
        match self {
            Color::White => 0,
            Color::Black => 7
        }
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, Eq, PartialEq)]
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

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
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

impl std::fmt::Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut token = self.kind.token();

        if self.color == Color::White {
            token = token.to_uppercase().to_string().chars().nth(0).unwrap();
        }

        write!(f, "{}", token.to_string())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Move {
    from: Square,
    to: Square,
    piece_kind: PieceKind,
    capture: Option<PieceOnBoard>,
    en_passant_before: Option<Square>,
    en_passant_after: Option<Square>,

    castle_rights_before: BoardCastleRights,

    castle: Option<Castle>,
    promotion: Option<PieceKind>,
}

impl Move {
    fn castle(board: &Board, color: Color, castle: Castle) -> Move {
        let file = match castle {
            Castle::KingSide => 6,
            Castle::QueenSide => 2,
        };

        let rank = color.back_rank();

        let mut m = Move::from_to(board, PieceKind::King, Square::at(4, rank), Square::at(file, rank));
        m.castle = Some(castle);

        return m;
    }

    fn promotion(board: &Board, from: Square, to: Square, promotion: PieceKind) -> Move {
        let mut m = Move::from_to(board, PieceKind::Pawn, from, to);
        m.promotion = Some(promotion);
        return m;
    }

    fn promotion_capture(board: &Board, from: Square, to: Square, capture: PieceOnBoard, promotion: PieceKind) -> Move {
        let mut m = Move::from_to(board, PieceKind::Pawn, from, to);
        m.capture = Some(capture);
        m.promotion = Some(promotion);
        return m;
    }

    fn from_to(board: &Board, piece_kind: PieceKind, from: Square, to: Square) -> Move {
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
    fn from_to_en_passant(board: &Board, from: Square, to: Square, en_passant: Square) -> Move {
        let mut m = Move::from_to(board, PieceKind::Pawn, from, to);
        m.en_passant_after = Some(en_passant);
        return m;
    }

    fn from_to_capture(board: &Board, piece_kind: PieceKind, from: Square, to: Square, capture: PieceOnBoard) -> Move {
        let mut m = Move::from_to(board, piece_kind, from, to);
        m.capture = Some(capture);
        return m;
    }

    fn castle_rights_after(&self, side: Color) -> BoardCastleRights {
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
    fn rook_castle(board: &Board, castle: Castle, rank: i8) -> Move {
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

    fn long_algebraic(&self) -> String {
        format!("{}{}{}", self.from.algebraic(), "-", self.to.algebraic())
    }
}

struct Line {
    moves: Vec<Move>
}

impl Line {
    fn new() -> Line {
        Line{moves: Vec::new()}
    }

    fn from_moves(moves: Vec<Move>) -> Line {
        Line{moves: moves}
    }

    fn to_string(&self) -> String {
        self.moves.iter().map(|m| m.long_algebraic()).collect::<Vec<String>>().join(" ")
    }
}

type PieceOnBoard = (Piece, Square);

// Castle rights of one side
#[derive(Clone, Copy, Debug, PartialEq)]
struct ColorCastleRights {
    king_side: bool,
    queen_side: bool,
}

impl ColorCastleRights {
    fn all() -> ColorCastleRights {
        ColorCastleRights { king_side: true, queen_side: true }
    }

    fn none() -> ColorCastleRights {
        ColorCastleRights { king_side: false, queen_side: false }
    }

    fn test(&self, side: Castle) -> bool {
        match side {
            Castle::KingSide => self.king_side,
            Castle::QueenSide => self.queen_side,
        }
    }
}

// Castle rights on the Board
#[derive(Clone, Copy, Debug, PartialEq)]
struct BoardCastleRights {
    white: ColorCastleRights,
    black: ColorCastleRights,
}

impl BoardCastleRights {
    fn all() -> BoardCastleRights {
        BoardCastleRights {
            white: ColorCastleRights::all(),
            black: ColorCastleRights::all(),
        }
    }

    fn none() -> BoardCastleRights {
        BoardCastleRights {
            white: ColorCastleRights::none(),
            black: ColorCastleRights::none(),
        }
    }

    fn get_rights(&self, color: Color) -> ColorCastleRights {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }

    fn get_rights_mut(&mut self, color: Color) -> &mut ColorCastleRights {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }

    fn set_rights(&mut self, color: Color, rights: &ColorCastleRights) {
        match color {
            Color::White => self.white = *rights,
            Color::Black => self.black = *rights,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Board {
    piece_list: Vec<PieceOnBoard>,
    side: Color,
    en_passant: Option<Square>,
    castle_rights: BoardCastleRights,
}

impl Board {
    fn create_empty() -> Board {
        Board {
            piece_list: Vec::new(),
            side: Color::White,
            en_passant: None,
            castle_rights: BoardCastleRights::none(),
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

        board.castle_rights = BoardCastleRights::all();

        return board;
    }

    fn create_king_rooks() -> Board {
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

    fn create_rooks() -> Board {
        let mut board = Board::create_empty();
        board.piece_list.push(PieceKind::Rook.colored(Color::White).at(7, 0));
        board.piece_list.push(PieceKind::Rook.colored(Color::Black).at(7, 7));

        board.castle_rights = BoardCastleRights::none();

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

    fn piece_at_mut(&mut self, square: Square) -> &mut PieceOnBoard {
        for piece_on_board in self.piece_list.iter_mut() {
            if square == piece_on_board.1 {
                return piece_on_board;
            }
        }
        panic!("No piece found on {:?}", square);
    }

    fn has_piece_at(&self, square: Square) -> bool {
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

    fn apply_move(&mut self, m: Move) {
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

    fn revert_move(&mut self, m: Move) {
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

    fn semantic_eq(&self, other: &Self) -> bool {
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
    fn get_best_line(&self) -> &Line;
    fn get_statistics(&self) -> DynamicEvaluatorStatistics;
}

struct MinimaxEvaluator {
    statistics: DynamicEvaluatorStatistics,
    best_line: Line
}

impl MinimaxEvaluator {
    fn create() -> MinimaxEvaluator {
        MinimaxEvaluator { statistics: DynamicEvaluatorStatistics::create(), best_line: Line::new() }
    }

    fn minimax(&mut self, board: &mut Board, depth: u32, max_depth: u32, neg: f32) -> f32 {
        self.statistics.node_count += 1;

        if depth == max_depth {
            return static_evaluation(&board);
        }

        let moves = generate_moves(&board);
        if moves.is_empty() {
            return static_evaluation(&board);
        }

        let mut best_move_evaluation = None;

        for m in moves.iter() {
            let mut move_unmove = MoveUnmove::apply_move(board, m);
            let evaluation = self.minimax(board, depth + 1, max_depth, neg * -1.0) * neg;
            move_unmove.revert_move(board);

            if best_move_evaluation.is_none() || evaluation > best_move_evaluation.unwrap() {
                best_move_evaluation = Some(evaluation);
            }
        }

        return best_move_evaluation.unwrap() * neg;
    }
}

impl DynamicEvaluator for MinimaxEvaluator {
    fn evaluate(&mut self, board: &mut Board, depth: u32) -> f32 {
        self.best_line.moves.clear();

        let neg = match board.side {
            Color::White => 1.0,
            Color::Black => -1.0
        };

        let stopwatch = std::time::Instant::now();
        let evaluation = self.minimax(board, 0, depth, neg);
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

struct AlphaBetaEvaluator {
    statistics: DynamicEvaluatorStatistics,
    best_line: Line
}

impl AlphaBetaEvaluator {
    fn create() -> AlphaBetaEvaluator {
        AlphaBetaEvaluator { statistics: DynamicEvaluatorStatistics::create(), best_line: Line::new() }
    }

    fn alpha_beta_min(&mut self, board: &mut Board, alpha: f32, mut beta: f32, depth: u32) -> f32 {
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
            let mut move_unmove = MoveUnmove::apply_move(board, m);
            let evaluation = self.alpha_beta_max(board, alpha, beta, depth - 1);
            move_unmove.revert_move(board);

            if evaluation <= alpha {
                return evaluation;
            }

            if evaluation < beta {
                beta = evaluation;
            }

            if best_move_evaluation == None || evaluation > best_move_evaluation.unwrap() {
                best_move_evaluation = Some(evaluation);
            }
        }

        return best_move_evaluation.unwrap();
    }

    fn alpha_beta_max(&mut self, board: &mut Board, mut alpha: f32, beta: f32, depth: u32) -> f32 {
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
            let mut move_unmove = MoveUnmove::apply_move(board, m);
            let evaluation = self.alpha_beta_min(board, alpha, beta, depth - 1);
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
    fn evaluate(&mut self, board: &mut Board, depth: u32) -> f32 {
        self.best_line.moves.clear();

        let stopwatch = std::time::Instant::now();
        let evaluation = match board.side {
            Color::White => self.alpha_beta_max(board, num_traits::float::Float::min_value(), num_traits::float::Float::max_value(), depth),
            Color::Black => self.alpha_beta_min(board, num_traits::float::Float::min_value(), num_traits::float::Float::max_value(), depth)
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

struct MoveUnmove {
    board_before: Board,
    move_: Move,
}

impl MoveUnmove {
    fn apply_move(board: &mut Board, move_: &Move) -> MoveUnmove {
        let move_unmove = MoveUnmove {
            board_before: board.clone(),
            move_: *move_,
        };
        board.apply_move(*move_);
        return move_unmove;
    }

    fn revert_move(&mut self, board: &mut Board) {
        board.revert_move(self.move_);

        // if !board.semantic_eq(&self.board_before) {
        //     panic!("Board mismatch after {:?}\n{:?}\nvs\n{:?}", self.move_, self.board_before, board);
        // }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    struct TestMove {}

    impl TestMove {
        // Wrapper around Move::from_to() that performs a lookup of the PieceKind
        fn from_to(board: &Board, from: Square, to: Square) -> Move {
            Move::from_to(board, board.piece_at(from).unwrap().kind, from, to)
        }

        fn castle(board: &Board, color: Color, castle: Castle) -> Move {
            Move::castle(board, color, castle)
        }

        fn promotion(board: &Board, from: Square, to: Square, promotion: PieceKind) -> Move {
            Move::promotion(board, from, to, promotion)
        }

        fn promotion_capture(board: &Board, from: Square, to: Square, capture: PieceOnBoard, promotion: PieceKind) -> Move {
            Move::promotion_capture(board, from, to, capture, promotion)
        }

        fn from_to_en_passant(board: &Board, from: Square, to: Square, en_passant: Square) -> Move {
            Move::from_to_en_passant(board, from, to, en_passant)
        }

        // Wrapper around Move::from_to_capture() that performs a lookup of the PieceKind
        fn from_to_capture(board: &Board, from: Square, to: Square, capture: PieceOnBoard) -> Move {
            Move::from_to_capture(board, board.piece_at(from).unwrap().kind, from, to, capture)
        }
    }

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

    #[test]
    fn minimax_basic() {
        let minimax = |board: &mut Board, depth: u32, neg: f32| {
            MinimaxEvaluator::create().minimax(board, 0, depth, neg)
        };

        // Just a white pawn
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 1));
        assert_eq!(minimax(&mut board, 3, 1.0), 1.0);

        // Just a black pawn
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::Black).at(0, 6));
        assert_eq!(minimax(&mut board, 3, -1.0), -1.0);

        // A white pawn that can capture a black pawn
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 1),
            PieceKind::Pawn.colored(Color::Black).at(1, 2));
        assert_eq!(minimax(&mut board, 3, 1.0), 1.0);

        // A black pawn that can capture a white pawn
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 2),
            PieceKind::Pawn.colored(Color::Black).at(1, 3));
        assert_eq!(minimax(&mut board, 3, -1.0), -1.0);

        // A white pawn that can capture a black pawn and another black pawn
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 1),
            PieceKind::Pawn.colored(Color::Black).at(1, 2),
            PieceKind::Pawn.colored(Color::Black).at(3, 2));
        assert_eq!(minimax(&mut board, 3, 1.0), 0.0);

        // A white pawn that will be capture by a black pawn after it moves
        let mut board = Board::create_empty();
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 4),
            PieceKind::Pawn.colored(Color::Black).at(1, 6));
        assert_eq!(minimax(&mut board, 3, 1.0), -1.0);

        // A white pawn that will capture a black pawn after the black pawn moves
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 3),
            PieceKind::Pawn.colored(Color::Black).at(1, 5));
        assert_eq!(minimax(&mut board, 3, -1.0), 1.0);

        // A white pawn that will be captured by a black pawn after a couple of moves
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 2),
            PieceKind::Pawn.colored(Color::Black).at(1, 5), );
        assert_eq!(minimax(&mut board, 10, -1.0), -1.0);

        // ...
        let mut board = Board::create_empty();
        board.side = Color::Black;
        board.piece_list = vec!(
            PieceKind::Pawn.colored(Color::White).at(0, 3),
            PieceKind::Pawn.colored(Color::White).at(1, 5),
            PieceKind::Pawn.colored(Color::Black).at(0, 6), );
        assert_eq!(minimax(&mut board, 10, -1.0), -1.0);
    }

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

fn play(board: &mut Board) {
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

        let nodes_per_second = evaluator.statistics.node_count as f32 / evaluator.statistics.duration.as_secs_f32();
        let best_move = best_move.unwrap();

        println!("Chose move {:?} with an evaluation of {}, evaluated {} nodes at {} nodes/s", best_move, best_move_evaluation * neg, evaluator.statistics.node_count, nodes_per_second);
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

fn main() {
    let mut board = Board::create_king_rooks();
    play(&mut board);
}
