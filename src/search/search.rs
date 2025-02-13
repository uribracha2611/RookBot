use std::cmp::PartialEq;
use crate::board::board::Board;
use crate::board::piece::PieceColor;
use crate::movegen::generate::generate_moves;
use crate::movegen::movedata::MoveData;
use crate::movegen::movelist::MoveList;
use crate::search::constants::INFINITY;
use crate::search::move_ordering::{get_move_score, get_moves_score, store_killers, KillerMoves};
use crate::search::transposition_table::{Entry, EntryType, TRANSPOSITION_TABLE};
use crate::search::types::{ChosenMove, SearchInput, SearchOutput};

use std::sync::MutexGuard;


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
pub fn pick_move(ml: &mut MoveList, start_index: u8,scores:&Vec<u8>) {

    for i in (start_index + 1)..(ml.len() as u8) {
        if scores[i as usize] > scores[start_index as usize] {
            ml.swap(start_index as usize, i as usize);
        }
    }
}

pub fn search(mut board: &mut Board, input: SearchInput) -> SearchOutput {
    let mut nodes_evaluated = 0;
    let mut principal_variation = vec![MoveData::defualt(); input.depth as usize];
    let mut best_eval = -INFINITY;
    let mut  killer_moves = [[MoveData::defualt(); 2]; 256];
    for current_depth in 1..=input.depth {
        let  alpha = -INFINITY;
        let  beta = INFINITY;
       

        let eval = search_internal(&mut board, current_depth as i32, 0, alpha, beta, &mut nodes_evaluated, &mut principal_variation, &mut killer_moves);
        best_eval = eval;
    }

    SearchOutput {
        nodes_evaluated,
        principal_variation,
        eval: best_eval,
    }
}

fn search_internal(
    board: &mut Board,
    depth: i32,
    ply: i32,
    mut alpha: i32,
    beta: i32,
    nodes_evaluated: &mut i32,
    pv: &mut Vec<MoveData>,
    killer_moves: &mut KillerMoves
) -> i32 {
    if depth == 0 {
        return eval(board);
    }

    if let Some(entry) = TRANSPOSITION_TABLE.lock().unwrap().retrieve(board.game_state.zobrist_hash, depth as u8, alpha, beta) {
        if ply == 0 {
            pv[ply as usize] = entry.best_move;
        }
        return entry.eval;
    }

    let mut best_move = MoveData::defualt();
    let mut entry_type = EntryType::UpperBound;
    let mut move_list = generate_moves(board);
    let mut local_pv = vec![MoveData::defualt(); pv.len()]; 
    // Store a local PV
     let tt_move =TRANSPOSITION_TABLE.lock().unwrap().get_TT_move(board.game_state.zobrist_hash).unwrap_or(MoveData::defualt());
    let move_score=get_moves_score(&move_list, &tt_move, *killer_moves, ply as usize);
    for i in 0..move_list.len() {
        *nodes_evaluated += 1;
        pick_move(&mut move_list, i as u8,&move_score);
        let curr_move = move_list.get_move(i);

        board.make_move(curr_move);
        let score_mv = -search_internal(board, depth - 1, ply + 1, -beta, -alpha, nodes_evaluated, &mut local_pv,killer_moves);
        board.unmake_move(curr_move);

        if score_mv >= beta {
            entry_type = EntryType::LowerBound;
            best_move = *curr_move;
           
            TRANSPOSITION_TABLE.lock().unwrap().store(board.game_state.zobrist_hash, depth as u8, score_mv, entry_type, best_move);
            if !curr_move.is_capture() {
                store_killers(killer_moves, *curr_move, ply as usize);
            }
            return beta;
        }
            
        if score_mv > alpha {
            alpha = score_mv;
            best_move = *curr_move;
            entry_type = EntryType::Exact;

            // **Update PV:** Copy local PV down the line
            pv[ply as usize] = *curr_move;
          pv[ply as usize + 1..].copy_from_slice(&local_pv[ply as usize + 1..]);
        }

      
    }

    TRANSPOSITION_TABLE.lock().unwrap().store(board.game_state.zobrist_hash, depth as u8, alpha, entry_type, best_move);
    alpha
}

