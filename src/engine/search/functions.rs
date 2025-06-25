use std::thread::current;
use crate::engine::board::board::Board;
use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::constants::FUTILITY_MARGIN_DEPTH;
use crate::engine::search::types::SearchRefs;

pub fn is_allowed_futility_pruning(depth:u8, alpha:i32, eval:i32, mv:&MoveData, board: &Board) -> bool {
    if depth > 2 || depth==0 {
        return false;
    }

    if mv.is_capture()  || mv.is_promotion() || board.is_check  {
        return false;
    }
    eval<=alpha-FUTILITY_MARGIN_DEPTH[(depth-1) as usize]



}
pub fn is_allowed_reverse_futility_pruning(depth: u8, beta: i32, eval: i32, board: &Board,improving:bool) -> bool {
    if depth > 9 || depth == 0 {
        return false; // Lower depth threshold
    }

    if board.is_check {
        return false; // Avoid pruning in check
    }

    let  margin= if improving {50} else { 100 };
    let rep_margin = margin* (depth as i32); // Lower margin
    eval - rep_margin >= beta
}
pub fn is_improving(refs: &SearchRefs, ply: i32) -> bool {
    
    if ply<2{
        return  false;
    }
    if let (Some(this_depth_eval), Some(two_moves_ago_eval)) = (
        refs.get_eval_ply(ply),
        refs.get_eval_ply(ply - 2),
    ) {
        if this_depth_eval > two_moves_ago_eval{
            return true;
        }
        
        
    }
    if ply<4{
        return  false;
    }
     if let (Some(this_depth_eval), Some(four_moves_ago_eval)) = (
        refs.get_eval_ply(ply),
        refs.get_eval_ply(ply - 4),
    ){
        if this_depth_eval > four_moves_ago_eval{
            return true;
        }
    }

    false
}