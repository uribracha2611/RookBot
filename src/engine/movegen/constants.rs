use crate::engine::board::bitboard::Bitboard;
use crate::engine::board::position::Position;
use const_for::const_for;

pub const A_FILE: Bitboard = Bitboard::new(0x0101010101010101); // Mask for the a-file (bits 0, 8, 16, ..., 56)
pub const H_FILE: Bitboard = Bitboard::new(0x8080808080808080); // Mask for the h-file (bits 7, 15, 23, ..., 63)
pub const MAX_MOVES: usize = 218;
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
    Position { x: 1, y: 0 },   // Right
    Position { x: -1, y: 0 },  // Left
    Position { x: 0, y: 1 },   // Up
    Position { x: 0, y: -1 },  // Down
    Position { x: 1, y: 1 },   // Up-Right
    Position { x: 1, y: -1 },  // Down-Right
    Position { x: -1, y: 1 },  // Up-Left
    Position { x: -1, y: -1 }, // Down-Left
];
pub const ROOK_OFFSETS: [Position; 4] = [
    Position { x: 1, y: 0 },  // Right
    Position { x: -1, y: 0 }, // Left
    Position { x: 0, y: 1 },  // Up
    Position { x: 0, y: -1 }, // Down
];
pub const BISHOP_OFFSETS: [Position; 4] = [
    Position { x: 1, y: 1 },   // Up-Right
    Position { x: 1, y: -1 },  // Down-Right
    Position { x: -1, y: 1 },  // Up-Left
    Position { x: -1, y: -1 }, // Down-Left
];
pub const ALL_OFSET: [Position; 8] = [
    Position { x: 1, y: 0 },   // Right
    Position { x: -1, y: 0 },  // Left
    Position { x: 0, y: 1 },   // Up
    Position { x: 0, y: -1 },  // Down
    Position { x: 1, y: 1 },   // Up-Right
    Position { x: 1, y: -1 },  // Down-Right
    Position { x: -1, y: 1 },  // Up-Left
    Position { x: -1, y: -1 }, // Down-Left
];

pub const RANK_1: Bitboard = Bitboard::new(0x00000000000000FF);
pub const RANK_2: Bitboard = Bitboard::new(0x000000000000FF00);
pub const RANK_3: Bitboard = Bitboard::new(0x0000000000FF0000);
pub const RANK_4: Bitboard = Bitboard::new(0x00000000FF000000);
pub const RANK_5: Bitboard = Bitboard::new(0x000000FF00000000);
pub const RANK_6: Bitboard = Bitboard::new(0x0000FF0000000000);
pub const RANK_7: Bitboard = Bitboard::new(0x00FF000000000000);
pub const RANK_8: Bitboard = Bitboard::new(0xFF00000000000000);
pub const KNIGHT_MOVES: [Bitboard; 64] = {
    let mut moves = [Bitboard::new(0); 64];

    const_for!(i in 0..64 => {
        if let Some(pos) = Position::from_sqr(i) {
            let mut curr_move = Bitboard::new(0);
            const_for!(offset_index in 0..KNIGHT_OFFSETS.len()=> {
               let  offset=KNIGHT_OFFSETS[offset_index];
                let new_pos = pos.add_position_const(offset);
                if let Some(sqr) = new_pos.to_sqr() {
                    curr_move.set_square(sqr as u8);
                }
            });
            moves[i as usize] = curr_move;
        }
    });
    moves
};

pub const KING_MOVES: [Bitboard; 64] = {
    let mut moves = [Bitboard::new(0); 64];

    const_for!(i in 0..64 => {
        if let Some(pos) = Position::from_sqr(i) {
            let mut curr_move = Bitboard::new(0);
            const_for!(offset_index in 0..KING_OFFSETS.len()=> {
               let  offset=KING_OFFSETS[offset_index];
                let new_pos = pos.add_position_const(offset);
                if let Some(sqr) = new_pos.to_sqr() {
                    curr_move.set_square(sqr as u8);
                }
            });
            moves[i as usize] = curr_move;
        }
    });
    moves
};
pub const DIR_RAY_MASK: [[Bitboard; 8]; 64] = {
    let mut dir_ray_mask = [[Bitboard::new(0); 8]; 64];

    const_for!(square in 0..64=> {
        let pos = Position::from_sqr(square).unwrap();
           const_for!(dir_index in 0..8 => {
            let offset=offset_rook_bishop_from_index(dir_index);
            let mut ray = Bitboard::new(0);
            let mut current_pos = pos;
            loop {
                current_pos = current_pos.add_position_const(offset);
                if let Some(index) = current_pos.to_sqr() {
                    ray.set_square(index as u8);
                } else {
                    break;
                }
            }
            dir_ray_mask[square as usize][dir_index] = ray;
        })
    });

    dir_ray_mask
};

pub static ALIGN_MASK: [[Bitboard; 64]; 64] = {
    let mut align_mask = [[Bitboard::new(0); 64]; 64];

    const_for!(square_a in 0..64=> {
        const_for!(square_b in 0..64=> {
            let pos_a = Position::from_sqr(square_a).unwrap();
            let pos_b = Position::from_sqr(square_b).unwrap();
            let delta = pos_b.sub_position_const( pos_a);
            let dir = Position::new(delta.x.signum(), delta.y.signum());

            const_for!( i in -8..8=> {
                let coord = pos_a.add_position_const( dir.mul_position_const( i as i8)) ;
                if let Some(index) = coord.to_sqr() {
                    align_mask[square_a as usize][square_b as usize].set_square(index as u8);
                }
            });
        });
    });

    align_mask
};
pub static DIR_SQUARES: [[Position; 64]; 64] = {
    let mut dir_squares = [[Position::new(0, 0); 64]; 64];
    const_for!(square_a in 0..64=> {
        const_for!(square_b in 0..64=> {
            let pos_a = Position::from_sqr(square_a).unwrap();
            let pos_b = Position::from_sqr(square_b).unwrap();
            let delta = pos_b.sub_position_const( pos_a);
            let dir = Position::new(delta.x.signum(), delta.y.signum());
            dir_squares[square_a as usize][square_b as usize] = dir;
        });
    });
    dir_squares
};
pub static SQR_A_B_MASK: [[Bitboard; 64]; 64] = {
    let mut align_mask = [[Bitboard::new(0); 64]; 64];

    const_for!(square_a in 0..64=> {
        const_for!(square_b in 0..64=> {
            let pos_a = Position::from_sqr(square_a).unwrap();
            let pos_b = Position::from_sqr(square_b).unwrap();
       let delta = pos_b.sub_position_const( pos_a);
            let dir = Position::new(delta.x.signum(), delta.y.signum());

            const_for!( i in 1..8 => {
                let coord = pos_a.add_position_const( dir.mul_position_const(i as i8)) ;
                if let Some(index) = coord.to_sqr() {
                    if index == square_b {
                        align_mask[square_a as usize][square_b as usize].set_square(index as u8);
                        break;
                    }

                    align_mask[square_a as usize][square_b as usize].set_square(index as u8);
                }
            });
        });
    });

    align_mask
};
pub static NUM_SQUARES_FROM_SQUARE: [[u8; 64]; 8] = {
    let mut num_squares_from_square = [[0; 64]; 8];

    const_for!(square in 0..64=> {
        let pos = Position::from_sqr(square).unwrap();
            const_for! (dir_index in 0..8 =>  {
            let offset=offset_rook_bishop_from_index(dir_index);
            let mut count = 0;
            let mut current_pos = pos;
            loop {
                current_pos = current_pos.add_position_const(offset);
                if current_pos.to_sqr().is_some() {
                    count += 1;
                } else {
                    break;
                }
            }
            num_squares_from_square[dir_index][square as usize] = count;
        });
    });

    num_squares_from_square
};
const fn offset_rook_bishop_from_index(index: usize) -> Position {
    if index < 4 {
        ROOK_OFFSETS[index]
    } else {
        BISHOP_OFFSETS[index - 4]
    }
}