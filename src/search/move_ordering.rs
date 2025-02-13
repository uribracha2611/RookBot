use crate::movegen::movedata::MoveData;
use crate::movegen::movelist::MoveList;

pub const MVV_LVA: [[u8; 6]; 6] = [
    [10,   11,   12,   13,   14,   15],  // Victim: PAWN
    [20,   21,   22,   23,   24,   25],  // Victim: KNIGHT
    [30,   31,   32,   33,   34,   35],  // Victim: BISHOP
    [40,   41,   42,   43,   44,   45],  // Victim: ROOK
    [50,   51,   52,   53,   54,   55],  // Victim: QUEEN
    [0,    0,    0,    0,    0,    0],   // Victim: KING
];
pub const BASE_CAPTURE: u8 =100;
pub const BASE_KILLER: u8 = 50;
pub type KillerMoves = [[MoveData; 2];256];
pub fn store_killers(killer_moves: &mut KillerMoves, mv: MoveData, ply: usize) {
    killer_moves[ply][1] = killer_moves[ply][0];
    killer_moves[ply][0] = mv;
}
pub fn get_moves_score(moves:&MoveList,tt_move:&MoveData,killer_moves: KillerMoves,ply:usize) -> Vec<u8> {
    let mut scores = Vec::with_capacity(moves.len());
    for mv in moves.iter() {
        scores.push(get_move_score(mv,tt_move,killer_moves,ply));
    }
    scores
}
pub fn get_move_score(mv: &MoveData,tt_move:&MoveData,killer_moves: KillerMoves,ply:usize) -> u8 {
    if mv==tt_move{
        return u8::MAX;
    }
    else if mv.is_capture() {
        return BASE_CAPTURE + MVV_LVA[mv.get_captured_piece().unwrap().piece_type as usize][mv.piece_to_move.piece_type as usize];
    }
   else  if *mv == killer_moves[ply][0] {

        return BASE_KILLER;
    }
    else if  *mv == killer_moves[ply][1]
     {

        return BASE_KILLER - 1;
    }
    0
}
