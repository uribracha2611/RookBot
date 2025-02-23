use std::cmp::PartialEq;
use crate::board::board::Board;
use crate::board::piece::PieceColor;
use crate::movegen::generate::generate_moves;
use crate::movegen::movedata::MoveData;
use crate::movegen::movelist::MoveList;
use crate::search::constants::{FUTILITY_MARGIN, FUTILITY_MARGIN_2, INFINITY, MATE_VALUE, VAL_WINDOW};
use crate::search::move_ordering::{get_capture_score, get_move_score, get_moves_score, store_killers, KillerMoves, BASE_KILLER};
use crate::search::transposition_table::{Entry, EntryType, TRANSPOSITION_TABLE};
use crate::search::types::{ChosenMove, SearchInput, SearchOutput};

use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::search::late_move_reduction::reduce_depth;

pub fn quiescence_search(
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    nodes_evaluated: &mut i32,
    start_time: Option<Instant>,
    time_limit: Option<Duration>,
) -> i32 {
    // Check if time has exceeded
    if let (Some(start_time), Some(time_limit)) = (start_time, time_limit) {
        if start_time.elapsed() >= time_limit {
            return alpha; // Return the best evaluation found so far
        }
    }

    let stand_pat = eval(board);
    let mut best_val = stand_pat;

    // Alpha-Beta pruning
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let mut moves = generate_moves(board, true);
    let TT_Move = TRANSPOSITION_TABLE.lock().unwrap().get_TT_move(board.game_state.zobrist_hash).unwrap_or(MoveData::defualt());
    let scores = get_capture_score(moves, TT_Move);

    // Iterate through the moves
    for i in 0..moves.len() {
        *nodes_evaluated += 1;

        // Pick the move to search next
        pick_move(&mut moves, i as u8, &scores);
        let mv = moves.get_move(i);

        // Check time again before making a move
        if let (Some(start_time), Some(time_limit)) = (start_time, time_limit) {
            if start_time.elapsed() >= time_limit {
                return alpha; // Return the best evaluation if time is up
            }
        }

        // Make the move and perform recursive quiescence search
        board.make_move(mv);
        let score = -quiescence_search(board, -beta, -alpha, nodes_evaluated, start_time, time_limit);
        board.unmake_move(mv);

        // Apply pruning if necessary
        if score >= beta {
            return beta;
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
pub fn pick_move(ml: &mut MoveList, start_index: u8,scores: &Vec<u32>) {

    for i in (start_index + 1)..(ml.len() as u8) {
        if scores[i as usize] > scores[start_index as usize] {
            ml.swap(start_index as usize, i as usize);
        }
    }
}

pub fn search(mut board: &mut Board, input: &SearchInput) -> SearchOutput {
    let mut current_depth=1;
    let mut nodes_evaluated = 0;
    let mut  history_table=[[[0;64];64];2];
    let mut principal_variation:Vec<MoveData>=Vec::new();
    let mut best_eval = -INFINITY;
    let mut  killer_moves = [[MoveData::defualt(); 2]; 256];
    let mut alpha = -INFINITY;
    let mut beta = INFINITY;
    while current_depth<= input.depth {
     {



    let eval = search_internal(&mut board, current_depth as i32, 0, alpha, beta, &mut nodes_evaluated, &mut principal_variation, &mut killer_moves, &mut history_table);
        best_eval = eval;
         if (eval<=alpha || eval>=beta){
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
        nodes_evaluated,
        principal_variation,
        eval: best_eval,
        depth: (current_depth-1) as i32
    }
}






pub fn timed_search(board: &mut Board, time_limit: Duration, increment: Duration) -> SearchOutput {
    let mut nodes_evaluated = 0;
    let mut pv = Vec::new();
    let mut killer_moves = [[MoveData::defualt(); 2]; 256];
    let mut history_table = [[[0; 64]; 64]; 2];
    let start_time = Instant::now();
    let mut curr_eval =0;
    let mut best_move = MoveData::defualt();
    let move_time = time_limit.mul_f64(0.0225) + increment / 2;
    let max_depth=256;
    let mut depth =1;
    while depth<= max_depth {
       

        if start_time.elapsed() >= move_time {
            break;
        }

   

        let  curr_depth_eval = timed_search_internal(
            board,
            depth as i32,
            0,
            -INFINITY,
            INFINITY,
            &mut nodes_evaluated,
            &mut pv,
            &mut killer_moves,
            &mut history_table,
            start_time,
            move_time,
        );
       
        if start_time.elapsed() >= move_time {
            break;
        }
        curr_eval = curr_depth_eval;
            if let Some(&best) = pv.first() {
            best_move = best;
        }
        depth+=1;


    }

    SearchOutput {
        nodes_evaluated,
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
    nodes_evaluated: &mut i32,
    pv: &mut Vec<MoveData>,
    killer_moves: &mut KillerMoves,
    history_table: &mut [[[u32; 64]; 64]; 2],
    start_time: Option<Instant>,
    time_limit: Option<Duration>,
) -> i32 {
    // Stop search if time has elapsed
    if let (Some(start_time), Some(time_limit)) = (start_time, time_limit) {
        if start_time.elapsed() >= time_limit {
            return 0;
        }
    }

    if depth == 0 {
        return quiescence_search(board, alpha, beta, nodes_evaluated, start_time, time_limit);
    }

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
    if depth >= 3 && !board.is_check {
        board.make_null_move();
        let null_move_score = -search_common(
            board,
            depth - 3,
            ply + 1,
            -beta,
            -beta + 1,
            nodes_evaluated,
            pv,
            killer_moves,
            history_table,
            start_time,
            time_limit,
        );
        board.unmake_null_move();
        if null_move_score >= beta {
            return beta;
        }
    }

    let mut best_move = MoveData::defualt();
    let mut entry_type = EntryType::UpperBound;
    let mut is_pvs = false;

    let curr_eval = if depth == 1 || depth == 2 {
        eval(board)
    } else {
        INFINITY
    };

    let do_futile_prune = !board.is_check
        && ((curr_eval < alpha - FUTILITY_MARGIN && depth == 1)
        || (curr_eval < alpha - FUTILITY_MARGIN_2 && depth == 2));

    let tt_move = TRANSPOSITION_TABLE
        .lock()
        .unwrap()
        .get_TT_move(board.game_state.zobrist_hash)
        .unwrap_or(MoveData::defualt());

    let move_score = get_moves_score(
        &move_list,
        &tt_move,
        *killer_moves,
        ply as usize,
        *history_table,
        board.turn,
    );

    for i in 0..move_list.len() {
        // Stop search if time has elapsed
        if let (Some(start_time), Some(time_limit)) = (start_time, time_limit) {
            if start_time.elapsed() >= time_limit {
                break;
            }
        }

        let mut node_pv: Vec<MoveData> = Vec::new();

        pick_move(&mut move_list, i as u8, &move_score);
        let curr_move = move_list.get_move(i);

        board.make_move(curr_move);
        if !board.is_check && !curr_move.is_capture() && do_futile_prune {
            board.unmake_move(curr_move);
            continue;
        }

        *nodes_evaluated += 1;
        let mut score_mv = 0;

        if depth >= 3 && is_pvs {
            let new_depth = reduce_depth(board, curr_move, depth as f64, i as f64) as i32;
            score_mv = -search_common(
                board,
                new_depth,
                ply + 1,
                -alpha - 1,
                -alpha,
                nodes_evaluated,
                &mut node_pv,
                killer_moves,
                history_table,
                start_time,
                time_limit,
            );

            if score_mv > alpha {
                score_mv = -search_common(
                    board,
                    depth - 1,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    nodes_evaluated,
                    &mut node_pv,
                    killer_moves,
                    history_table,
                    start_time,
                    time_limit,
                );

                if score_mv > alpha {
                    score_mv = -search_common(
                        board,
                        depth - 1,
                        ply + 1,
                        -beta,
                        -alpha,
                        nodes_evaluated,
                        &mut node_pv,
                        killer_moves,
                        history_table,
                        start_time,
                        time_limit,
                    );
                }
            }
        } else {
            score_mv = -search_common(
                board,
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
                nodes_evaluated,
                &mut node_pv,
                killer_moves,
                history_table,
                start_time,
                time_limit,
            );
        }

        board.unmake_move(curr_move);

        if score_mv >= beta {
            entry_type = EntryType::LowerBound;
            best_move = *curr_move;

            TRANSPOSITION_TABLE
                .lock()
                .unwrap()
                .store(board.game_state.zobrist_hash, depth as u8, score_mv, entry_type, best_move);

            if !curr_move.is_capture() && !curr_move.is_promotion() {
                store_killers(killer_moves, *curr_move, ply as usize);
            }
            return beta;
        }

        if score_mv > alpha {
            is_pvs = true;
            alpha = score_mv;
            best_move = *curr_move;
            entry_type = EntryType::Exact;

            // Update PV
            pv.clear();
            pv.push(*curr_move);
            pv.append(&mut node_pv);
        }
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
    mut alpha: i32,
    beta: i32,
    nodes_evaluated: &mut i32,
    pv: &mut Vec<MoveData>,
    killer_moves: &mut KillerMoves,
    history_table: &mut [[[u32; 64]; 64]; 2],
) -> i32 {
    search_common(
        board,
        depth,
        ply,
        alpha,
        beta,
        nodes_evaluated,
        pv,
        killer_moves,
        history_table,
        None,
        None,
    )
}

fn timed_search_internal(
    board: &mut Board,
    depth: i32,
    ply: i32,
    mut alpha: i32,
    beta: i32,
    nodes_evaluated: &mut i32,
    pv: &mut Vec<MoveData>,
    killer_moves: &mut KillerMoves,
    history_table: &mut [[[u32; 64]; 64]; 2],
    start_time: Instant,
    time_limit: Duration,
) -> i32 {
    search_common(
        board,
        depth,
        ply,
        alpha,
        beta,
        nodes_evaluated,
        pv,
        killer_moves,
        history_table,
        Some(start_time),
        Some(time_limit),
    )
}
