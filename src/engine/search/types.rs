use crate::engine::board::board::Board;
use crate::engine::board::piece::{Piece, PieceColor};
use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::move_ordering::{BASE_KILLER, KillerMoves};
use crate::engine::search::transposition_table::TranspositionTable;
use std::time::{Duration, Instant};

const HISTORY_MAX: i16 = 16_384;

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
    pub fn depth_input(depth: u8) -> SearchInput {
        SearchInput {
            move_time: None,
            depth: Some(depth),
            node_count: None,
        }
    }
    pub fn time_input(move_time: Duration) -> SearchInput {
        SearchInput {
            move_time: Some(move_time),
            depth: None,
            node_count: None,
        }
    }
    pub fn node_count_input(node_count: u64) -> SearchInput {
        SearchInput {
            move_time: None,
            depth: None,
            node_count: Some(node_count),
        }
    }
}

#[derive(Copy, Clone)]
struct MoveEntryStack {
    mv: MoveData,
    piece_moved: Piece,
}
pub struct SearchRefs<'a> {
    killer_moves: KillerMoves,
    nodes_evaluated: u64,
    start_time: Option<Instant>,
    history_table: [[[i16; 64]; 64]; 2],
    eval_stack: [Option<i32>; 256],
    move_stack: [Option<MoveEntryStack>; 256],
    continuation_history: Vec<Vec<i16>>,
    pub table: &'a mut TranspositionTable,
}
impl SearchRefs<'_> {
    pub fn new_search_refs<'a>(transposition_table: &'a mut TranspositionTable) -> SearchRefs<'a> {
        let killer_moves: KillerMoves = [[None; 2]; 256];
        let history_table = [[[0; 64]; 64]; 2];
        SearchRefs {
            killer_moves,
            nodes_evaluated: 0,
            start_time: Some(Instant::now()),
            history_table,
            eval_stack: [None; 256],
            move_stack: [None; 256],
            table: transposition_table,
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
    pub fn add_history(&mut self, color: PieceColor, mv: MoveData, depth: i32, is_malus: bool) {
        let sign = if is_malus { -1 } else { 1 };
        let bonus = (Self::calculate_history_bonus(depth) * sign).clamp(-HISTORY_MAX, HISTORY_MAX);
        let entry = &mut self.history_table[color as usize][mv.from() as usize][mv.to() as usize];
        let current = *entry as i32;
        let b = bonus as i32;
        let max = HISTORY_MAX as i32;
        *entry = (current + b - current * b.abs() / max) as i16;
    }
    #[inline(always)]
    pub fn get_history_value(&self, mv: MoveData, color: PieceColor) -> i32 {
        self.history_table[color as usize][mv.from() as usize][mv.to() as usize] as i32
    }
    #[inline(always)]
    pub fn get_nodes_evaluated(&self) -> u64 {
        self.nodes_evaluated
    }
    #[inline(always)]
    pub fn get_eval_ply(&self, ply: i32) -> Option<i32> {
        if ply >= 256 {
            return None;
        }
        self.eval_stack[ply as usize]
    }
    #[inline(always)]
    pub fn calculate_history_bonus(depth: i32) -> i16 {
        300 * (depth as i16) - 250
    }
    pub fn set_eval_ply(&mut self, ply: i32, eval: i32) {
        self.eval_stack[ply as usize] = Some(eval);
    }
    pub fn disable_eval_ply(&mut self, ply: i32) {
        self.eval_stack[ply as usize] = None;
    }
    pub fn set_move_ply(&mut self, ply: i32, move_data: MoveData, board: &Board) {
        self.move_stack[ply as usize] = Some(MoveEntryStack {
            mv: move_data,
            piece_moved: board.game_state.squares[move_data.from() as usize].unwrap(),
        });
    }
    pub fn get_move_ply(&self, ply: i32) -> Option<MoveEntryStack> {
        self.move_stack[ply as usize]
    }
    fn cont_hist_index(mv_1: MoveData, mv_2: MoveData, piece1: Piece, piece2: Piece) -> usize {
        let to1 = mv_1.to() as usize;
        let to2 = mv_2.to() as usize;
        ((to1 * 12 + piece1.to_history_index()) * 64 + to2) * 12 + piece2.to_history_index()
    }
    #[inline(always)]
    pub fn add_cont_hist(
        &mut self,
        board: &Board,
        depth: i32,
        ply: i32,
        mv: MoveData,
        is_malus: bool,
    ) {
        let sign = if is_malus { -1 } else { 1 };
        let bonus = Self::calculate_history_bonus(depth) * sign;

        for ply_index in 1..=2 {
            if ply >= ply_index
                && let Some(stack_mv) = self.move_stack[(ply - ply_index) as usize]
            {
                let piece_1 = board.game_state.squares[mv.from() as usize].unwrap();
                let index = Self::cont_hist_index(mv, stack_mv.mv, piece_1, stack_mv.piece_moved);

                let entry = &mut self.continuation_history[(ply_index - 1) as usize][index];

                let current = *entry as i32;
                let b = bonus as i32;
                let max = HISTORY_MAX as i32;

                *entry = (current + b - current * b.abs() / max) as i16;
            }
        }
    }

    pub fn get_cont_history(&self, board: &Board, ply: i32, mv: MoveData) -> i32 {
        let mut cont = 0;
        let piece_1 = board.game_state.squares[mv.from() as usize].unwrap();

        if ply >= 1
            && let Some(stack_mv) = self.move_stack[(ply - 1) as usize]
        {
            cont += self.continuation_history[0]
                [Self::cont_hist_index(mv, stack_mv.mv, piece_1, stack_mv.piece_moved)];
        }
        if ply >= 2
            && let Some(stack_mv) = self.move_stack[(ply - 2) as usize]
        {
            cont += self.continuation_history[1]
                [Self::cont_hist_index(mv, stack_mv.mv, piece_1, stack_mv.piece_moved)];
        }
        cont as i32
    }

    pub fn store_killers(&mut self, mv: MoveData, ply: usize) {
        let first_killer = self.killer_moves[ply][0];

        // First killer must not be the same as the move being stored.
        if first_killer != Some(mv) {
            // Shift all the moves one index upward...
            for i in (1..2).rev() {
                self.killer_moves[ply][i] = self.killer_moves[ply][i - 1];
            }

            // and add the new killer move in the first spot.
            self.killer_moves[ply][0] = Some(mv);
        }
    }
    pub fn return_killer_move_score(&self, ply: i32, mv: MoveData) -> Option<i32> {
        for i in 0..2 {
            if self.killer_moves[ply as usize][i] == Some(mv) {
                return Some(BASE_KILLER - (i as i32));
            }
        }
        None
    }
}
