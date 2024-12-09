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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub piece_color: PieceColor,
    pub piece_type: PieceType,
}
impl Piece 
{
    pub fn new (piece_color:PieceColor,piece_type:PieceType) -> Self {
        return Piece{piece_color,piece_type}
    }
    pub fn is_color(&self,piece_color:PieceColor) -> bool {
        return self.piece_color == piece_color
    }
    pub fn is_type(&self,piece_type:PieceType) -> bool {
        return self.piece_type == piece_type
    }
    pub fn is_diag(&self) -> bool {
        return self.piece_type == PieceType::BISHOP || self.piece_type == PieceType::QUEEN
    
}
pub fn is_ortho(&self) -> bool {
    return self.piece_type == PieceType::ROOK || self.piece_type == PieceType::QUEEN
}
}