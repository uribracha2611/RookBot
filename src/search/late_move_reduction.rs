use crate::board::board::Board;
use crate::movegen::movedata::MoveData;

pub fn reduce_depth(board: &Board, mv: &MoveData, depth: f64, moves_played: f64) -> f64 {
    if mv.is_capture() || mv.is_promotion() {
        if board.is_check {
            depth - 2.0
        } else {
            depth - 3.0
        }
    } else {
        depth - (0.7844 + (depth.ln() * moves_played.ln()) / 2.4696)
    }
}