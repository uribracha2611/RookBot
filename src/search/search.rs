
use crate::board::board::Board;
use crate::board::piece::PieceColor;
use crate::movegen::generate::generate_moves;
use crate::movegen::movedata::MoveData;
use crate::movegen::movelist::MoveList;
use crate::search::constants::{ INFINITY, MATE_VALUE, VAL_WINDOW};
use crate::search::move_ordering::{get_capture_score, get_moves_score, BASE_CAPTURE, MVV_LVA};
use crate::search::transposition_table::{EntryType, TRANSPOSITION_TABLE};
use crate::search::types::{CaptureHistoryTable, SearchInput, SearchOutput, SearchRefs};

use std::time::{Duration, Instant};
use crate::board::see::static_exchange_evaluation;
use crate::search::functions::{is_allowed_futility_pruning, is_allowed_reverse_futility_pruning};
use crate::search::late_move_reduction::{reduce_depth, should_movecount_based_pruning};

pub fn quiescence_search(
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    refs: &mut SearchRefs,
    
    
) -> i32 {
    // Check if time has exceeded
    if refs.is_time_done() {
        return 0;
    }
  
    refs.increment_nodes_evaluated();
        let stand_pat = eval(board);
        let mut best_val = stand_pat;

        // Alpha-Beta pruning
        if stand_pat >= beta {
         
            return stand_pat;
        }
        if alpha < stand_pat {
         
            alpha = stand_pat;
        }

        let mut moves = generate_moves(board, true);
        let TT_Move = TRANSPOSITION_TABLE.lock().unwrap().get_TT_move(board.game_state.zobrist_hash).unwrap_or(MoveData::defualt());
        let mut scores = get_capture_score(board, moves, TT_Move,refs);

        // Iterate through the moves
        for i in 0..moves.len() {


            // Pick the move to search next
            pick_move(&mut moves, i as u8, &mut scores);
            let mv = moves.get_move(i);

            if *mv!=TT_Move && static_exchange_evaluation(board, mv.get_capture_square().unwrap() as i32,mv.get_captured_piece().unwrap(),mv.piece_to_move, mv.from as i32)<0 {
                continue
            }
        
            

            // Check time again before making a move

          if refs.is_time_done() {
              
                    return alpha; // Return the best evaluation if time is up
                }


            // Make the move and perform recursive quiescence search
            board.make_move(mv);
            let score = -quiescence_search(board, -beta, -alpha, refs);
            board.unmake_move(mv);

            // Apply pruning if necessary
            if score >= beta {
                
                return score;
            }

            if score > best_val {
                best_val = score;
            }

            if score > alpha {
                alpha = score;
            }
        }
    
        best_val
    }




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
pub fn pick_move(ml: &mut MoveList, start_index: u8, scores: &mut Vec<i32>) {

    for i in (start_index + 1)..(ml.len() as u8) {
        if scores[i as usize] > scores[start_index as usize] {
            ml.swap(start_index as usize, i as usize);
            scores.swap(start_index as usize, i as usize);
        }
    }
}

pub fn search(mut board: &mut Board, input: &SearchInput) -> SearchOutput {
    let mut current_depth=1;
    let history_table=[[[0;64];64];2];
    let mut principal_variation:Vec<MoveData>=Vec::new();
    let mut best_eval = -INFINITY;
    let killer_moves = [[MoveData::defualt(); 2]; 256];
    let mut cap_hist:CaptureHistoryTable=[[[0;12];64];12];
    let mut alpha = -INFINITY;
    let mut beta = INFINITY;
    let mut refs=SearchRefs::new_depth_search(killer_moves,history_table,cap_hist);
    while current_depth<= input.depth {
     {



    let eval = search_internal(&mut board, current_depth as i32, 0, alpha, beta,  &mut principal_variation, &mut refs);
        best_eval = eval;
         if eval<=alpha || eval>=beta {
             alpha=-INFINITY;
             beta=INFINITY;
             continue
         }
         else {
             alpha=eval-VAL_WINDOW;
                beta=eval+VAL_WINDOW;
             current_depth+=1
         }
         
     }
    }

    SearchOutput {
        nodes_evaluated: refs.get_nodes_evaluated(),
        principal_variation,
        eval: best_eval,
        depth: (current_depth-1) as i32
    }
}






pub fn timed_search(board: &mut Board, time_limit: Duration, increment: Duration) -> SearchOutput {
    let nodes_evaluated = 0;
    let mut pv = Vec::new();
    let killer_moves = [[MoveData::defualt(); 2]; 256];
    let history_table = [[[0; 64]; 64]; 2];
    let mut curr_eval =0;
    let best_move = MoveData::defualt();
    let move_time = time_limit/40 + increment / 2;
    let max_depth=256;
    let mut depth =1;
    let mut alpha = -INFINITY;
    let mut beta = INFINITY;
    let   start_time = Instant::now();
    let cap_hist:CaptureHistoryTable=[[[0;12];64];12];
    
    let mut refs = SearchRefs::new_timed_search(killer_moves, &start_time, &move_time, history_table,cap_hist);
    while depth<= max_depth {
       

        if start_time.elapsed()*2 > move_time {
            break;
        }


        let  curr_depth_eval = timed_search_internal(
            board,
            depth,
            0,
            alpha,
            beta,

            &mut pv,
            &mut refs
        );
       
        if start_time.elapsed() >= move_time {
            break;
        }
        curr_eval = curr_depth_eval;
        if curr_eval<=alpha || curr_eval>=beta {
            alpha=-INFINITY;
            beta=INFINITY;
            continue
        }
        else {
            alpha=curr_eval-VAL_WINDOW;
            beta=curr_eval+VAL_WINDOW;
            depth+=1
        }





    }

    SearchOutput {
        nodes_evaluated: refs.get_nodes_evaluated(),
        principal_variation:pv,
        eval: curr_eval,
        depth:depth-1
    }
}




fn search_common(
    board: &mut Board,
    depth: i32,
    ply: i32,
    mut alpha: i32,
    beta: i32,
    pv: &mut Vec<MoveData>,
     refs:&mut SearchRefs


) -> i32 {
    // Stop search if time has elapsed
   if refs.is_time_done(){
            return 0;
        }


    if depth == 0 {
        return quiescence_search(board, alpha, beta, refs);
    }
    refs.increment_nodes_evaluated();
    let is_in_check=board.is_check;
    if let Some(entry) = TRANSPOSITION_TABLE
        .lock()
        .unwrap()
        .retrieve(board.game_state.zobrist_hash, depth as u8, alpha, beta)
    {
        if ply == 0 {
            pv.clear();
            pv.push(entry.best_move);
        }
        return entry.eval;
    }

    let mut move_list = generate_moves(board, false);
    if move_list.len()==0{
        return if board.is_check{
            -MATE_VALUE+ply
        }
        else{
            0
        }
    }
    if board.is_board_draw(){
        return 0;
    }
    
    let curr_eval= eval(board);
    if board.is_check{
        refs.disable_eval_ply(ply);
    }
    else { 
        refs.set_eval_ply(ply,curr_eval);
    }
    let improving = if ply >= 2 {
        if let (Some(this_depth_eval), Some(two_moves_ago_eval)) = (
            refs.get_eval_ply(ply),
            refs.get_eval_ply(ply - 2),
        ) {
            this_depth_eval > two_moves_ago_eval
        } else {
            false
        }
    } else if ply >= 4 {
        if let (Some(second_depth_eval), Some(four_moves_ago_eval)) = (
            refs.get_eval_ply(ply - 2),
            refs.get_eval_ply(ply - 4),
        ) {
            second_depth_eval > four_moves_ago_eval
        } else {
            false
        }
    } else {
        false
    };
    
    let mut should_extend =false;
    if board.is_check && refs.is_extension_allowed(){
        should_extend=true;
        refs.increment_extensions();
    }
    else {
        refs.reset_extensions();
    }
    if depth >= 3 && !board.is_check && !board.detect_pawns_only(board.turn)  {
        board.make_null_move();
        let r= depth/3;
        let new_depth=(depth-(r+2)).max(0);
        let null_move_score = -search_common(
            board,
            new_depth,
            ply + 1,
            -beta,
            -beta + 1,
            pv,
            refs

        );
        board.unmake_null_move();
        if null_move_score>= beta {
            
            return null_move_score;
        }
    }

    let mut best_move = MoveData::defualt();
    let mut entry_type = EntryType::UpperBound;
    let mut is_pvs = false;



    let tt_move = TRANSPOSITION_TABLE
        .lock()
        .unwrap()
        .get_TT_move(board.game_state.zobrist_hash)
        .unwrap_or(MoveData::defualt());
    let depth_actual= if tt_move==MoveData::defualt() && depth>5
    {
        depth-2
    }
    else {
        depth
        
    };
    

    if is_allowed_reverse_futility_pruning(depth as u8, beta, curr_eval, board,improving) {
        
        return curr_eval;
    }

    let mut move_score = get_moves_score(
        &move_list,
        &tt_move,
        ply as usize,
        board,
        &*refs,

        board.turn,
    );

    let mut quiet_moves=0;
    for i in 0..move_list.len() {
        // Stop search if time has elapsed
        if refs.is_time_done(){
            return 0;
        }



        pick_move(&mut move_list, i as u8, &mut move_score);

        let mut curr_move = move_list.get_move(i);
        if *curr_move!=tt_move && curr_move.is_capture() && static_exchange_evaluation(board, curr_move.get_capture_square().unwrap() as i32,curr_move.get_captured_piece().unwrap(),curr_move.piece_to_move, curr_move.from as i32) < 0 {
            move_score[i]= -BASE_CAPTURE+MVV_LVA[curr_move.get_captured_piece().unwrap().piece_type as usize][curr_move.piece_to_move.piece_type as usize] as i32;
            pick_move(&mut move_list, i as u8, &mut move_score);
            curr_move= move_list.get_move(i);       
        }

        if board.is_quiet_move(curr_move){
            
            if should_movecount_based_pruning(board, *curr_move, depth as u32, quiet_moves ,alpha,improving) && is_pvs{
                continue;
            }
            quiet_moves+=1;
        }

        let mut node_pv: Vec<MoveData> = Vec::new();
        board.make_move(curr_move);


       if is_allowed_futility_pruning(depth as u8, alpha,curr_eval, curr_move, board) && is_pvs && !is_in_check{
            board.unmake_move(curr_move);
            break;
        }



        let mut score_mv = 0;
    let extension_adding=if should_extend {1} else { 0 };
        if depth >= 3 && is_pvs {
            let new_depth =reduce_depth(board, curr_move, depth_actual as f64, i as f64,improving) as i32;
            score_mv = -search_common(
                board,
                new_depth,
                ply + 1,
                -alpha - 1,
                -alpha,
                &mut node_pv,
                refs

            );

            if score_mv > alpha {
                score_mv = -search_common(
                    board,
                    depth_actual - 1+ extension_adding,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    &mut node_pv,
                    refs

                );

                if score_mv > alpha {
                    score_mv = -search_common(
                        board,
                        depth_actual - 1+ extension_adding,
                        ply + 1,
                        -beta,
                        -alpha,

                        &mut node_pv,
                       refs
                    );
                }
            }
        } else {
            score_mv = -search_common(
                board,
                depth_actual - 1,
                ply + 1,
                -beta,
                -alpha,

                &mut node_pv,
                refs
            );

        }


        board.unmake_move(curr_move);


        if score_mv >= beta  {
            entry_type = EntryType::LowerBound;
            best_move = *curr_move;

            TRANSPOSITION_TABLE
                .lock()
                .unwrap()
                .store(board.game_state.zobrist_hash, depth as u8, score_mv, entry_type, best_move);

            if curr_move.is_capture(){
                refs.add_capture_history(curr_move, depth);
            }
            if !curr_move.is_capture() && !curr_move.is_promotion() && !board.is_move_check(curr_move) {
                refs.store_killers(*curr_move, ply as usize);
                refs.add_history(board.turn, *curr_move, depth);
            }
            
            return score_mv;
        }
        
        if curr_move.is_capture(){
            refs.reduce_capture_history(curr_move, depth);
        }
        
        if score_mv > alpha {
          
            alpha = score_mv;
            best_move = *curr_move;
            entry_type = EntryType::Exact;

            // Update PV
            pv.clear();
            pv.push(*curr_move);
            pv.append(&mut node_pv);
        }
        is_pvs = true;
    }

    TRANSPOSITION_TABLE
        .lock()
        .unwrap()
        .store(board.game_state.zobrist_hash, depth as u8, alpha, entry_type, best_move);

    alpha
}

fn search_internal(
    board: &mut Board,
    depth: i32,
    ply: i32,
    alpha: i32,
    beta: i32,
    pv: &mut Vec<MoveData>,
    refs: &mut SearchRefs,
) -> i32 {

    search_common(
        board,
        depth,
        ply,
        alpha,
        beta,
        pv,
        refs

    )
}

fn timed_search_internal(
    board: &mut Board,
    depth: i32,
    ply: i32,
    alpha: i32,
    beta: i32,
    pv: &mut Vec<MoveData>,
    refs: &mut SearchRefs

) -> i32 {
    search_common(
        board,
        depth,
        ply,
        alpha,
        beta,

        pv,
        refs

    )
}
