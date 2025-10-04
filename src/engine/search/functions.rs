use crate::engine::board::board::Board;
use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::types::SearchRefs;

pub fn is_allowed_futility_pruning(
    depth: u8,
    alpha: i32,
    eval: i32,
    mv: &MoveData,
    board: &Board,
) -> bool {
    if depth > 8 || depth == 0 {
        return false;
    }

    if mv.is_capture() || mv.is_promotion() || board.is_check {
        return false;
    }
    eval <= alpha - (100 + ((depth as i32) * 150))
}
pub fn is_allowed_reverse_futility_pruning(
    depth: u8,
    beta: i32,
    eval: i32,
    board: &Board,
    improving: bool,
) -> bool {
    if depth > 9 || depth == 0 {
        return false; // Lower depth threshold
    }

    if board.is_check {
        return false; // Avoid pruning in check
    }

    let margin = if improving { 50 } else { 100 };
    let rep_margin = margin * (depth as i32); // Lower margin
    eval - rep_margin >= beta
}
#[inline(always)]
pub fn is_improving(board: &Board, eval: i32, refs: &SearchRefs, ply: i32) -> bool {
    if board.is_check {
        return false;
    }
    return if ply >= 2 && let Some(two_moves_ago) = refs.get_eval_ply(ply - 2) {
        eval > two_moves_ago
    } else if ply >= 4 && let Some(four_moves_ago) = refs.get_eval_ply(ply - 4) {
        eval > four_moves_ago
    } else {
        true
    };
}

