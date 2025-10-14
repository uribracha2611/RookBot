use crate::engine::board::board::Board;
use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::constants::{LMR_TABLE, MATE_VALUE};
use num_traits::abs;
use num_traits::real::Real;

pub fn reduce_depth(
    board: &Board,
    mv: &MoveData,
    depth: i32,
    moves_played: i32,
    improving: bool,
) -> i32 {
    if mv.is_capture() || mv.is_promotion() {
        if board.game_state.is_check {
            depth - 2
        } else {
            depth - 3
        }
    } else {
        let move_index = moves_played.min(63);
        let reg_reduction = LMR_TABLE[depth as usize][move_index as usize] as i32;

        if !improving {
            reg_reduction - 1
        } else {
            reg_reduction
        }
    }
}
pub fn should_movecount_based_pruning(
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
