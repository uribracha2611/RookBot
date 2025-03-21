use crate::board::board::Board;
use crate::movegen::movedata::MoveData;
use crate::search::constants::FUTILITY_MARGIN_DEPTH;
use crate::search::search::eval;


pub fn is_allowed_futility_pruning(depth:u8, alpha:i32, eval:i32, mv:&MoveData, board: &Board) -> bool {
    if depth > 2 || depth==0 {
        return false;
    }

    if mv.is_capture()  || mv.is_promotion() || board.is_check  {
        return false;
    }
    eval<=alpha-FUTILITY_MARGIN_DEPTH[(depth-1) as usize]



}
pub fn is_allowed_reverse_futility_pruning(depth: u8, beta: i32, eval: i32, board: &Board) -> bool {
    if depth > 9 || depth == 0 {
        return false; // Lower depth threshold
    }

    if board.is_check {
        return false; // Avoid pruning in check
    }

    let rep_margin = 150 * depth as i32; // Lower margin
    eval - rep_margin >= beta
}