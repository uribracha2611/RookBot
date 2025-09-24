use crate::engine::board::board::Board;
use crate::engine::board::piece::PieceColor;
use crate::engine::board::see::static_exchange_evaluation;
use crate::engine::movegen::generate::generate_moves;
use crate::engine::movegen::movedata::MoveData;
use crate::engine::movegen::movelist::MoveList;
use crate::engine::search::constants::{INFINITY, MATE_VALUE};
use crate::engine::search::move_ordering::{get_capture_score, get_moves_score};
use crate::engine::search::transposition_table::EntryType::UpperBound;
use crate::engine::search::transposition_table::{EntryType, TranspositionTable};
use crate::engine::search::types::{SearchInput, SearchOutput, SearchRefs};
use std::time::Duration;

pub fn quiescence_search(
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    refs: &mut SearchRefs,
) -> i32 {
    debug_assert!(alpha < beta);
    debug_assert!(alpha >= -INFINITY);
    debug_assert!(beta <= INFINITY);
    // Check if time has exceeded
    if refs.is_time_done() || refs.is_nodes_exceeded() {
        return 0;
    }


    let stand_pat = eval(board);
    let mut best_val = stand_pat;

    // Alpha-Beta pruning
    if stand_pat >= beta {
        return stand_pat;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }
    let mut tt_move = MoveData::default();
    if let Some(entry) = refs
        .get_transposition_table()
        .retrieve(board.game_state.zobrist_hash)
    {
        tt_move = entry.best_move;

        match entry.entry_type {
            EntryType::Exact => return entry.eval,
            EntryType::LowerBound => {
                if entry.eval >= beta {
                    return entry.eval;
                }
            }
            UpperBound => {
                if entry.eval <= alpha {
                    return entry.eval;
                }
            }
        }
    }

    let mut moves = generate_moves(board, true);
    let mut scores = get_capture_score(moves, tt_move);

    // Iterate through the moves
    for i in 0..moves.len() {
        if refs.is_nodes_exceeded() {
            return 0;
        }
        // Pick the move to search next
        pick_move(&mut moves, i, &mut scores);
        let mv = moves.get_move(i);


        if *mv != tt_move && static_exchange_evaluation(board, mv) < 0 {
            continue;
        }

        // Check time again before making a move

        if refs.is_time_done() {
            return 0;
        }

        refs.increment_nodes_evaluated();
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

pub fn eval(board: &Board) -> i32 {
    debug_assert!(board.calc_eval() == (board.psqt_white.get_middle_game(), board.psqt_white.get_end_game(), board.psqt_black.get_middle_game(), board.psqt_black.get_end_game()));
    debug_assert!(board.calc_gamephase() == board.game_phase);
    if board.is_insufficient_material() {
        return 0;
    }
    let mg_phase = board.game_phase.min(24);
    let eg_phase = 24 - mg_phase;
    let mg_score = board.psqt_white.get_middle_game() - board.psqt_black.get_middle_game();
    let eg_score = board.psqt_white.get_end_game() - board.psqt_black.get_end_game();
    let score = (mg_score * mg_phase + eg_score * eg_phase) / 24;

    if board.turn == PieceColor::WHITE {
        score
    } else {
        -score
    }
}
pub fn pick_move(ml: &mut MoveList, start_index: usize, scores: &mut Vec<i32>) {
    for i in (start_index + 1)..(ml.len()) {
        if scores[i as usize] > scores[start_index as usize] {
            ml.swap(start_index as usize, i as usize);
            scores.swap(start_index as usize, i as usize);
        }
    }
}

pub fn search(
    board: &mut Board,
    input: &mut SearchInput,
    tt_table: &mut TranspositionTable,
) -> SearchOutput {
    let mut current_depth = 1;

    let mut principal_variation: Vec<MoveData> = Vec::new();
    let mut best_eval = -INFINITY;
    let is_depth_search = input.depth.is_some();
    let is_move_count_search = input.node_count.is_some();
    let max_depth = input.depth.unwrap_or(64);
    let node_count = input.node_count.unwrap_or(0);
    let move_time = input.move_time.unwrap_or(Duration::from_millis(0));
    let alpha = -INFINITY;
    let beta = INFINITY;
    let mut refs = if is_depth_search { SearchRefs::new_depth_search(tt_table) } else if is_move_count_search {
        SearchRefs::new_node_search(node_count, tt_table)
    } else { SearchRefs::new_timed_search(&move_time, tt_table) };
    while current_depth <= max_depth {
        if refs.is_time_elapsed_iterative_search() {
            break;
        }
        let old_pv = principal_variation.clone();
        let eval = search_common(
            board,
            current_depth as i32,
            0,
            alpha,
            beta,
            &mut principal_variation,
            &mut refs,
        );
        if refs.is_time_done() || refs.is_nodes_exceeded() {
            principal_variation = old_pv;
            break;
        }

        best_eval = eval;
        current_depth += 1;
    }

    SearchOutput {
        nodes_evaluated: refs.get_nodes_evaluated(),
        principal_variation,
        eval: best_eval,
        depth: (current_depth - 1) as i32,
    }
}


fn search_common(
    board: &mut Board,
    mut depth: i32,
    ply: i32,
    mut alpha: i32,
    beta: i32,
    pv: &mut Vec<MoveData>,
    refs: &mut SearchRefs,
) -> i32 {
    debug_assert!(ply >= 0);
    debug_assert!(alpha < beta);
    debug_assert!(alpha >= -INFINITY);
    debug_assert!(beta <= INFINITY);
    // Stop search if time has elapsed
    if refs.is_time_done() || refs.is_nodes_exceeded() {
        return 0;
    }
    let mut best_score = -INFINITY;


    if depth <= 0 {
        return eval(board);
    }
    let mut move_list = generate_moves(board, false);

    if move_list.len() == 0 {
        return if board.is_check { -MATE_VALUE + ply } else { 0 };
    }
    if board.is_board_draw() {
        return 0;
    }


    let mut best_move = MoveData::default();

    let mut scores = get_moves_score(&move_list);
    for i in 0..move_list.len() {
        if refs.is_nodes_exceeded() {
            return 0;
        }
        pick_move(&mut move_list, i, &mut scores);
        let curr_move = move_list.get_move(i);


        refs.increment_nodes_evaluated();
        let mut node_pv: Vec<MoveData> = Vec::new();
        board.make_move(curr_move);


        let score_mv = -search_common(
            board,
            depth - 1,
            ply + 1,
            -beta,
            -alpha,
            &mut node_pv,
            refs,
        );


        board.unmake_move(curr_move);

        if score_mv >= beta {
            best_move = *curr_move;

            return score_mv;
        }
        if score_mv > best_score {
            best_score = score_mv;
            if score_mv > alpha {
                alpha = score_mv;
                best_move = *curr_move;

                // Update PV
                pv.clear();
                pv.push(*curr_move);
                pv.append(&mut node_pv);
            }
        }
    }


    best_score
}
