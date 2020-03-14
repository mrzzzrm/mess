#[derive(Clone, Copy, Ord, Eq, PartialOrd, PartialEq)]
pub struct Square {
    x: i8,
    y: i8,
}

impl Square {
    pub fn at(x: i8, y: i8) -> Square {
        Square { x, y }
    }

    pub fn file(&self) -> i8 {
        self.x
    }

    pub fn rank(&self) -> i8 {
        self.y
    }

    pub fn is_on_board(&self) -> bool {
        return self.x >= 0 && self.x < 8 && self.y >= 0 && self.y < 8;
    }

    pub fn delta(&self, x: i8, y: i8) -> Square {
        Square { x: self.x + x, y: self.y + y }
    }

    pub fn algebraic(&self) -> String {
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
pub enum Castle {
    KingSide,
    QueenSide,
}

#[derive(Clone, Copy, Debug, Ord, Eq, PartialOrd, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn switch(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White
        }
    }
    pub fn index(self) -> usize {
        match self {
            Color::White => 0,
            Color::Black => 1
        }
    }
    pub fn token(&self) -> char {
        match self {
            Color::White => 'B',
            Color::Black => 'W'
        }
    }
    pub fn evaluation_sign(&self) -> f32 {
        match self {
            Color::White => 1.0_f32,
            Color::Black => -1.0_f32
        }
    }
    pub fn promotion_rank(&self) -> u8 {
        match self {
            Color::White => 7,
            Color::Black => 0
        }
    }
    pub fn back_rank(&self) -> i8 {
        match self {
            Color::White => 0,
            Color::Black => 7
        }
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, Eq, PartialEq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    Dummy,
}

impl PieceKind {
    pub fn value(&self) -> f32 {
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

    pub fn token(&self) -> char {
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

    pub fn colored(self, color: Color) -> Piece {
        Piece::create(self, color)
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

impl Piece {
    pub fn create(kind: PieceKind, color: Color) -> Piece {
        return Piece { kind, color };
    }

    pub fn at(&self, file: i8, rank: i8) -> PieceOnBoard {
        return (*self, Square { x: file, y: rank });
    }

    pub fn value(&self) -> f32 {
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

// Castle rights of one side
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorCastleRights {
    pub king_side: bool,
    pub queen_side: bool,
}

impl ColorCastleRights {
    pub fn all() -> ColorCastleRights {
        ColorCastleRights { king_side: true, queen_side: true }
    }

    pub fn none() -> ColorCastleRights {
        ColorCastleRights { king_side: false, queen_side: false }
    }

    pub fn test(&self, side: Castle) -> bool {
        match side {
            Castle::KingSide => self.king_side,
            Castle::QueenSide => self.queen_side,
        }
    }
}

// Castle rights on the Board
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoardCastleRights {
    pub white: ColorCastleRights,
    pub black: ColorCastleRights,
}

impl BoardCastleRights {
    pub fn all() -> BoardCastleRights {
        BoardCastleRights {
            white: ColorCastleRights::all(),
            black: ColorCastleRights::all(),
        }
    }

    pub fn none() -> BoardCastleRights {
        BoardCastleRights {
            white: ColorCastleRights::none(),
            black: ColorCastleRights::none(),
        }
    }

    pub fn get_rights(&self, color: Color) -> ColorCastleRights {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }

    pub fn get_rights_mut(&mut self, color: Color) -> &mut ColorCastleRights {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }

    pub fn set_rights(&mut self, color: Color, rights: &ColorCastleRights) {
        match color {
            Color::White => self.white = *rights,
            Color::Black => self.black = *rights,
        }
    }
}

pub type PieceOnBoard = (Piece, Square);