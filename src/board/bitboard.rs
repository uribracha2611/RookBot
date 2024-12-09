use std::ops::Not;
use super::piece::PieceColor;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitboard(u64);

impl Not for Bitboard {
    type Output = Bitboard;

    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

impl Bitboard {
    /// Create a new `Bitboard` from a `u64` value.
    pub fn new(bit: u64) -> Self {
        Bitboard(bit)
    }

    /// Check if a specific bit is set in the bitboard.
    pub fn is_set(self, bit: u64) -> bool {
        (self.0 & bit) != 0
    }

    /// Return the intersection (AND) of two bitboards.
    pub fn intersection(self, other: Bitboard) -> Bitboard {
        Bitboard(self.0 & other.0)
    }

    /// Return the union (OR) of two bitboards.
    pub fn union(self, other: Bitboard) -> Bitboard {
        Bitboard(self.0 | other.0)
    }

    /// Set a specific bit on the bitboard.
    pub fn set(&mut self, bit: u64) {
        self.0 |= bit;
    }

    /// Clear a specific bit on the bitboard.
    pub fn clear(&mut self, bit: u64) {
        self.0 &= !bit;
    }

    /// Perform a pawn push in the specified direction for the given color.
    pub fn pawn_push(self, color: &PieceColor) -> Bitboard {
        if *color == PieceColor::WHITE {
            Bitboard(self.0 << 8)
        } else {
            Bitboard(self.0 >> 8)
        }
    }

    /// Perform a double pawn push, ensuring there are no blockers.
    pub fn pawn_double_push(self, color: &PieceColor, blockers: Bitboard) -> Bitboard {
        self.pawn_push(color)
            .intersection(!blockers)
            .pawn_push(color)
            .intersection(!blockers)
    }
}
