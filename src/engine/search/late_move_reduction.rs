use crate::engine::board::board::Board;
use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::constants::MATE_VALUE;
use num_traits::abs;

pub fn reduce_depth(
    board: &Board,
    mv: &MoveData,
    depth: f32,
    moves_played: f32,
    improving: bool,
) -> f32 {
    if mv.is_capture() || mv.is_promotion() {
        if board.game_state.is_check {
            depth - 2.0
        } else {
            depth - 3.0
        }
    } else {
        let reg_reduction = depth - (0.7844 + (depth.ln() * moves_played.ln()) / 2.4696);

        let actual_reduction = if !improving {
            reg_reduction - 1.0
        } else {
            reg_reduction
        };

        actual_reduction.clamp(1.0, depth.floor())
    }
}
pub fn should_movecount_based_pruning(
    board: &Board,
    mv: MoveData,
    depth: u32,
    moves_played_so_far: i32,
    best_score: i32,
    improving: bool,
) -> bool {
    if depth >= 4 {
        return false;
    }

    if abs(best_score) >= MATE_VALUE - 100 {
        return false;
    }
    let factor = if improving { 1 } else { 2 };
    moves_played_so_far > (3 + (depth * depth) / factor) as i32
}
