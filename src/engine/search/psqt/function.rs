use crate::engine::board::piece::{Piece, PieceColor, PieceType};
use crate::engine::search::psqt::constants::{
    BISHOP_TABLE, KING_TABLE, KNIGHT_TABLE, PAWN_TABLE, QUEEN_TABLE, ROOK_TABLE,
};
use crate::engine::search::psqt::weight::W;

pub fn flip_sqr(sqr: usize) -> usize {
    sqr ^ 56
}
pub fn get_psqt(sqr: usize, piece: Piece) -> W {
    let index = if piece.piece_color == PieceColor::BLACK {
        sqr
    } else {
        flip_sqr(sqr)
    };
    match piece.piece_type {
        PieceType::PAWN => PAWN_TABLE[index],
        PieceType::KNIGHT => KNIGHT_TABLE[index],
        PieceType::BISHOP => BISHOP_TABLE[index],
        PieceType::ROOK => ROOK_TABLE[index],
        PieceType::QUEEN => QUEEN_TABLE[index],
        PieceType::KING => KING_TABLE[index],
    }
}
