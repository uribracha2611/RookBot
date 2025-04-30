use num_traits::{abs, Float};
use crate::board::board::Board;
use crate::movegen::movedata::MoveData;
use crate::search::constants::MATE_VALUE;

pub fn reduce_depth(board: &Board, mv: &MoveData, depth: f64, moves_played: f64,improving:bool) -> f64 {
    if mv.is_capture() || mv.is_promotion() {
        if board.is_check {
            depth - 2.0
        } else {
            depth - 3.0
        }
    } else {
        let reg_reduction =depth - (0.7844 + (depth.ln() * moves_played.ln()) / 2.4696);
        let actual_reduction=if !improving{
            reg_reduction-1.0
        }
        else { 
            reg_reduction
        };
        actual_reduction.clamp(1.0, depth.floor())
        
    }
}
pub fn should_movecount_based_pruning(board: &Board, mv: MoveData, depth: u32, moves_played_so_far: i32,best_score:i32,improving:bool ) -> bool {
    if depth>=4{
        return false;
    }


    if  abs(best_score)>= MATE_VALUE-100 {
        return false;
    }
let factor=if improving{1} else { 2 };
     moves_played_so_far> ((3 + depth * depth) / factor) as i32


}
