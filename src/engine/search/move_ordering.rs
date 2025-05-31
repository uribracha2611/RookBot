use crate::engine::board::board::Board;
use crate::engine::board::piece::PieceColor;
use crate::engine::movegen::movedata::MoveData;
use crate::engine::movegen::movelist::MoveList;
use crate::engine::search::transposition_table::TranspositionTable;
use crate::engine::search::types::SearchRefs;

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
pub fn get_moves_score(moves:&MoveList,  ply:usize, board: &Board, tt_move:MoveData, refs:&SearchRefs,color: PieceColor) -> Vec<i32> {
    let mut scores = Vec::with_capacity(moves.len());
    for mv in moves.iter() {
        scores.push(get_move_score(mv, ply, tt_move, board, &refs, color));
    }
    scores
}
pub fn get_move_score(
    mv: &MoveData,

    ply: usize,
    tt_move:MoveData,
    board: &Board,
    refs:&SearchRefs,
    color: PieceColor
) -> i32 {
    if *mv==tt_move{
        return i32::MAX
    }
      if mv.is_capture() {

        BASE_CAPTURE+((mv.get_captured_piece().unwrap().get_value()*10)-mv.piece_to_move.get_value())
    }
        // else if board.is_move_check(mv){
        //     return  BASE_KILLER+1;
        // }
        // else if let Some(killer_val)= refs.return_killer_move_score(ply as i32, *mv) { 
        //     return killer_val
        //     
        // }
    else {
       refs.get_history_value(mv,board.turn)
        
    }
}

pub fn get_capture_score_only(board: &Board,move_data: MoveData, tt_move:MoveData,refs:&SearchRefs) -> i32 {
    if move_data==tt_move{
        i32::MAX
    }
    else {
        BASE_CAPTURE+move_data.get_captured_piece().unwrap().mvv_score()+ refs.get_capture_history(&move_data)          
    }
    
}
pub fn get_capture_score(board: &Board,mv_list:MoveList, tt_move:MoveData,refs:&SearchRefs) -> Vec<i32> {
    let mut scores = Vec::with_capacity(mv_list.len());
    for mv in mv_list.iter() {
        scores.push(get_capture_score_only(board,*mv, tt_move,refs));
    }
    scores
}

