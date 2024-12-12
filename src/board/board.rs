use super::{bitboard::Bitboard, piece::{Piece, PieceColor}};

pub struct  Board{
     squares:[Option<Piece>;64],
    turn:PieceColor,
   pub  color_bitboards:[Bitboard;2],
   pub  piece_bitboards:[[Bitboard;6];2],
   pub all_pieces_bitboard:Bitboard,

    

    
    
}