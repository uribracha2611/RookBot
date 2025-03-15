
use crate::board::board::Board;
use crate::movegen::movedata::MoveData;
use crate::search::constants::INFINITY;

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
pub fn should_movecount_based_pruning(board: &Board, mv: MoveData, depth: u32, moves_played_so_far: i32, scores: &[i32]) -> bool {
    if board.is_check {
        return false;
    }

    let score = if moves_played_so_far < scores.len() as i32 {
        scores[moves_played_so_far as usize]
    } else {
        0 // default value indicating no score for this move
    };

    let prerequisites = !mv.is_capture() || score < 0;
    if !prerequisites {
        return false;
    }

    let move_count_required: i32 = match depth {
        1 => 50,
        2 => 50,
        3 => 50,
        4 => 50,
        5 => 50,
        6 => 100,
        _ => INFINITY,
    };

    moves_played_so_far > move_count_required
}
