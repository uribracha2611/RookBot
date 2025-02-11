use crate::movegen::movedata::MoveData;

pub const MVV_LVA: [[u8; 6]; 6] = [
    [10,   11,   12,   13,   14,   15],  // Victim: PAWN
    [20,   21,   22,   23,   24,   25],  // Victim: KNIGHT
    [30,   31,   32,   33,   34,   35],  // Victim: BISHOP
    [40,   41,   42,   43,   44,   45],  // Victim: ROOK
    [50,   51,   52,   53,   54,   55],  // Victim: QUEEN
    [0,    0,    0,    0,    0,    0],   // Victim: KING
];
pub const BASE_CAPTURE: u8 =100;
pub fn get_move_score(mv: &MoveData) -> u8 {
    if mv.is_capture() {
        return BASE_CAPTURE + MVV_LVA[mv.get_captured_piece().unwrap().piece_type as usize][mv.piece_to_move.piece_type as usize];
    }
    0
}
