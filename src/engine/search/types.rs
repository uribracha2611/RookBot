use crate::engine::board::piece::PieceColor;
use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::move_ordering::{KillerMoves, BASE_KILLER};
use crate::engine::search::transposition_table::TranspositionTable;
use std::time::{Duration, Instant};
const HISTORY_MAX: i32 = 16_384;


pub struct SearchOutput {
    pub nodes_evaluated: u64,
    pub principal_variation: Vec<MoveData>,
    pub eval: i32,
    pub depth: i32,
}

impl SearchOutput {
    pub fn new(
        nodes_evaluated: u64,
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

    pub fn get_nodes_evaluated(&self) -> u64 {
        self.nodes_evaluated
    }

    pub fn get_principal_variation(&self) -> &Vec<MoveData> {
        &self.principal_variation
    }
}


pub struct SearchInput {
    pub depth: Option<u8>,
    pub move_time: Option<Duration>,
    pub node_count: Option<u64>,
}
impl SearchInput {
    pub fn depth_input(depth: u8) -> SearchInput
    {
        SearchInput { move_time: None, depth: Some(depth), node_count: None }
    }
    pub fn time_input(move_time: Duration) -> SearchInput {
        SearchInput { move_time: Some(move_time), depth: None, node_count: None }
    }
    pub fn node_count_input(node_count: u64) -> SearchInput {
        SearchInput { move_time: None, depth: None, node_count: Some(node_count) }
    }
}

pub struct SearchRefs<'a> {
    killer_moves: KillerMoves,
    nodes_evaluated: u64,
    start_time: Option<Instant>,
    time_limit: Option<Duration>,
    nodes_limit: Option<u64>,
    history_table: [[[i32; 64]; 64]; 2],
    eval_stack: [Option<i32>; 256],
    move_stack: [Option<MoveData>; 256],
    continuation_history: Vec<Vec<i32>>,
    pub table: &'a mut TranspositionTable,

}
impl SearchRefs<'_> {
    pub fn new_timed_search<'a>(
        time_limit: &Duration,
        transposition_table: &'a mut TranspositionTable,
    ) -> SearchRefs<'a> {
        let killer_moves: KillerMoves = [[MoveData::default(); 2]; 256];
        let history_table = [[[0; 64]; 64]; 2];
        SearchRefs {
            killer_moves,
            nodes_evaluated: 0,
            start_time: Some(Instant::now()),
            time_limit: Some(*time_limit),
            nodes_limit: None,
            history_table,
            eval_stack: [None; 256],
            move_stack: [None; 256],
            table: transposition_table,
            continuation_history: vec![vec![0; 64 * 12 * 64 * 12]; 2],
        }
    }
    pub fn new_depth_search(
        transposition_table: &'_ mut TranspositionTable,
    ) -> SearchRefs<'_> {
        let killer_moves: KillerMoves = [[MoveData::default(); 2]; 256];
        let history_table = [[[0; 64]; 64]; 2];
        SearchRefs {
            killer_moves,
            nodes_evaluated: 0,
            start_time: None,
            time_limit: None,
            nodes_limit: None,
            history_table,
            eval_stack: [None; 256],
            move_stack: [None; 256],
            table: transposition_table, // Removed &mut here
            continuation_history: vec![vec![0; 64 * 12 * 64 * 12]; 2],
        }
    }
    pub fn new_node_search(node_count: u64,
                           transposition_table: &'_ mut TranspositionTable) -> SearchRefs<'_>
    {
        let killer_moves: KillerMoves = [[MoveData::default(); 2]; 256];
        let history_table = [[[0; 64]; 64]; 2];
        SearchRefs {
            killer_moves,
            nodes_evaluated: 0,
            start_time: None,
            time_limit: None,
            nodes_limit: Some(node_count),
            history_table,
            eval_stack: [None; 256],
            move_stack: [None; 256],
            table: transposition_table, // Removed &mut here
            continuation_history: vec![vec![0; 64 * 12 * 64 * 12]; 2],
        }
    }

    #[inline(always)]
    pub fn is_nodes_exceeded(&self) -> bool {
        if let Some(node_limit) = self.nodes_limit {
            return self.nodes_evaluated >= node_limit;
        }
        false
    }
    #[inline(always)]
    pub fn is_time_elapsed_iterative_search(&self) -> bool {
        if let Some(start_time) = self.start_time {
            if let Some(time) = self.time_limit {
                return start_time.elapsed() * 2 > time;
            }
            return false;
        }
        false
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
    pub fn add_history(&mut self, color: PieceColor, mv: MoveData, depth: i32, is_malus: bool) {
        let sign = if is_malus { -1 } else { 1 };
        let bonus = (Self::calculate_history_bonus(depth) * sign).clamp(-HISTORY_MAX, HISTORY_MAX);


        self.history_table[color as usize][mv.from as usize][mv.to as usize] += bonus - self.history_table[color as usize][mv.from as usize][mv.to as usize] * bonus.abs() / HISTORY_MAX;
    }
    #[inline(always)]
    pub fn get_history_value(&self, mv: &MoveData, color: PieceColor) -> i32 {
        self.history_table[color as usize][mv.from as usize][mv.to as usize]
    }
    #[inline(always)]
    pub fn get_nodes_evaluated(&self) -> u64 {
        self.nodes_evaluated
    }
    #[inline(always)]
    pub fn is_time_done(&self) -> bool {
        if let (Some(start_time), Some(time_limit)) = (self.start_time, self.time_limit) {
            return self.nodes_evaluated.is_multiple_of(8192) && start_time.elapsed() >= time_limit;
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
    #[inline(always)]
    pub fn calculate_history_bonus(depth: i32) -> i32 {
        300 * depth - 250
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
    #[inline(always)]
    pub fn add_cont_hist(&mut self, depth: i32, ply: i32, mv: &MoveData, is_malus: bool) {
        let sign = if is_malus { -1 } else { 1 };
        let bonus = Self::calculate_history_bonus(depth) * sign ;

        for ply_index in 1..=2 {
            if ply >= ply_index 
                && let Some(stack_mv) = &mut self.move_stack[(ply - ply_index) as usize] {
                    let index = Self::cont_hist_index(mv, stack_mv);
                    self.continuation_history[(ply_index - 1) as usize][index] += bonus - self.continuation_history[(ply_index - 1) as usize][index] * bonus.abs() / HISTORY_MAX;
                }
        }
    }


    pub fn get_cont_history(&self, ply: i32, mv: &MoveData) -> i32 {
        let mut cont = 0;
        if ply >= 1
            && let Some(stack_mv) = self.move_stack[(ply - 1) as usize] {
                cont += self.continuation_history[0][Self::cont_hist_index(mv, &stack_mv)];
            }
        if ply >= 2
            && let Some(stack_mv) = self.move_stack[(ply - 2) as usize] {
                cont += self.continuation_history[1][Self::cont_hist_index(mv, &stack_mv)]
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
}
