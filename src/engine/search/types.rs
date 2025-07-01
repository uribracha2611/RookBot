use crate::engine;
use crate::engine::board::piece::PieceColor;
use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::constants::MAX_EXTENSIONS;
use crate::engine::search::move_ordering::{KillerMoves, BASE_KILLER};
use crate::engine::search::transposition_table::TranspositionTable;
use std::ops::Neg;
use std::time::{Duration, Instant};

pub type CaptureHistoryTable = [[[i32; 12]; 64]; 12];
#[derive(Copy, Clone)]

pub struct SearchStackEntry {
    pub eval: i32,
    pub curr_move: MoveData,
}
impl SearchStackEntry {
    pub fn new(eval: i32, curr_move: MoveData) -> SearchStackEntry {
        SearchStackEntry { eval, curr_move }
    }
}
#[derive(Clone, Copy)]
pub struct ChosenMove {
    mv: MoveData,
    eval: i32,
}

impl ChosenMove {
    pub fn new(mv: MoveData, eval: i32) -> ChosenMove {
        ChosenMove { mv, eval }
    }
    pub fn get_move(&self) -> MoveData {
        self.mv
    }
    pub fn get_eval(&self) -> i32 {
        self.eval
    }
}
impl Neg for ChosenMove {
    type Output = ChosenMove;

    fn neg(self) -> Self::Output {
        ChosenMove {
            mv: self.mv,
            eval: -self.eval,
        }
    }
}
pub struct SearchOutput {
    pub nodes_evaluated: i32,
    pub principal_variation: Vec<MoveData>,
    pub eval: i32,
    pub depth: i32,
}

impl SearchOutput {
    pub fn new(
        nodes_evaluated: i32,
        principal_variation: Vec<MoveData>,
        eval: i32,
        depth: i32,
    ) -> SearchOutput {
        SearchOutput {
            nodes_evaluated,
            principal_variation,
            eval,
            depth,
        }
    }

    pub fn get_nodes_evaluated(&self) -> i32 {
        self.nodes_evaluated
    }

    pub fn get_principal_variation(&self) -> &Vec<MoveData> {
        &self.principal_variation
    }
}
pub struct SearchInput {
    pub depth: u8,
}

pub struct SearchRefs<'a> {
    killer_moves: KillerMoves,
    nodes_evaluated: i32,
    start_time: Option<Instant>,
    time_limit: Option<Duration>,
    history_table: [[[i32; 64]; 64]; 2],
    caphist: CaptureHistoryTable,
    eval_stack: [Option<i32>; 256],
    move_stack: [Option<MoveData>; 256],
    current_extensions: i32,
    continuation_history: Vec<Vec<i32>>,
    pub table: &'a mut TranspositionTable,
}
impl SearchRefs<'_> {
    pub fn new_timed_search<'a>(
        killer_moves: KillerMoves,
        start_time: &Instant,
        time_limit: &Duration,
        history_table: [[[i32; 64]; 64]; 2],
        cap_hist: CaptureHistoryTable,
        transposition_table: &'a mut TranspositionTable,
    ) -> SearchRefs<'a> {
        SearchRefs {
            killer_moves,
            nodes_evaluated: 0,
            start_time: Some(*start_time),
            time_limit: Some(*time_limit),
            history_table,
            caphist: cap_hist,
            eval_stack: [None; 256],
            move_stack: [None; 256],
            current_extensions: 0,
            table: transposition_table,
            continuation_history: vec![vec![0; 64 * 12 * 64 * 12]; 2],
        }
    }
    pub fn new_depth_search(
        killer_moves: KillerMoves,
        history_table: [[[i32; 64]; 64]; 2],
        cap_hist: CaptureHistoryTable,
        transposition_table: &mut TranspositionTable,
    ) -> SearchRefs {
        SearchRefs {
            killer_moves,
            nodes_evaluated: 0,
            start_time: None,
            time_limit: None,
            history_table,
            caphist: cap_hist,
            eval_stack: [None; 256],
            move_stack: [None; 256],
            current_extensions: 0,
            table: transposition_table, // Removed &mut here
            continuation_history: vec![vec![0; 64 * 12 * 64 * 12]; 2],
        }
    }

    #[inline(always)]
    pub fn get_transposition_table(&mut self) -> &mut TranspositionTable {
        self.table
    }
    #[inline(always)]
    pub fn increment_nodes_evaluated(&mut self) {
        self.nodes_evaluated += 1;
    }
    #[inline(always)]
    pub fn add_history(&mut self, color: PieceColor, mv: MoveData, depth: i32,is_malus:bool) {
        if is_malus {
            self.history_table[color as usize][mv.from as usize][mv.to as usize] -= depth * depth;
        }
        else {
            self.history_table[color as usize][mv.from as usize][mv.to as usize] += depth * depth;
        }
    }
    #[inline(always)]
    pub fn get_history_value(&self, mv: &MoveData, color: PieceColor) -> i32 {
        self.history_table[color as usize][mv.from as usize][mv.to as usize]
    }
    #[inline(always)]
    pub fn get_nodes_evaluated(&self) -> i32 {
        self.nodes_evaluated
    }
    #[inline(always)]
    pub fn is_time_done(&self) -> bool {
        if let (Some(start_time), Some(time_limit)) = (self.start_time, self.time_limit) {
            return start_time.elapsed() >= time_limit;
        }

        false
    }
    #[inline(always)]
    pub fn get_eval_ply(&self, ply: i32) -> Option<i32> {
        if ply >= 256 {
            return None;
        }
        self.eval_stack[ply as usize]
    }
    pub fn set_eval_ply(&mut self, ply: i32, eval: i32) {
        self.eval_stack[ply as usize] = Some(eval);
    }
    pub fn disable_eval_ply(&mut self, ply: i32) {
        self.eval_stack[ply as usize] = None;
    }
    pub fn set_move_ply(&mut self, ply: i32, move_data: MoveData) {
        self.move_stack[ply as usize] = Some(move_data);
    }
    pub fn get_move_ply(&self, ply: i32) -> Option<MoveData> {
        self.move_stack[ply as usize]
    }
    fn cont_hist_index(mv_1: &MoveData, mv_2: &MoveData) -> usize {
        let to1 = mv_1.to as usize;
        let to2 = mv_2.to as usize;
        let piece1 = mv_1.piece_to_move;
        let piece2 = mv_2.piece_to_move;
        ((to1 * 12 + piece1.to_history_index()) * 64 + to2) * 12 + piece2.to_history_index()
    }
    pub fn increament_cont_hist(&mut self, depth: i32, ply: i32, mv: &MoveData) {
        if ply >= 1 {
            if let Some(stack_mv) = &mut self.move_stack[(ply - 1) as usize] {
                let index = Self::cont_hist_index(mv, stack_mv);
                self.continuation_history[0][index] += depth * depth;
            }
        }

        if ply >= 2 {
            if let Some(stack_mv) = &mut self.move_stack[(ply - 2) as usize] {
                let index = Self::cont_hist_index(mv, stack_mv);
                self.continuation_history[1][index] += depth * depth;
            }
        }
    }
    pub fn decreament_cont_hist(&mut self, depth: i32, ply: i32, mv: &MoveData) {
        if ply >= 1 {
            if let Some(stack_mv) = &mut self.move_stack[(ply - 1) as usize] {
                let index = Self::cont_hist_index(mv, stack_mv);
                self.continuation_history[0][index] -= depth * depth;
            }
        }

        if ply >= 2 {
            if let Some(stack_mv) = &mut self.move_stack[(ply - 2) as usize] {
                let index = Self::cont_hist_index(mv, stack_mv);
                self.continuation_history[1][index] -= depth * depth;
            }
        }
    }

    pub fn get_cont_history(&self, ply: i32, mv: &MoveData) -> i32 {
        let mut cont = 0;
        if ply >= 1 {
            if let Some(stack_mv) = self.move_stack[(ply - 1) as usize] {
                cont += self.continuation_history[0][Self::cont_hist_index(mv, &stack_mv)];
            }
        }
        if ply >= 2 {
            if let Some(stack_mv) = self.move_stack[(ply - 2) as usize] {
                cont += self.continuation_history[1][Self::cont_hist_index(mv, &stack_mv)]
            }
        }
        cont
    }

    pub fn store_killers(&mut self, mv: MoveData, ply: usize) {
        let first_killer = self.killer_moves[ply][0];

        // First killer must not be the same as the move being stored.
        if first_killer != mv {
            // Shift all the moves one index upward...
            for i in (1..2).rev() {
                self.killer_moves[ply][i] = self.killer_moves[ply][i - 1];
            }

            // and add the new killer move in the first spot.
            self.killer_moves[ply][0] = mv;
        }
    }
    pub fn return_killer_move_score(&self, ply: i32, mv: MoveData) -> Option<i32> {
        for i in 0..2 {
            if self.killer_moves[ply as usize][i] == mv {
                return Some(BASE_KILLER - (i as i32));
            }
        }
        None
    }
    #[inline(always)]
    pub fn increment_extensions(&mut self) {
        self.current_extensions += 1;
    }
    #[inline(always)]
    pub fn is_extension_allowed(&self) -> bool {
        self.current_extensions <= MAX_EXTENSIONS
    }
    #[inline(always)]
    pub fn reset_extensions(&mut self) {
        self.current_extensions = 0;
    }

    #[inline(always)]
    pub fn add_capture_history(&mut self, mv: &MoveData, depth: i32) {
        let captured_piece_index = mv.get_captured_piece().unwrap().to_history_index();
        let capture_piece_index = mv.piece_to_move.to_history_index();
        let square_index = mv.get_capture_square().unwrap();
        self.caphist[captured_piece_index][square_index as usize][capture_piece_index as usize] +=
            depth * depth;
    }
    pub fn reduce_capture_history(&mut self, mv: &MoveData, depth: i32) {
        let captured_piece_index = mv.get_captured_piece().unwrap().to_history_index();
        let capture_piece_index = mv.piece_to_move.to_history_index();
        let square_index = mv.get_capture_square().unwrap();
        self.caphist[captured_piece_index as usize][square_index as usize]
            [capture_piece_index as usize] -= depth * depth;
    }
    pub fn get_capture_history(&self, mv: &MoveData) -> i32 {
        let captured_piece_index = mv.get_captured_piece().unwrap().to_history_index();
        let capture_piece_index = mv.piece_to_move.to_history_index();
        let square_index = mv.get_capture_square().unwrap();
        self.caphist[captured_piece_index as usize][square_index as usize]
            [capture_piece_index as usize]
    }
}
