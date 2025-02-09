use crate::board::bitboard::Bitboard;
use crate::board::position::Position;

pub(crate) const A_FILE: Bitboard = Bitboard::new(0x0101010101010101); // Mask for the a-file (bits 0, 8, 16, ..., 56)
pub(crate) const H_FILE: Bitboard = Bitboard::new(0x8080808080808080); // Mask for the h-file (bits 7, 15, 23, ..., 63)
pub(crate) const MAX_MOVES: usize = 218;
pub const KNIGHT_OFFSETS: [Position; 8] = [
    Position { x: 2, y: 1 },
    Position { x: 2, y: -1 },
    Position { x: -2, y: 1 },
    Position { x: -2, y: -1 },
    Position { x: 1, y: 2 },
    Position { x: 1, y: -2 },
    Position { x: -1, y: 2 },
    Position { x: -1, y: -2 },
];
pub const KING_OFFSETS: [Position; 8] = [
    Position { x: 1, y: 0 },  // Right
    Position { x: -1, y: 0 }, // Left
    Position { x: 0, y: 1 },  // Up
    Position { x: 0, y: -1 }, // Down
    Position { x: 1, y: 1 },  // Up-Right
    Position { x: 1, y: -1 }, // Down-Right
    Position { x: -1, y: 1 }, // Up-Left
    Position { x: -1, y: -1 }, // Down-Left
];
pub const ROOK_OFFSETS: [Position; 4] = [
    Position { x: 1, y: 0 },  // Right
    Position { x: -1, y: 0 }, // Left
    Position { x: 0, y: 1 },  // Up
    Position { x: 0, y: -1 }, // Down
];
pub const BISHOP_OFFSETS: [Position; 4] = [
    Position { x: 1, y: 1 },  // Up-Right
    Position { x: 1, y: -1 }, // Down-Right
    Position { x: -1, y: 1 }, // Up-Left
    Position { x: -1, y: -1 }, // Down-Left
];
pub const ALL_OFSET: [Position; 8] = [
    Position { x: 1, y: 0 },  // Right
    Position { x: -1, y: 0 }, // Left
    Position { x: 0, y: 1 },  // Up
    Position { x: 0, y: -1 }, // Down
    Position { x: 1, y: 1 },  // Up-Right
    Position { x: 1, y: -1 }, // Down-Right
    Position { x: -1, y: 1 }, // Up-Left
    Position { x: -1, y: -1 }, // Down-Left
];

pub(crate) const RANK_1: Bitboard = Bitboard::new(0x00000000000000FF);
pub(crate) const RANK_2: Bitboard = Bitboard::new(0x000000000000FF00);
pub(crate) const RANK_3: Bitboard = Bitboard::new(0x0000000000FF0000);
pub(crate) const RANK_4: Bitboard = Bitboard::new(0x00000000FF000000);
pub(crate) const RANK_5: Bitboard = Bitboard::new(0x000000FF00000000);
pub(crate) const RANK_6: Bitboard = Bitboard::new(0x0000FF0000000000);
pub(crate) const RANK_7: Bitboard = Bitboard::new(0x00FF000000000000);
pub(crate) const RANK_8: Bitboard = Bitboard::new(0xFF00000000000000);