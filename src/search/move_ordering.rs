use clap::builder::styling::Color;
use crate::board::board::Board;
use crate::board::piece::PieceColor;
use crate::board::see::static_exchange_evaluation;
use crate::movegen::movedata::MoveData;
use crate::movegen::movelist::MoveList;
use crate::search::types::SearchRefs;

pub const MVV_LVA: [[u32; 6]; 6] = [
    [10,   11,   12,   13,   14,   15],  // Victim: PAWN
    [20,   21,   22,   23,   24,   25],  // Victim: KNIGHT
    [30,   31,   32,   33,   34,   35],  // Victim: BISHOP
    [40,   41,   42,   43,   44,   45],  // Victim: ROOK
    [50,   51,   52,   53,   54,   55],  // Victim: QUEEN
    [0,    0,    0,    0,    0,    0],   // Victim: KING
];
pub const BASE_CAPTURE: i32 =10000000;
pub const BASE_KILLER: i32 = 5000000;
pub type KillerMoves = [[MoveData; 2];256];
pub fn store_killers(killer_moves: &mut KillerMoves, mv: MoveData, ply: usize) {
    let first_killer = killer_moves[ply][0];

    // First killer must not be the same as the move being stored.
    if first_killer != mv {
        // Shift all the moves one index upward...
        for i in (1..2).rev() {
            killer_moves[ply][i] = killer_moves[ply][i - 1];
        }

        // and add the new killer move in the first spot.
        killer_moves[ply][0] = mv;
    }
}
pub fn get_moves_score(moves:&MoveList, tt_move:&MoveData,  ply:usize, board: &Board, refs:&SearchRefs,color: PieceColor) -> Vec<i32> {
    let mut scores = Vec::with_capacity(moves.len());
    for mv in moves.iter() {
        scores.push(get_move_score(mv, tt_move, ply, board, &refs, color));
    }
    scores
}
pub fn get_move_score(
    mv: &MoveData,
    tt_move: &MoveData,
    ply: usize,

    board: &Board,
    refs:&SearchRefs,
    color: PieceColor
) -> i32 {
    if mv == tt_move {
        i32::MAX
    } else if mv.is_capture() {
        let see_score=static_exchange_evaluation(&board, mv.get_capture_square().unwrap() as i32, mv.get_captured_piece().unwrap(), mv.piece_to_move, mv.from as i32);
        if(see_score>=0){
            return BASE_CAPTURE+see_score;
        }
        else { 
            return  -BASE_CAPTURE-see_score;
        }

    }
        else if let Some(killer_val)= refs.return_killer_move_score(ply as i32, *mv) { 
            return killer_val
            
        }
    else {
        refs.get_history_value(mv, color)
        
    }
}

pub fn get_capture_score_only(board: &Board,move_data: MoveData, tt_move:MoveData) -> i32 {
    if move_data==tt_move{
        i32::MAX
    }
    else {
        let see_score=static_exchange_evaluation(&board, move_data.get_capture_square().unwrap() as i32, move_data.get_captured_piece().unwrap(), move_data.piece_to_move, move_data.from as i32);
        see_score
          
    }
    
}
pub fn get_capture_score(board: &Board,mv_list:MoveList, tt_move:MoveData) -> Vec<i32> {
    let mut scores = Vec::with_capacity(mv_list.len());
    for mv in mv_list.iter() {
        scores.push(get_capture_score_only(board,*mv, tt_move));
    }
    scores
}

