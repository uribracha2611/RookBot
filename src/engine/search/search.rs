use std::time::{Duration, Instant};

use crate::engine::board::board::Board;
use crate::engine::board::piece::PieceColor;
use crate::engine::board::see::static_exchange_evaluation;
use crate::engine::movegen::generate::generate_moves;
use crate::engine::movegen::movedata::MoveData;
use crate::engine::movegen::movelist::MoveList;
use crate::engine::search::constants::{
    INFINITY, MATE_VALUE, RAZOR_DEPTH, RAZOR_MARGIN, VAL_WINDOW,
};
use crate::engine::search::functions::{
    is_allowed_futility_pruning, is_allowed_reverse_futility_pruning, is_improving,
};
use crate::engine::search::late_move_reduction::{reduce_depth, should_movecount_based_pruning};
use crate::engine::search::move_ordering::{
    get_capture_score, get_move_score, get_moves_score, BASE_CAPTURE, MVV_LVA,
};
use crate::engine::search::transposition_table::EntryType::UpperBound;
use crate::engine::search::transposition_table::{EntryType, TranspositionTable};
use crate::engine::search::types::{CaptureHistoryTable, SearchInput, SearchOutput, SearchRefs};

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

    let mut scores = get_capture_score(board, moves, tt_move, refs);

    // Iterate through the moves
    for i in 0..moves.len() {
        // Pick the move to search next
        pick_move(&mut moves, i as u8, &mut scores);
        let mv = moves.get_move(i);

        if *mv != tt_move
            && static_exchange_evaluation(
                board,
                mv.to as i32,
                mv.get_captured_piece().unwrap(),
                mv.piece_to_move,
                mv.from as i32,
            ) < 0
        {
            continue;
        }

        // Check time again before making a move

        if refs.is_time_done() {
            return 0;
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

pub fn eval(board: &Board) -> i32 {
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
pub fn pick_move(ml: &mut MoveList, start_index: u8, scores: &mut Vec<i32>) {
    for i in (start_index + 1)..(ml.len() as u8) {
        if scores[i as usize] > scores[start_index as usize] {
            ml.swap(start_index as usize, i as usize);
            scores.swap(start_index as usize, i as usize);
        }
    }
}

pub fn search(
    mut board: &mut Board,
    input: &mut SearchInput,
    tt_table: &mut TranspositionTable,
) -> SearchOutput {
    let mut current_depth = 1;
    let history_table = [[[0; 64]; 64]; 2];
    let mut principal_variation: Vec<MoveData> = Vec::new();
    let mut best_eval = -INFINITY;
    let killer_moves = [[MoveData::default(); 2]; 256];
    let mut cap_hist: CaptureHistoryTable = [[[0; 12]; 64]; 12];
    let mut alpha = -INFINITY;
    let mut beta = INFINITY;
    let mut refs = SearchRefs::new_depth_search(killer_moves, history_table, cap_hist, tt_table);
    while current_depth <= input.depth {
        let eval = search_common(
            &mut board,
            current_depth as i32,
            0,
            alpha,
            beta,
            &mut principal_variation,
            &mut refs,
        );
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

pub fn timed_search(
    board: &mut Board,
    time_limit: Duration,
    increment: Duration,
    is_move_time: bool,
    tt_table: &mut TranspositionTable,
) -> SearchOutput {
    let mut pv = Vec::new();
    let killer_moves = [[MoveData::default(); 2]; 256];
    let history_table = [[[0; 64]; 64]; 2];
    let mut curr_eval = 0;
    let move_time = if is_move_time {
        time_limit
    } else {
        time_limit / 40 + increment / 2
    };
    let max_depth = 256;
    let mut depth = 1;
    let mut alpha = -INFINITY;
    let mut beta = INFINITY;
    let start_time = Instant::now();
    let cap_hist: CaptureHistoryTable = [[[0; 12]; 64]; 12];

    let mut refs = SearchRefs::new_timed_search(
        killer_moves,
        &start_time,
        &move_time,
        history_table,
        cap_hist,
        tt_table,
    );
    while depth <= max_depth {
        if start_time.elapsed() * 2 > move_time {
            break;
        }

        let curr_depth_eval = search_common(board, depth, 0, alpha, beta, &mut pv, &mut refs);

        if start_time.elapsed() >= move_time {
            break;
        }
        curr_eval = curr_depth_eval;
        depth += 1;
    }

    SearchOutput {
        nodes_evaluated: refs.get_nodes_evaluated(),
        principal_variation: pv,
        eval: curr_eval,
        depth: depth - 1,
    }
}

fn search_common(
    board: &mut Board,
    depth: i32,
    ply: i32,
    mut alpha: i32,
    beta: i32,
    pv: &mut Vec<MoveData>,
    refs: &mut SearchRefs,
) -> i32 {
    // Stop search if time has elapsed
    if refs.is_time_done() {
        return 0;
    }

    refs.increment_nodes_evaluated();
    if depth <= 0 {
        return quiescence_search(board, alpha, beta, refs);
    }
    let mut move_list = generate_moves(board, false);
    let is_in_check = board.is_check;
    if move_list.len() == 0 {
        return if board.is_check { -MATE_VALUE + ply } else { 0 };
    }
    if board.is_board_draw() {
        return 0;
    }
    let mut tt_move = MoveData::default();

    if let Some(entry) = refs
        .get_transposition_table()
        .retrieve(board.game_state.zobrist_hash)
    {
        tt_move = entry.best_move;
        if ply > 0 && entry.depth >= depth as u8 {
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
    }

    let curr_eval = eval(board);
    if board.is_check {
        refs.disable_eval_ply(ply);
    } else {
        refs.set_eval_ply(ply, curr_eval);
    }
    let improving = is_improving(refs, ply);
    let mut should_extend = false;
    if board.is_check && refs.is_extension_allowed() {
        should_extend = true;
        refs.increment_extensions();
    } else {
        refs.reset_extensions();
    }
    if depth <= RAZOR_DEPTH && curr_eval + RAZOR_MARGIN < beta {
        let value = quiescence_search(board, alpha, beta, refs);
        if (value < beta) {
            return value;
        }
    }
    if is_allowed_reverse_futility_pruning(depth as u8, beta, curr_eval, board, improving) {
        return curr_eval;
    }
    if !board.is_check && depth >= 3 {
        let r = if depth > 10 {
            5
        } else if depth > 6 {
            4
        } else {
            3
        };
        board.make_null_move();
        let null_move_score =
            -search_common(board, depth - 1 - r, ply + 1, -beta, -beta + 1, pv, refs);
        board.unmake_null_move();
        if null_move_score >= beta {
            return null_move_score;
        }
    }

    let depth_actual = if tt_move == MoveData::default() && depth > 5 {
        depth - 2
    } else {
        depth
    };

    let mut move_score =
        get_moves_score(&move_list, ply as usize, board, tt_move, &*refs, board.turn);
    let mut best_move = MoveData::default();
    let mut entry_type = EntryType::UpperBound;
    let mut quiet_moves_count = 0;

    let mut quiet_moves: Vec<MoveData> = Vec::with_capacity(move_list.len());
    let mut is_pvs = false;
    for i in 0..move_list.len() {
        // Stop search if time has elapsed
        if refs.is_time_done() {
            return 0;
        }

        let mut is_quiet_move = false;
        pick_move(&mut move_list, i as u8, &mut move_score);

        let mut curr_move = move_list.get_move(i);
        let mut see_val = 0;
        while *curr_move != tt_move && curr_move.is_capture() {
            see_val = static_exchange_evaluation(
                board,
                curr_move.to as i32,
                curr_move.get_captured_piece().unwrap(),
                curr_move.piece_to_move,
                curr_move.from as i32,
            );
            if see_val >= 0 {
                break;
            }

            let old_move = *curr_move;
            move_score[i] = -BASE_CAPTURE
                + ((curr_move.get_captured_piece().unwrap().get_value() * 10)
                    - curr_move.piece_to_move.get_value());
            pick_move(&mut move_list, i as u8, &mut move_score);
            curr_move = move_list.get_move(i);
            if *curr_move == old_move {
                break;
            };
        }

        if curr_move.is_capture()
            && *curr_move != tt_move
            && see_val < -25 * depth_actual * depth_actual
            && !board.is_check
            && alpha > -MATE_VALUE + 500
            && i > 1
        {
            continue;
        }

        if board.is_quiet_move(curr_move) {
            is_quiet_move = true;
            if should_movecount_based_pruning(
                board,
                *curr_move,
                depth_actual as u32,
                quiet_moves_count,
                alpha,
                improving,
            ) && is_pvs
            {
                continue;
            }
            quiet_moves_count += 1;
        }

        let mut node_pv: Vec<MoveData> = Vec::new();
        board.make_move(curr_move);

        if is_allowed_futility_pruning(depth_actual as u8, alpha, curr_eval, curr_move, board)
            && is_pvs
            && !is_in_check
        {
            board.unmake_move(curr_move);
            break;
        }

        refs.set_move_ply(ply, *curr_move);
        let extension_adding = if (should_extend) { 1 } else { 0 };
        let mut score_mv = 0;
        if is_pvs && depth_actual >= 3 {
            let new_depth =
                reduce_depth(board, curr_move, depth_actual as f32, i as f32, improving) as i32;
            score_mv = -search_common(
                board,
                new_depth,
                ply + 1,
                -alpha - 1,
                -alpha,
                &mut node_pv,
                refs,
            );
            if score_mv > alpha && score_mv < beta {
                score_mv = -search_common(
                    board,
                    (depth_actual - 1) + extension_adding,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    &mut node_pv,
                    refs,
                );
                if score_mv > alpha && score_mv < beta {
                    score_mv = -search_common(
                        board,
                        (depth_actual - 1) + extension_adding,
                        ply + 1,
                        -beta,
                        -alpha,
                        &mut node_pv,
                        refs,
                    );
                }
            }
        } else {
            score_mv = -search_common(
                board,
                (depth_actual - 1) + extension_adding,
                ply + 1,
                -beta,
                -alpha,
                &mut node_pv,
                refs,
            );
        }

        board.unmake_move(curr_move);

        if score_mv >= beta {
            entry_type = EntryType::LowerBound;
            best_move = *curr_move;

            refs.table.store(
                board.game_state.zobrist_hash,
                depth_actual as u8,
                score_mv,
                entry_type,
                best_move,
            );

            if !curr_move.is_capture() {
                refs.store_killers(*curr_move, ply as usize);

                refs.add_history(board.turn, *curr_move, depth_actual, false);
                refs.increament_cont_hist(depth_actual, ply, curr_move);
            }
            for quiet_move in quiet_moves {
                refs.decreament_cont_hist(depth_actual, ply, &quiet_move);
                refs.add_history(board.turn, quiet_move, depth_actual, true);
            }

            return score_mv;
        }

        if is_quiet_move {
            quiet_moves.push(*curr_move);
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

    refs.get_transposition_table().store(
        board.game_state.zobrist_hash,
        depth_actual as u8,
        alpha,
        entry_type,
        best_move,
    );

    alpha
}
