use crate::board::bitboard::Bitboard;

pub(crate) const A_FILE: Bitboard = Bitboard::new(0x0101010101010101);  // Mask for the a-file (bits 0, 8, 16, ..., 56)
pub(crate) const H_FILE: Bitboard = Bitboard::new(0x8080808080808080);  // Mask for the h-file (bits 7, 15, 23, ..., 63)
pub(crate) const MAX_MOVES: usize = 218;