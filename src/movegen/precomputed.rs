use std::sync::LazyLock;
use crate::board::bitboard::Bitboard;
use crate::board::position::Position;
use crate::movegen::constants::{KING_OFFSETS, KNIGHT_OFFSETS};

pub static KNIGHT_MOVES: LazyLock<[Bitboard; 64]> = LazyLock::new(|| {
    let mut moves = [Bitboard::new(0); 64];
    for i in 0..64 {
        if let Some(pos) = Position::from_sqr(i) {
            let mut curr_move = Bitboard::new(0);
            for offset in KNIGHT_OFFSETS.iter() {
                let new_pos = pos + *offset;
                if let Some(sqr) = new_pos.to_sqr() {
                    curr_move.set_square(sqr as u8);
                }
            }
            moves[i as usize] = curr_move;
        }
    }
    moves
});

pub static KING_MOVES: LazyLock<[Bitboard; 64]> = LazyLock::new(|| {
    let mut moves = [Bitboard::new(0); 64];
    for i in 0..64 {
        if let Some(pos) = Position::from_sqr(i) {
            let mut curr_move = Bitboard::new(0);
            for offset in KING_OFFSETS.iter() {
                let new_pos = pos + *offset;
                if let Some(sqr) = new_pos.to_sqr() {
                    curr_move.set_square(sqr as u8);
                }
            }
            moves[i as usize] = curr_move;
        }
    }
    moves
});