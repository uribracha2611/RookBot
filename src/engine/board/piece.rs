use std::fmt;
use std::ops::Index;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    PAWN,
    KNIGHT,
    BISHOP,
    ROOK,
    QUEEN,
    KING,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PieceColor {
    WHITE,
    BLACK,
}
impl PieceColor {
    pub fn to_index(&self) -> usize {
        match self {
            PieceColor::WHITE => 0,
            PieceColor::BLACK => 1,
        }
    }
    
}
impl PieceType {
    pub fn to_index(&self) -> usize {
        match self {
            PieceType::PAWN => 0,
            PieceType::KNIGHT => 1,
            PieceType::BISHOP => 2,
            PieceType::ROOK => 3,
            PieceType::QUEEN => 4,
            PieceType::KING => 5,
        }
    }
    
}
impl<T> Index<PieceColor> for [T] {
    type Output = T;

    fn index(&self, index: PieceColor) -> &Self::Output {
        &self[index as usize]
    }
}


impl PieceColor {
    pub fn opposite(&self) -> PieceColor {
        match self {
            PieceColor::WHITE => PieceColor::BLACK,
            PieceColor::BLACK => PieceColor::WHITE,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub piece_color: PieceColor,
    pub piece_type: PieceType,
}

impl Piece {
    pub fn new(piece_color: PieceColor, piece_type: PieceType) -> Self {
        Piece {
            piece_color,
            piece_type,
        }
    }
    pub fn get_value(&self) -> i32 {
        match self.piece_type {
            PieceType::PAWN => 1,
            PieceType::KNIGHT => 3,
            PieceType::BISHOP => 3,
            PieceType::ROOK => 5,
            PieceType::QUEEN => 9,
            PieceType::KING => 0,
        }
    }

    pub fn is_color(&self, piece_color: PieceColor) -> bool {
        self.piece_color == piece_color
    }

    pub fn is_type(&self, piece_type: PieceType) -> bool {
        self.piece_type == piece_type
    }

    pub fn is_diag(&self) -> bool {
        matches!(self.piece_type, PieceType::BISHOP | PieceType::QUEEN)
    }

    pub fn is_ortho(&self) -> bool {
        matches!(self.piece_type, PieceType::ROOK | PieceType::QUEEN)
    }

    pub fn from_fen(fen: &str) -> Option<Self> {
        let piece_color = if fen.chars().next()?.is_uppercase() {
            PieceColor::WHITE
        } else {
            PieceColor::BLACK
        };

        let piece_type = match fen.to_ascii_uppercase().as_str() {
            "P" => PieceType::PAWN,
            "N" => PieceType::KNIGHT,
            "B" => PieceType::BISHOP,
            "R" => PieceType::ROOK,
            "Q" => PieceType::QUEEN,
            "K" => PieceType::KING,
            _ => return None,
        };

        Some(Piece::new(piece_color, piece_type))
    }

    pub fn to_fen(&self) -> String {
        let piece_str = self.piece_type.to_string();
        if self.piece_color == PieceColor::BLACK {
            piece_str.to_lowercase()
        } else {
            piece_str
        }
    }
    pub fn to_history_index(&self) -> usize {
     self.piece_color.to_index()*6+self.piece_type.to_index()
    }
    pub const  fn mvv_score(&self)->i32{
        match self.piece_type {
            PieceType::PAWN => 10,
            PieceType::KNIGHT => 11,
            PieceType::BISHOP => 12,
            PieceType::ROOK => 13,
            PieceType::QUEEN => 14,
            PieceType::KING => 15,
        }
    }
}


impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.piece_color.to_string(),
            self.piece_type.to_string()
        )
    }
}

impl fmt::Debug for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for PieceColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PieceType::PAWN => "P",
            PieceType::KNIGHT => "N",
            PieceType::BISHOP => "B",
            PieceType::ROOK => "R",
            PieceType::QUEEN => "Q",
            PieceType::KING => "K",
        };
        write!(f, "{}", s)
    }
}
impl PieceType {
    pub fn to_char(&self) -> char {
        match self {
            PieceType::KING => 'K',
            PieceType::QUEEN => 'Q',
            PieceType::ROOK => 'R',
            PieceType::BISHOP => 'B',
            PieceType::KNIGHT => 'N',
            PieceType::PAWN => 'P',
        }
    }
}

impl fmt::Display for PieceColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PieceColor::WHITE => "w",
            PieceColor::BLACK => "b",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let piece_str = self.piece_type.to_string();
        let s = if self.piece_color == PieceColor::BLACK {
            piece_str.to_lowercase()
        } else {
            piece_str
        };
        write!(f, "{}", s)
    }
}
