use crate::engine::board::board::Board;
use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::constants::{LMR_TABLE, MATE_VALUE};
use num_traits::abs;

pub fn reduce_depth(
    _board: &Board,
    _mv: &MoveData,
    depth: i32,
    moves_played: i32,
    improving: bool,
) -> i32 {
    let move_index = moves_played.min(63);
    let lmr_num = unsafe { LMR_TABLE[depth as usize][move_index as usize] };
    lmr_num + !improving as i32
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
