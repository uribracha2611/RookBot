use crate::board::bitboard::Bitboard;
use crate::board::position::Position;
use crate::movegen::constants::{BISHOP_OFFSETS, ROOK_OFFSETS};
use crate::movegen::magic::constants::{BISHOP_MAGICS, BISHOP_SHIFTS, ROOK_MAGICS, ROOK_SHIFTS};
use crate::movegen::magic::precomputed::{BISHOP_ATTACKS, BISHOP_MASK, ROOK_ATTACKS, ROOK_MASK};

pub fn build_mask_square(start_square:u8, is_rook:bool) ->Bitboard{
    let mut mask=Bitboard::new(0);
    let start_square_coord=Position::from_sqr(start_square as i8).unwrap();
    let offsets=if is_rook{ROOK_OFFSETS} else {BISHOP_OFFSETS};
    for coord in offsets
    {
        for i in 1..8 {
            let new_square = start_square_coord + (coord * i);
            if let (Some(new_square)) = new_square.to_sqr() {
                mask.set_square(new_square as u8);
            } else {
                break;
            }
        }
    }
    mask
}

pub fn build_mask(is_rook:bool)->[Bitboard;64]{
    let mut mask=[Bitboard::new(0);64];
    for i in 0..64{
        mask[i]=build_mask_square(i as u8,is_rook);
    }
    mask
}
pub fn build_blocker_bitboards(mask: Bitboard)->Vec<Bitboard>{

    let mask_set_vec = mask.bitboard_to_set_vec();
    let blocker_count=(1<<mask_set_vec.len()) as usize;
    let mut blocker_bitboards:Vec<Bitboard>=Vec::with_capacity(blocker_count);
    for pattern_index in 0..blocker_count{
        for  bit_index in 0..mask_set_vec.len(){
            let bit=(pattern_index>>bit_index)&1;
            blocker_bitboards[pattern_index]|=Bitboard::new ((bit as u64)<<mask_set_vec[bit_index]);
        }

    }
    blocker_bitboards
}
pub fn legal_move_bitboard_from_blockers(start_square: u8, blocker_bitboard: Bitboard, ortho: bool) -> Bitboard {
    let mut bitboard = Bitboard::new(0);
    let directions = if ortho { ROOK_OFFSETS } else { BISHOP_OFFSETS };
    let start_coord = Position::from_sqr(start_square as i8).unwrap();

    for dir in directions.iter() {
        for dst in 1..8 {
            let coord = start_coord + (*dir * dst);
            if let Some(square_index) = coord.to_sqr() {
                bitboard.set_square(square_index as u8);
                if blocker_bitboard.contains_square(square_index as u8) {
                    break;
                }
            } else {
                break;
            }
        }
    }

    bitboard
}


pub fn create_table(square: u8, rook: bool, magic: u64, left_shift: u8) -> Vec<Bitboard> {
    let num_bits = 64 - left_shift;
    let lookup_size = 1 << num_bits;
    let mut table = vec![Bitboard::new(0); lookup_size];

    let movement_mask = build_mask_square(square, rook);
    let blocker_patterns = build_blocker_bitboards(movement_mask);

    for pattern in blocker_patterns {
        let index = (pattern.get_bitboard() * magic) >> left_shift;
        let moves = legal_move_bitboard_from_blockers(square, pattern, rook);
        table[index as usize] = moves;
    }

    table
}
pub fn get_rook_attacks(square: usize, blockers: Bitboard) -> Bitboard {
    let key = ((blockers & ROOK_MASK[square]).get_bitboard() * ROOK_MAGICS[square]) >> ROOK_SHIFTS[square];
    ROOK_ATTACKS[square][key as usize]
}

pub fn get_bishop_attacks(square: usize, blockers: Bitboard) -> Bitboard {
    let key = ((blockers & BISHOP_MASK[square]).get_bitboard() * BISHOP_MAGICS[square]) >> BISHOP_SHIFTS[square];
    BISHOP_ATTACKS[square][key as usize]
}
