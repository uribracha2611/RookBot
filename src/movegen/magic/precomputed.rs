use std::sync::LazyLock;
use crate::board::bitboard::Bitboard;
use crate::movegen::magic::functions::{build_mask_square, create_table};
use crate::movegen::magic::constants::{BISHOP_MAGICS, BISHOP_SHIFTS, ROOK_MAGICS, ROOK_SHIFTS};

pub static ROOK_MASK: LazyLock<[Bitboard; 64]> = LazyLock::new(|| {
    let mut mask = [Bitboard::new(0); 64];
    for square_index in 0..64 {
        mask[square_index] = build_mask_square(square_index as u8, true);
    }
    mask
});

pub static BISHOP_MASK: LazyLock<[Bitboard; 64]> = LazyLock::new(|| {
    let mut mask = [Bitboard::new(0); 64];
    for square_index in 0..64 {
        mask[square_index] = build_mask_square(square_index as u8, false);
    }
    mask
});

pub static ROOK_ATTACKS: LazyLock<Vec<Vec<Bitboard>>> = LazyLock::new(|| {
    let mut attacks: Vec<Vec<Bitboard>> = vec![Vec::new(); 64];
    for i in 0..64 {
        attacks[i] = create_table(i as u8, true, ROOK_MAGICS[i], ROOK_SHIFTS[i]);
    }
    attacks
});

pub static BISHOP_ATTACKS: LazyLock<Vec<Vec<Bitboard>>> = LazyLock::new(|| {
    let mut attacks: Vec<Vec<Bitboard>> = vec![Vec::new(); 64];
    for i in 0..64 {
        attacks[i] = create_table(i as u8, false, BISHOP_MAGICS[i], BISHOP_SHIFTS[i]);
    }
    attacks
});

