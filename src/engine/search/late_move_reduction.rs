use num_traits::abs;
use crate::engine::board::board::Board;
use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::constants::MATE_VALUE;

pub fn reduce_depth(board: &Board, mv: &MoveData, depth: f64, moves_played: f64) -> f64 {
    if mv.is_capture() || mv.is_promotion() {
        if board.is_check {
            depth - 2.0
        } else {
            depth - 3.0
        }
    } else {
        let reg_reduction =depth - (0.7844 + (depth.ln() * moves_played.ln()) / 2.4696);



        reg_reduction.clamp(1.0, depth.floor())
        
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
