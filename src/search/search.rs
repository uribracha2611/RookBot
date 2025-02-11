use crate::board::board::Board;
use crate::board::piece::PieceColor;
use crate::movegen::generate::generate_moves;
use crate::movegen::movedata::MoveData;
use crate::movegen::movelist::MoveList;
use crate::search::constants::INFINITY;
use crate::search::move_ordering::get_move_score;
use crate::search::types::{ChosenMove, SearchInput, SearchOutput};

pub fn eval(board: &Board) ->i32{
    let mg_phase=board.game_phase.min(24);
    let eg_phase=24-mg_phase;
    let mg_score=board.psqt_white.get_middle_game()-board.psqt_black.get_middle_game();
    let eg_score=board.psqt_white.get_end_game()-board.psqt_black.get_end_game();
    let score=(mg_score*mg_phase+eg_score*eg_phase)/24;
    
     if board.turn==PieceColor::WHITE{
        score
    }
    else{
        -score
    }

}
pub fn pick_move(ml: &mut MoveList, start_index: u8) {
    for i in (start_index + 1)..(ml.len() as u8) {
        if get_move_score(ml.get_move(i as usize)) > get_move_score(ml.get_move(start_index as usize)) {
            ml.swap(start_index as usize, i as usize);
        }
    }
}
pub fn search(mut board: &mut Board, input: SearchInput) -> SearchOutput {
    fn search_internal(board: &mut Board, depth: u8, alpha: &mut i32, beta: &mut i32, nodes_evaluated: &mut i32, pv: &mut Vec<MoveData>) -> i32 {
        if depth == 0 {
            return eval(board);
        }

        let mut best_eval = -INFINITY;
        let mut local_pv = Vec::new();
        let mut moves = generate_moves(board);
        
        for i in 0..moves.len() {
            pick_move(&mut moves, i as u8);
            let mov=&moves[i];
            *nodes_evaluated += 1;
            board.make_move(mov);
            let curr_eval = -search_internal(board, depth - 1, &mut -(*beta), &mut -(*alpha), nodes_evaluated, &mut local_pv);
            board.unmake_move(mov);

            if curr_eval > best_eval {
                best_eval = curr_eval;
                *pv = local_pv.clone();
                pv.push(*mov);
            }

            if best_eval > *alpha {
                *alpha = best_eval;
            }

            if *alpha >= *beta {
                break;
            }
        }
        best_eval
    }

    let mut nodes_evaluated = 0;
    let mut alpha = -INFINITY;
    let mut beta = INFINITY;
    let mut principal_variation = Vec::new();
    let eval=search_internal(board, input.depth, &mut alpha, &mut beta, &mut nodes_evaluated, &mut principal_variation);
    principal_variation.reverse();
    SearchOutput {
        nodes_evaluated,
        principal_variation,
        eval
        
    }
}
