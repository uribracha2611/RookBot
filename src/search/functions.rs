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
