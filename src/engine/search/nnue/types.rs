use crate::engine::board::piece::PieceColor::WHITE;
use crate::engine::board::piece::{Piece, PieceColor};
use crate::engine::search::nnue::simd;
use crate::engine::search::nnue::simd::evaluate_part;

const FEATURE_COUNT: usize = 768;
const SCALE: i32 = 400;
const QA: i16 = 255;
const QB: i16 = 64;
const HL: usize = 64;
pub fn get_feature_indices(piece: Piece, sq: usize) -> (usize, usize) {
    let pc = piece.piece_color;
    let pt = piece.piece_type as usize;

    let is_piece_white = piece.piece_color == WHITE;
    let white_pt = if is_piece_white { pt } else { pt + 6 };
    let black_pt = if !is_piece_white { pt } else { pt + 6 };
    let sq_flipped = sq ^ 56;
    (white_pt * 64 + sq, black_pt * 64 + sq_flipped)
}
#[repr(C, align(64))]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Accumulator {
    pub(crate) vals: [i16; HL],
}
#[repr(C)]
pub struct Network {
    feature_weights: [Accumulator; FEATURE_COUNT],
    feature_bias: Accumulator,
    output_weight: [i16; 2 * HL],
    output_bias: i16,
}
impl Network {
    pub fn evaluate_nnue(&self, us: &Accumulator, them: &Accumulator) -> i32 {
        let mut output = unsafe {
            evaluate_part(&us.vals, &self.output_weight, HL)
                + evaluate_part(&them.vals, &self.output_weight[HL..], HL)
        };

        output /= i32::from(QA);
        output += i32::from(self.output_bias);
        output *= SCALE;
        output /= i32::from(QA) * i32::from(QB);

        output
    }

    pub fn update_piece(
        &self,
        piece: Piece,
        sq: usize,
        white_acc: &mut Accumulator,
        black_acc: &mut Accumulator,
        add: bool,
    ) {
        let (friendly_idx, opponent_idx) = get_feature_indices(piece, sq);

        if add {
            white_acc.add_feature(friendly_idx, self);
            black_acc.add_feature(opponent_idx, self);
        } else {
            white_acc.remove_feature(friendly_idx, self);
            black_acc.remove_feature(opponent_idx, self);
        }
    }
}
impl Accumulator {
    pub fn new(net: &Network) -> Self {
        net.feature_bias
    }

    #[inline(always)]
    pub(crate) fn add_feature(&mut self, feature_idx: usize, net: &Network) {
        unsafe {
            simd::add(
                self.vals.as_mut_ptr(),
                net.feature_weights[feature_idx].vals.as_ptr(),
                HL,
            );
        }
    }

    #[inline(always)]
    pub fn remove_feature(&mut self, feature_idx: usize, net: &Network) {
        unsafe {
            simd::sub(
                self.vals.as_mut_ptr(),
                net.feature_weights[feature_idx].vals.as_ptr(),
                HL,
            );
        }
    }
}
