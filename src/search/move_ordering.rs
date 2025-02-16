use clap::builder::styling::Color;
use crate::board::board::Board;
use crate::board::piece::PieceColor;
use crate::movegen::movedata::MoveData;
use crate::movegen::movelist::MoveList;

pub const MVV_LVA: [[u32; 6]; 6] = [
    [10,   11,   12,   13,   14,   15],  // Victim: PAWN
    [20,   21,   22,   23,   24,   25],  // Victim: KNIGHT
    [30,   31,   32,   33,   34,   35],  // Victim: BISHOP
    [40,   41,   42,   43,   44,   45],  // Victim: ROOK
    [50,   51,   52,   53,   54,   55],  // Victim: QUEEN
    [0,    0,    0,    0,    0,    0],   // Victim: KING
];
pub const BASE_CAPTURE: u32 =u32::MAX-100;
pub const BASE_KILLER: u32 = u32::MAX-200;
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
pub fn get_moves_score(moves:&MoveList, tt_move:&MoveData, killer_moves: KillerMoves, ply:usize, history_table: [[[u32; 64]; 64]; 2],color: PieceColor) -> Vec<u32> {
    let mut scores = Vec::with_capacity(moves.len());
    for mv in moves.iter() {
        scores.push(get_move_score(mv, tt_move, &killer_moves, ply, &history_table,color));
    }
    scores
}
pub fn get_move_score(
    mv: &MoveData,
    tt_move: &MoveData,
    killer_moves: &KillerMoves,
    ply: usize,
    history_table: &[[[u32; 64]; 64]; 2],
    color: PieceColor
) -> u32 {
    if mv == tt_move {
        u32::MAX
    } else if mv.is_capture() {
        return BASE_CAPTURE + MVV_LVA[mv.get_captured_piece().unwrap().piece_type as usize][mv.piece_to_move.piece_type as usize];
    } else if *mv == killer_moves[ply][0] {
        return BASE_KILLER;
    } else if *mv == killer_moves[ply][1] {
        return BASE_KILLER - 1;
    } else {
        let history_val = history_table[color as usize][mv.from as usize][mv.to as usize];
   
         return history_val;
        
    }
}

pub fn get_capture_score_only(move_data: MoveData, tt_move:MoveData) -> u32 {
    if move_data==tt_move{
        u32::MAX
    }
    else {
        
         BASE_CAPTURE + MVV_LVA[move_data.get_captured_piece().unwrap().piece_type as usize][move_data.piece_to_move.piece_type as usize]
    }
    
}
pub fn get_capture_score(mv_list:MoveList, tt_move:MoveData) -> Vec<u32> {
    let mut scores = Vec::with_capacity(mv_list.len());
    for mv in mv_list.iter() {
        scores.push(get_capture_score_only(*mv, tt_move));
    }
    scores
}

