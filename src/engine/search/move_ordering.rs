use crate::engine::board::board::Board;
use crate::engine::movegen::movedata::MoveData;
use crate::engine::movegen::movelist::MoveList;
use crate::engine::search::types::SearchRefs;

pub const MVV_LVA: [[u32; 6]; 6] = [
    [10, 11, 12, 13, 14, 15], // Victim: PAWN
    [20, 21, 22, 23, 24, 25], // Victim: KNIGHT
    [30, 31, 32, 33, 34, 35], // Victim: BISHOP
    [40, 41, 42, 43, 44, 45], // Victim: ROOK
    [50, 51, 52, 53, 54, 55], // Victim: QUEEN
    [0, 0, 0, 0, 0, 0],       // Victim: KING
];

pub const BASE_CAPTURE: i32 = 10000000;
pub type KillerMoves = [[Option<MoveData>; 2]; 256];

pub fn get_moves_score(
    moves: &mut MoveList,
    ply: usize,
    board: &Board,
    refs: &SearchRefs,
    start: usize,
    end: usize,
) {
    for mv in &mut moves[start..end] {
        mv.set_score(get_move_score(mv.get_mv(), ply, board, refs));
    }
}
pub fn get_move_score(mv: MoveData, ply: usize, board: &Board, refs: &SearchRefs) -> i32 {
    if mv.is_capture() {
        BASE_CAPTURE + capture_formula(board, mv)
    } else {
        (refs.get_history_value(mv, board.turn) as i32)
            + (refs.get_cont_history(board, ply as i32, mv) as i32)
    }
}
#[inline(always)]
pub fn capture_formula(board: &Board, mv: MoveData) -> i32 {
    board.game_state.squares[mv.get_capture_square() as usize]
        .unwrap()
        .get_value()
        * 10
        - board.game_state.squares[mv.from() as usize]
            .unwrap()
            .get_value()
}
pub fn get_capture_score_only(board: &Board, move_data: MoveData) -> i32 {
    BASE_CAPTURE + capture_formula(board, move_data)
}

pub fn get_capture_scores(mv_list: &mut MoveList, board: &Board, start: usize, end: usize) {
    for mv in &mut mv_list[start..end] {
        mv.set_score(get_capture_score_only(board, mv.get_mv()));
    }
}
