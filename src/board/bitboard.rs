use super::piece::PieceColor;
use crate::movegen::constants::{A_FILE, H_FILE, RANK_2, RANK_7}; // Assuming these constants are correctly imported
use derive_more::{Add, AddAssign, BitAnd, BitOr, BitAndAssign, BitOrAssign, BitXor, BitXorAssign, Mul, Not, Sub, SubAssign, Shr};
#[derive(
    Copy,
    Clone,
    PartialEq,
    Add,
    Sub,
    BitAnd,
    BitXor,
    BitOr,
    BitOrAssign,
    AddAssign,
    SubAssign,
    BitXorAssign,
    BitAndAssign,
    Not,
    Shr,
    Mul,
    Default,
)]
pub struct Bitboard(u64);

impl PartialEq<u64> for Bitboard {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Bitboard> for u64 {
    fn eq(&self, other: &Bitboard) -> bool {
        *self == other.0
    }
}

impl Bitboard {
    /// Create a new `Bitboard` from a `u64` value.
    pub const fn new(bit: u64) -> Self {
        Bitboard(bit)
    }

    /// Check if a specific bit is set in the bitboard.
    pub fn is_set(self, bit: u64) -> bool {
        (self.0 & bit) != 0
    }

    /// Set a specific bit on the bitboard.
    pub fn set(&mut self, bit: u64) {
        self.0 |= bit;
    }

    pub fn set_square(&mut self, square: u8) {
        self.set(1 << square);
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
    pub fn create_from_square(square: u8) -> Bitboard {
        Bitboard(1 << square)
    }
    /// Perform a double pawn push, ensuring there are no blockers.
    pub fn pawn_double_push(self, color: &PieceColor, blockers: Bitboard) -> Bitboard {
        let rank = if *color == PieceColor::WHITE { RANK_2 } else { RANK_7 };

        self.bitand(rank).pawn_push(color)
            .bitand(!blockers)
            .pawn_push(color)
            .bitand(!blockers)
    }
    pub fn get_single_set_bit(self) -> u8 {
          self.0.trailing_zeros() as u8
    }
    pub fn pop_count(self) -> u8 {
        self.0.count_ones() as u8
    }


    /// Perform a pawn attack in the specified direction for the given color.
    pub fn pawn_attack(self, color: PieceColor, opponent: Bitboard, attack_left: bool) -> Bitboard {
        let attack;

        if color == PieceColor::WHITE {
            // White pawns attack diagonally up and to the left and right
            if attack_left {
                attack = Bitboard(self.0 << 9).bitand(!A_FILE); // Mask out the a-file to prevent wraparound
            } else {
                attack = Bitboard(self.0 >> 7).bitand(!H_FILE); // Mask out the h-file to prevent wraparound
            }
        } else {
            // Black pawns attack diagonally down and to the left and right
            if attack_left {
                attack = Bitboard(self.0 >> 7).bitand(!A_FILE); // Mask out the a-file to prevent wraparound
            } else {
                attack = Bitboard(self.0 << 9).bitand(!H_FILE); // Mask out the h-file to prevent wraparound
            }
        }

        // Only include squares that are occupied by the opponent
        attack.bitand(opponent)
    }

    pub fn pop_lsb(&mut self) -> u8 {
        let lsb = self.0 & (!self.0 + 1);
        self.0 ^= lsb;
        lsb.trailing_zeros() as u8
    }
    pub fn bitboard_to_set_vec(&self) -> Vec<u8> {
        let mut set_vec = Vec::new();
        let bitboard = self.0;
        for i in 0..64 {
            if bitboard >> i & 1 == 1 {
                set_vec.push(i);
            }
        }
        set_vec
    }
    pub fn contains_square(&self, square: u8) -> bool {
        self.0 & (1 << square) != 0
    }

    pub fn get_bitboard(self) -> u64 {
        self.0
    }
}

use std::fmt;


impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}