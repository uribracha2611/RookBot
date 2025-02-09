use std::sync::LazyLock;
use crate::board::bitboard::Bitboard;
use crate::board::position::Position;
use crate::movegen::constants::{BISHOP_OFFSETS, KING_OFFSETS, KNIGHT_OFFSETS, ROOK_OFFSETS};

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
pub static DIR_RAY_MASK: LazyLock<[[Bitboard; 8]; 64]> = LazyLock::new(|| {
    let mut dir_ray_mask = [[Bitboard::new(0); 8]; 64];

    for square in 0..64 {
        let pos = Position::from_sqr(square).unwrap();
        for (dir_index, offset) in ROOK_OFFSETS.iter().chain(BISHOP_OFFSETS.iter()).enumerate() {
            let mut ray = Bitboard::new(0);
            let mut current_pos = pos;
            loop {
                current_pos = current_pos + *offset;
                if let Some(index) = current_pos.to_sqr() {
                    ray.set_square(index as u8);
                } else {
                    break;
                }
            }
            dir_ray_mask[square as usize][dir_index] = ray;
        }
    }

    dir_ray_mask
});
pub static ALIGN_MASK: LazyLock<[[Bitboard; 64]; 64]> = LazyLock::new(|| {
    let mut align_mask = [[Bitboard::new(0); 64]; 64];

    for square_a in 0..64 {
        for square_b in 0..64 {
            let pos_a = Position::from_sqr(square_a).unwrap();
            let pos_b = Position::from_sqr(square_b).unwrap();
            let delta = pos_b - pos_a;
            let dir = Position::new(delta.x.signum(), delta.y.signum());

            for i in -8..8 {
                let coord = pos_a + dir * i as i8;
                if let Some(index) = coord.to_sqr() {
                    align_mask[square_a as usize][square_b as usize].set_square(index as u8);
                }
            }
        }
    }

    align_mask
});
pub static SQR_A_B_MASK: LazyLock<[[Bitboard; 64]; 64]> = LazyLock::new(|| {
    let mut align_mask = [[Bitboard::new(0); 64]; 64];

    for square_a in 0..64 {
        for square_b in 0..64 {
            let pos_a = Position::from_sqr(square_a).unwrap();
            let pos_b = Position::from_sqr(square_b).unwrap();
            let delta = pos_b - pos_a;
            let dir = Position::new(delta.x.signum(), delta.y.signum());

            for i in 1..8 {
                let coord = pos_a + dir * i as i8;
                if let Some(index) = coord.to_sqr()  {
                    if(index == square_b ){
                        align_mask[square_a as usize][square_b as usize].set_square(index as u8);
                        break;
                    }

                    align_mask[square_a as usize][square_b as usize].set_square(index as u8);
                }
            }
        }
    }

    align_mask
});
pub static NUM_SQUARES_FROM_SQUARE: LazyLock<[[u8; 64]; 8]> = LazyLock::new(|| {
    let mut num_squares_from_square = [[0; 64]; 8];

    for square in 0..64 {
        let pos = Position::from_sqr(square).unwrap();
        for (dir_index, offset) in ROOK_OFFSETS.iter().chain(BISHOP_OFFSETS.iter()).enumerate() {
            let mut count = 0;
            let mut current_pos = pos;
            loop {
                current_pos = current_pos + *offset;
                if current_pos.to_sqr().is_some() {
                    count += 1;
                } else {
                    break;
                }
            }
            num_squares_from_square[dir_index][square as usize] = count;
        }
    }
    num_squares_from_square
    });
