use crate::engine::board::board::Board;
use crate::engine::board::piece::PieceColor::WHITE;
use crate::engine::board::piece::{Piece, PieceColor};
use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::constants::MATE_VALUE;
use crate::engine::search::nnue::simd;
use crate::engine::search::nnue::simd::evaluate_part;

const FEATURE_COUNT: usize = 768;
const SCALE: i32 = 400;
const QA: i16 = 255;
const QB: i16 = 64;
const HL: usize = 256;
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

        output.clamp(-(MATE_VALUE - 1024), MATE_VALUE - 1024)
    }
    //assumes the move hasn't been made on the board yet
    pub fn handle_mv_nnue(&self, board: &mut Board, mv: MoveData) {
        let from = mv.from() as usize;
        let to = mv.to() as usize;
        let moved_piece = board.game_state.squares[from].unwrap();
        let (from_white_index, from_black_index) = get_feature_indices(moved_piece, from);

        let (from_white_weights, from_black_weights) = (
            self.feature_weights[from_white_index].vals.as_ptr(),
            self.feature_weights[from_black_index].vals.as_ptr(),
        );

        let (to_white_index, to_black_index) = if mv.is_promotion() {
            get_feature_indices(mv.get_promotion_piece(board.turn), to)
        } else {
            get_feature_indices(moved_piece, to)
        };

        let (to_white_weights, to_black_weights) = (
            self.feature_weights[to_white_index].vals.as_ptr(),
            self.feature_weights[to_black_index].vals.as_ptr(),
        );
        if mv.is_capture() {
            let capture_square = mv.get_capture_square() as usize;
            let captured_piece = board.game_state.squares[capture_square].unwrap();
            let (capture_white_index, capture_black_index) =
                get_feature_indices(captured_piece, capture_square);
            let (capture_white_weights, capture_black_weights) = (
                self.feature_weights[capture_white_index].vals.as_ptr(),
                self.feature_weights[capture_black_index].vals.as_ptr(),
            );
            unsafe {
                simd::add_1_sub_2(
                    board.game_state.acc_white.vals.as_mut_ptr(),
                    to_white_weights,
                    from_white_weights,
                    capture_white_weights,
                    HL,
                );

                simd::add_1_sub_2(
                    board.game_state.acc_black.vals.as_mut_ptr(),
                    to_black_weights,
                    from_black_weights,
                    capture_black_weights,
                    HL,
                );
            }
        } else {
            if mv.is_castle() {
                let rook_start = mv.get_rook_start(board.turn) as usize;
                let rook_end = mv.get_rook_end(board.turn) as usize;
                let piece_rook = board.game_state.squares[rook_start].unwrap();
                let (rook_start_white_index, rook_start_black_index) =
                    get_feature_indices(piece_rook, rook_start);
                let (rook_start_white_weights, rook_start_black_weights) = (
                    self.feature_weights[rook_start_white_index].vals.as_ptr(),
                    self.feature_weights[rook_start_black_index].vals.as_ptr(),
                );
                let (rook_end_white_index, rook_end_black_index) =
                    get_feature_indices(piece_rook, rook_end);
                let (rook_end_white_weights, rook_end_black_weights) = (
                    self.feature_weights[rook_end_white_index].vals.as_ptr(),
                    self.feature_weights[rook_end_black_index].vals.as_ptr(),
                );
                unsafe {
                    simd::add_2_sub_2(
                        board.game_state.acc_white.vals.as_mut_ptr(),
                        rook_end_white_weights,
                        to_white_weights,
                        from_white_weights,
                        rook_start_white_weights,
                        HL,
                    );

                    simd::add_2_sub_2(
                        board.game_state.acc_black.vals.as_mut_ptr(),
                        rook_end_black_weights,
                        to_black_weights,
                        from_black_weights,
                        rook_start_black_weights,
                        HL,
                    );
                }
            } else {
                unsafe {
                    simd::add_1_sub_1(
                        board.game_state.acc_white.vals.as_mut_ptr(),
                        to_white_weights,
                        from_white_weights,
                        HL,
                    );

                    simd::add_1_sub_1(
                        board.game_state.acc_black.vals.as_mut_ptr(),
                        to_black_weights,
                        from_black_weights,
                        HL,
                    );
                }
            }
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
