use std::ops::Neg;
use std::time::{Duration, Instant};
use crate::board::piece::PieceColor;
use crate::movegen::movedata::MoveData;
use crate::search::constants::MAX_EXTENSIONS;
use crate::search::move_ordering::{KillerMoves, BASE_KILLER};

#[derive(Clone, Copy)]
pub struct ChosenMove{
     mv: MoveData,
    eval: i32
}

impl ChosenMove {
    pub(crate) fn new(mv: MoveData, eval: i32) -> ChosenMove {
        ChosenMove {
            mv,
            eval
        }
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
            eval: -self.eval
        }
    }
}
pub struct SearchOutput {
    pub(crate) nodes_evaluated: i32,
    pub(crate) principal_variation: Vec<MoveData>,
    pub(crate) eval: i32,
    pub (crate) depth:i32
}

impl SearchOutput {
    pub fn new(nodes_evaluated: i32, principal_variation: Vec<MoveData>,eval:i32,depth:i32) -> SearchOutput {
        SearchOutput {
            nodes_evaluated,
            principal_variation,
            eval,
            depth
            
        }
    }

    pub fn get_nodes_evaluated(&self) -> i32 {
        self.nodes_evaluated
    }

    pub fn get_principal_variation(&self) -> &Vec<MoveData> {
        &self.principal_variation
    }
}
pub struct SearchInput{
    pub(crate) depth:u8,
}
pub struct SearchRefs
{
    killer_moves: KillerMoves,
    nodes_evaluated: i32,
    start_time: Option<Instant>,
    time_limit: Option<Duration>,
    history_table: [[[i32; 64]; 64]; 2],
    eval_stack:[Option<i32>;256],
    current_extensions: i32
}
impl SearchRefs {
    pub fn new_timed_search(killer_moves: KillerMoves, start_time:&Instant, time_limit: &Duration, history_table: [[[i32; 64]; 64]; 2]) -> SearchRefs {
        SearchRefs {
            killer_moves,
            nodes_evaluated: 0,
            start_time: Some(*start_time),
            time_limit: Some(*time_limit),
            history_table,
            eval_stack:[None;256],
            current_extensions: 0
        }
    }
    pub fn new_depth_search(killer_moves: KillerMoves, history_table: [[[i32; 64]; 64]; 2]) -> SearchRefs {
        SearchRefs {
            killer_moves,
            nodes_evaluated: 0,
            start_time: None,
            time_limit: None,
            history_table,
            eval_stack:[None;256],
            current_extensions: 0
        }
    }
    #[inline(always)]
    pub fn increment_nodes_evaluated(&mut self) {
        self.nodes_evaluated += 1;
    }
    #[inline(always)]
    pub fn add_history(&mut self, color: PieceColor, mv: MoveData, depth: i32) {
        self.history_table[color as usize][mv.from as usize][mv.to as usize] += depth * depth;
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
        self.eval_stack[ply as usize]
    }
    pub fn set_eval_ply(&mut self, ply: i32,eval:i32) {
        self.eval_stack[ply as usize] = Some(eval);
    }
    pub fn disable_eval_ply(&mut self, ply: i32) {
        self.eval_stack[ply as usize] = None;
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
pub fn return_killer_move_score(&self,ply:i32,mv:MoveData)->Option<i32>{
    for i in 0..2{
        if self.killer_moves[ply as usize][i]==mv{
        return Some( BASE_KILLER-(i as i32));
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
    
}
