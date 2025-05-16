use super::piece::PieceColor;


#[derive(
    Copy,
    Clone,
    PartialEq,
    
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
#[derive(Debug)]
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

impl Shl<i32> for Bitboard {
    type Output = Bitboard;

    fn shl(self, rhs: i32) -> Self::Output {
        Bitboard(self.0 << rhs)
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
        let bit:u64 = 1u64 << square;
        self.set(bit);
    }

    /// Clear a specific bit on the bitboard.
    pub fn clear(&mut self, bit: u64) {
        self.0 &= !bit;
    }
    pub fn clear_square(&mut self, square: u8) {
        let bit:u64 = 1u64 << square;
        self.clear(bit);
    }

    /// Perform a pawn push in the specified direction for the given color.
    #[inline(always)]
    pub fn pawn_push(self, color: &PieceColor) -> Bitboard {
        if *color == PieceColor::WHITE {
            Bitboard(self.0 << 8)
        } else {
            Bitboard(self.0 >> 8)
        }
    }
    #[inline(always)]
    pub fn create_from_square(square: u8) -> Bitboard {
        let bit:u64=1u64<<square ;
        Bitboard(bit)
    }
    /// Perform a double pawn push, ensuring there are no blockers.
    #[inline(always)]
    pub fn pawn_double_push(self, color: &PieceColor, blockers: Bitboard) -> Bitboard {
        let rank = if *color == PieceColor::WHITE { RANK_2 } else { RANK_7 };

        self.bitand(rank).pawn_push(color)
            .bitand(!blockers)
            .pawn_push(color)
            .bitand(!blockers)
    }
    #[inline(always)]
    pub fn get_single_set_bit(self) -> u8 {
          self.0.trailing_zeros() as u8
    }
    #[inline(always)]
    pub fn pop_count(self) -> u8 {
        self.0.count_ones() as u8
    }


    /// Perform a pawn attack in the specified direction for the given color.
    #[inline(always)]
    pub fn pawn_attack(self, color: PieceColor, opponent: Bitboard, attack_left: bool) -> Bitboard {
        let pawn_mask=self & match(color,attack_left){
            (PieceColor::WHITE,true)=> !A_FILE,
            (PieceColor::WHITE,false)=>!H_FILE,
            (PieceColor::BLACK,true)=>!H_FILE,
            (PieceColor::BLACK,false)=>!A_FILE,
        };
        let attacks=match(color,attack_left){
            (PieceColor::WHITE,true)=>(pawn_mask << 7) & opponent,
            (PieceColor::WHITE,false)=>(pawn_mask << 9) & opponent,
            (PieceColor::BLACK,true)=>(pawn_mask >> 7) & opponent,
            (PieceColor::BLACK,false)=>(pawn_mask >> 9) & opponent,
        };
        attacks
    }

    #[inline(always)]
    pub fn pop_lsb(&mut self) -> u8 {
        let lsb = self.0 & (!self.0 + 1);
        self.0 ^= lsb;
        lsb.trailing_zeros() as u8
    }

    #[inline(always)]
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
    #[inline(always)]
    pub fn contains_square(&self, square: u8) -> bool {
        self.0 & (1u64 << square) != 0
    }

    #[inline(always)]
    pub fn get_bitboard(self) -> u64 {
        self.0
    }
}
impl Bitboard {
    /// Returns an iterator over the set bits in the bitboard.
    pub fn iter(self) -> BitboardIterator {
        BitboardIterator(self)
    }
}

pub struct BitboardIterator(Bitboard);

impl Iterator for BitboardIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == Bitboard::new(0) {
            None
        } else {
            Some(self.0.pop_lsb())
        }
    }
}
use std::fmt;
use std::ops::{BitAnd, Shl};
use derive_more::{AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, Not, Shr, Sub, SubAssign};
use crate::engine::movegen::constants::{A_FILE, H_FILE, RANK_2, RANK_7};


impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}