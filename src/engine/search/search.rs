use crate::engine::board::board::Board;
use crate::engine::board::piece::PieceColor;
use crate::engine::board::piece::PieceType::KING;
use crate::engine::board::see::static_exchange_evaluation;
use crate::engine::movegen::generate::{generate_moves, update_check};
use crate::engine::movegen::movedata::MoveData;
use crate::engine::movegen::movelist::MoveList;
use crate::engine::search::clock::TimeManager;
use crate::engine::search::constants::{
    INFINITY, MATE_VALUE, RAZOR_DEPTH, RAZOR_MARGIN, VAL_WINDOW,
};
use crate::engine::search::functions::{is_allowed_reverse_futility_pruning, is_improving};
use crate::engine::search::late_move_reduction::{reduce_depth, should_movecount_based_pruning};
use crate::engine::search::move_ordering::{
    BASE_CAPTURE, capture_formula, get_capture_score, get_moves_score,
};
use crate::engine::search::nnue::NNUE_NETWORK;
use crate::engine::search::transposition_table::EntryType::UpperBound;
use crate::engine::search::transposition_table::{self, EntryType, TranspositionTable};
use crate::engine::search::types::{SearchInput, SearchOutput, SearchRefs};
use num_traits::real::Real;
use std::time::Duration;

pub fn quiescence_search(
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    refs: &mut SearchRefs,
    time_manager: &TimeManager,
) -> i32 {
    debug_assert!(alpha < beta);
    debug_assert!(alpha >= -INFINITY);
    debug_assert!(beta <= INFINITY);
    // Check if time has exceeded
    if time_manager.check_if_done(refs.get_nodes_evaluated()) {
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
    let mut tt_move = None;
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
    update_check(board);
    let mut moves = generate_moves(board, true);
    let mut scores = get_capture_score(moves, tt_move, board);

    // Iterate through the moves
    for i in 0..moves.len() {
        if time_manager.check_if_only_nodes_done(refs.get_nodes_evaluated()) {
            return 0;
        }
        // Pick the move to search next
        pick_move(&mut moves, i as u8, &mut scores);
        let mv = moves.get_move(i);

        if Some(mv) != tt_move && static_exchange_evaluation(board, mv) < 0 {
            continue;
        }
        refs.table.prefetch(board.calc_hash_after_move(&mv));
        // Check time again before making a move

        if time_manager.check_if_time_only_done() {
            return 0;
        }

        refs.increment_nodes_evaluated();
        // Make the move and perform recursive quiescence search
        board.make_move(mv);
        let score = -quiescence_search(board, -beta, -alpha, refs, time_manager);
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
    if board.is_insufficient_material() {
        return 0;
    }
    #[cfg(debug_assertions)]
    {
        let (real_white, real_black) = board.rebuild_acc();

        if real_white != board.game_state.acc_white || real_black != board.game_state.acc_black {
            println!("NNUE Drift Detected!");
            println!("Side to move: {:?}", board.turn);

            for i in 0..board.game_state.acc_white.vals.len() {
                let diff = real_white.vals[i] - board.game_state.acc_white.vals[i];
                if diff != 0 {
                    println!(
                        "White Acc Index [{}]: Expected {}, Got {} (Diff: {})",
                        i, real_white.vals[i], board.game_state.acc_white.vals[i], diff
                    );
                }
            }

            for i in 0..board.game_state.acc_black.vals.len() {
                let diff = real_black.vals[i] - board.game_state.acc_black.vals[i];
                if diff != 0 {
                    println!(
                        "Black Acc Index [{}]: Expected {}, Got {} (Diff: {})",
                        i, real_black.vals[i], board.game_state.acc_black.vals[i], diff
                    );
                }
            }
            panic!("Accumulator out of sync!");
        }
    }
    let (friendly_acc, opp_acc) = if board.turn == PieceColor::WHITE {
        (board.game_state.acc_white, board.game_state.acc_black)
    } else {
        (board.game_state.acc_black, board.game_state.acc_white)
    };
    let score = NNUE_NETWORK.evaluate_nnue(&friendly_acc, &opp_acc);

    score
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
    board: &mut Board,
    tt_table: &mut TranspositionTable,
    time_manager: &TimeManager,
) -> SearchOutput {
    let mut refs = SearchRefs::new_search_refs(tt_table);
    let mut current_depth = 1;
    let mut principal_variation: Vec<MoveData> = Vec::new();
    let mut best_eval = -INFINITY;

    while !time_manager.check_if_depth_done(current_depth) {
        let mut alpha = -INFINITY;
        let mut beta = INFINITY;
        if current_depth >= 5 {
            alpha = best_eval - VAL_WINDOW;
            beta = best_eval + VAL_WINDOW;
        }
        if time_manager.check_if_soft_time_done() {
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
            time_manager,
        );
        if time_manager.check_if_done(refs.get_nodes_evaluated()) {
            principal_variation = old_pv;
            break;
        }

        best_eval = eval;
        if current_depth >= 5 && best_eval >= beta || best_eval <= alpha {
            alpha = -INFINITY;
            beta = INFINITY;
            let eval = search_common(
                board,
                current_depth as i32,
                0,
                alpha,
                beta,
                &mut principal_variation,
                &mut refs,
                time_manager,
            );
            best_eval = eval;
        }
        current_depth += 1;
    }
    if principal_variation.is_empty() {
        let tt_mv = refs
            .get_transposition_table()
            .get_tt_move(board.game_state.zobrist_hash);
        let mut moves = generate_moves(board, false);
        if moves.is_empty() {
            panic!("ran search in a position with zero legal moves")
        }
        let mut scores = get_moves_score(&moves, 0, board, tt_mv, &refs);
        pick_move(&mut moves, 0, &mut scores);
        principal_variation.push(moves.get_move(0));
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
    time_manager: &TimeManager,
) -> i32 {
    debug_assert!(ply >= 0);
    debug_assert!(alpha < beta);
    debug_assert!(alpha >= -INFINITY);
    debug_assert!(beta <= INFINITY);
    // Stop search if time has elapsed
    if time_manager.check_if_done(refs.get_nodes_evaluated()) {
        return 0;
    }
    let mut best_score = -INFINITY;
    update_check(board);
    if board.game_state.is_check && depth < 63 {
        depth += 1;
    }
    if depth <= 0 {
        return quiescence_search(board, alpha, beta, refs, time_manager);
    }

    if board.is_board_draw() {
        return 0;
    }
    let mut move_list = generate_moves(board, false);

    if move_list.len() == 0 {
        return if board.game_state.is_check {
            -MATE_VALUE + ply
        } else {
            0
        };
    }
    let mut tt_move = None;

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

    let curr_eval = if board.game_state.is_check {
        -INFINITY
    } else {
        eval(board)
    };
    if board.game_state.is_check {
        refs.disable_eval_ply(ply);
    } else {
        refs.set_eval_ply(ply, curr_eval);
    }
    let improving = is_improving(board, curr_eval, refs, ply);

    if alpha.abs() < 2000
        && depth <= RAZOR_DEPTH
        && curr_eval + RAZOR_MARGIN < beta
        && !board.game_state.is_check
    {
        let value = quiescence_search(board, alpha, alpha + 1, refs, time_manager);
        if value <= alpha {
            return value;
        }
    }
    if is_allowed_reverse_futility_pruning(depth as u8, beta, curr_eval, board, improving) {
        return curr_eval;
    }
    if !board.game_state.is_check
        && depth >= 3
        && curr_eval >= beta
        && board.has_major_or_minor_material()
    {
        refs.table.prefetch(board.calc_hash_after_null_move());
        let r = if depth > 10 {
            5
        } else if depth > 6 {
            4
        } else {
            3
        };
        board.make_null_move();
        let null_move_score = -search_common(
            board,
            depth - 1 - r,
            ply + 1,
            -beta,
            -beta + 1,
            pv,
            refs,
            time_manager,
        );
        board.unmake_null_move();
        if null_move_score >= beta {
            return null_move_score;
        }
    }

    if tt_move.is_none() && depth > 5 {
        depth -= 2;
    }

    let mut move_score = get_moves_score(&move_list, ply as usize, board, tt_move, &*refs);
    let mut best_move = None;
    let mut entry_type = EntryType::UpperBound;
    let mut quiet_moves_count = 0;

    let mut quiet_moves: Vec<MoveData> = Vec::with_capacity(move_list.len());
    let mut is_pvs = false;
    for i in 0..move_list.len() {
        if time_manager.check_if_only_nodes_done(refs.get_nodes_evaluated()) {
            return 0;
        }
        let mut is_quiet_move = false;
        pick_move(&mut move_list, i as u8, &mut move_score);

        let mut curr_move = move_list.get_move(i);
        let mut see_val = 0;
        while Some(curr_move) != tt_move && curr_move.is_capture() {
            see_val = static_exchange_evaluation(board, curr_move);
            if see_val >= 0 {
                break;
            }

            let old_move = curr_move;
            move_score[i] = -BASE_CAPTURE + capture_formula(board, curr_move);
            pick_move(&mut move_list, i as u8, &mut move_score);

            curr_move = move_list.get_move(i);
            debug_assert!(
                curr_move.to()
                    != board
                        .get_piece_bitboard(board.turn.opposite(), KING)
                        .pop_lsb(),
                "move is {:?} and fen is {}",
                curr_move,
                board.to_fen()
            );
            if curr_move == old_move {
                break;
            };
        }

        if curr_move.is_capture()
            && Some(curr_move) != tt_move
            && see_val < -25 * depth * depth
            && !board.game_state.is_check
            && alpha > -MATE_VALUE + 500
            && i > 1
        {
            continue;
        }

        if board.is_quiet_move(curr_move) {
            is_quiet_move = true;
            if should_movecount_based_pruning(depth as u32, quiet_moves_count, alpha, improving)
                && is_pvs
            {
                continue;
            }
            quiet_moves_count += 1;
        }
        refs.table.prefetch(board.calc_hash_after_move(&curr_move));
        refs.increment_nodes_evaluated();
        let mut node_pv: Vec<MoveData> = Vec::new();
        let hist = refs.get_history_value(curr_move, board.turn)
            + refs.get_cont_history(board, ply, curr_move);
        refs.set_move_ply(ply, curr_move, board);
        let (curr_white_acc, curr_black_acc) =
            (board.game_state.acc_white, board.game_state.acc_black);
        board.make_move(curr_move);

        let mut score_mv = 0;
        if depth >= 3 && !curr_move.is_capture() && !curr_move.is_promotion() && is_pvs {
            let new_depth = depth - reduce_depth(depth, i as i32, improving, hist);

            score_mv = -search_common(
                board,
                new_depth,
                ply + 1,
                -alpha - 1,
                -alpha,
                &mut node_pv,
                refs,
                time_manager,
            );
            if score_mv > alpha {
                score_mv = -search_common(
                    board,
                    depth - 1,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    &mut node_pv,
                    refs,
                    time_manager,
                );
            }
        } else if is_pvs {
            score_mv = -search_common(
                board,
                depth - 1,
                ply + 1,
                -alpha - 1,
                -alpha,
                &mut node_pv,
                refs,
                time_manager,
            );
        }
        if !is_pvs || score_mv > alpha {
            score_mv = -search_common(
                board,
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
                &mut node_pv,
                refs,
                time_manager,
            );
        }

        board.unmake_move(curr_move);
        debug_assert!(
            curr_white_acc == board.game_state.acc_white
                && curr_black_acc == board.game_state.acc_black
        );
        if score_mv >= beta {
            entry_type = EntryType::LowerBound;
            best_move = Some(curr_move);

            refs.table.store(
                board.game_state.zobrist_hash,
                depth as u8,
                score_mv,
                entry_type,
                best_move,
            );

            if !curr_move.is_capture() {
                refs.store_killers(curr_move, ply as usize);

                refs.add_history(board.turn, curr_move, depth, false);
                refs.add_cont_hist(board, depth, ply, curr_move, false);
            }
            for quiet_move in quiet_moves {
                refs.add_cont_hist(board, depth, ply, quiet_move, true);
                refs.add_history(board.turn, quiet_move, depth, true);
            }

            return score_mv;
        }
        if score_mv > best_score {
            best_score = score_mv;
            if score_mv > alpha {
                alpha = score_mv;
                best_move = Some(curr_move);
                entry_type = EntryType::Exact;
                // Update PV
                pv.clear();
                pv.push(curr_move);
                pv.append(&mut node_pv);
            }
        }

        if is_quiet_move {
            quiet_moves.push(curr_move);
        }
        is_pvs = true;
    }

    refs.get_transposition_table().store(
        board.game_state.zobrist_hash,
        depth as u8,
        best_score,
        entry_type,
        best_move,
    );

    best_score
}
