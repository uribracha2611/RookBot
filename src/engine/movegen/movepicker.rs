use crate::engine::board::board::Board;
use crate::engine::movegen::constants::MAX_MOVES;
use crate::engine::movegen::generate::generate_moves_from_list;
use crate::engine::movegen::generate::MoveGenerationType::{CapturesOnly, QuietsOnly};
use crate::engine::movegen::movedata::MoveData;
use crate::engine::movegen::movelist::{MoveEntry, MoveList};
use crate::engine::movegen::movepicker::MOVEGENSTAGE::{BadCaptures, GenerateCaptures, GenerateQuiets, GoodCaptures, KillerMove1, KillerMove2, QuietMoves, TtMove, DONE};
use crate::engine::search::constants::INFINITY;
use crate::engine::search::move_ordering::{get_capture_score_only, get_move_score, BASE_CAPTURE};
use crate::engine::search::types::SearchRefs;
use std::cmp::PartialEq;

#[derive(PartialEq, Eq)]
enum MOVEGENSTAGE {
    TtMove,
    GenerateCaptures,
    GoodCaptures,
    KillerMove1,
    KillerMove2,
    GenerateQuiets,
    QuietMoves,
    BadCaptures,
    DONE,
}
impl MOVEGENSTAGE {}
pub struct MovePicker {
    curr_stage: MOVEGENSTAGE,
    moves: MoveList,
    killer_moves: [MoveData; 2],
    tt_move: MoveData,
    split: usize,
    ply: usize,
    only_captures: bool,
    quiet_size: i32,
}
pub fn get_best_index(start: usize, end: usize, mv_list: MoveList) -> usize {
    let mut best_index = 0;
    let mut best_score = -INFINITY;
    for i in start..end {
        if (mv_list[i].get_score() > best_score) {
            best_score = mv_list[i].get_score();
            best_index = i;
        }
    }

    best_index
}

impl MovePicker
{
    pub fn init_captures_only(ply: usize, tt_move: &MoveData, killer_moves: [MoveData; 2]) -> MovePicker {
        MovePicker { killer_moves, only_captures: true, tt_move: *tt_move, curr_stage: TtMove, moves: MoveList::default(), quiet_size: 0, split: 0, ply }
    }
    pub fn init_all_moves(ply: usize, tt_move: &MoveData, killer_moves: [MoveData; 2]) -> MovePicker {
        MovePicker { killer_moves, only_captures: false, tt_move: *tt_move, curr_stage: TtMove, moves: MoveList::default(), quiet_size: 0, split: 0, ply }
    }


    pub(crate) fn next(&mut self, board: &mut Board, refs: &SearchRefs) -> Option<MoveEntry> {
        loop {
            match &self.curr_stage {
                TtMove => {
                    self.curr_stage = GenerateCaptures;
                    if (self.tt_move.is_capture() || !self.only_captures) && board.is_legal_move(&self.tt_move) && self.tt_move != MoveData::default() {
                        return Some(MoveEntry::entry_from_mv(self.tt_move));
                    }
                }
                GenerateCaptures => {
                    generate_moves_from_list(board, &mut self.moves, 0, CapturesOnly);
                    self.split = self.moves.len();
                    for i in 0..self.moves.len() {
                        let mv_entry = &mut self.moves[i];
                        if mv_entry.get_mv() == self.tt_move {
                            mv_entry.set_score(-INFINITY);
                            continue;
                        }
                        let score = get_capture_score_only(&board, &mv_entry.get_mv(), self.tt_move, &refs);
                        mv_entry.set_score(score);
                    }
                    self.curr_stage = GoodCaptures;
                }
                GoodCaptures => {
                    while !self.moves.is_empty() {
                        let best_index = get_best_index(0, self.moves.len(), self.moves);
                        let curr_entry = &mut self.moves[best_index];
                        let curr_move = curr_entry.get_mv();
                        curr_entry.calc_see_score(&board);
                        if (curr_entry.get_see_score() < 0 && curr_entry.get_score() >= 0) {
                            curr_entry.set_score(-BASE_CAPTURE + (curr_move.get_captured_piece().unwrap().get_value() * 10 - curr_move.piece_to_move.get_value()));

                            continue;
                        }
                        if curr_entry.get_score() < 0 {
                            break;
                        }
                        let ret_entry = self.moves.pop_move(best_index);
                        if curr_move == self.tt_move {
                            continue;
                        }
                        return Some(ret_entry);
                    }
                    if self.only_captures {
                        self.curr_stage = DONE;
                    } else {
                        self.curr_stage = KillerMove1;
                    }
                }

                KillerMove1 => {
                    if self.killer_moves[0] == MoveData::default() || !board.is_legal_move(&self.killer_moves[0]) {
                        self.curr_stage = KillerMove2;
                        continue;
                    }
                    self.curr_stage = KillerMove2;
                    return Some(MoveEntry::entry_from_mv(self.killer_moves[0]));
                }
                KillerMove2 => {
                    if self.killer_moves[1] == MoveData::default() || !board.is_legal_move(&self.killer_moves[0]) {
                        self.curr_stage = GenerateQuiets;
                        continue;
                    }
                    self.curr_stage = GenerateQuiets;
                    return Some(MoveEntry::entry_from_mv(self.killer_moves[1]));
                }
                GenerateQuiets => {
                    let rem_cap_count = self.moves.len();
                    generate_moves_from_list(board, &mut self.moves, self.split, QuietsOnly);
                    self.quiet_size = (self.moves.len() - rem_cap_count) as i32;
                    for i in self.split..MAX_MOVES
                    {
                        let mv_entry = &mut self.moves[i];
                        if mv_entry.get_mv() == self.tt_move {
                            mv_entry.set_score(-INFINITY);
                            continue;
                        }
                        if mv_entry.get_mv() == MoveData::default() {
                            break;
                        }
                        let score = get_move_score(&mv_entry.get_mv(), self.ply, self.tt_move, &board, &refs);
                        mv_entry.set_score(score);
                    }
                    self.curr_stage = QuietMoves;
                }
                QuietMoves => {
                    while (self.quiet_size > 0)
                    {
                        let index = get_best_index(self.split, self.split + (self.quiet_size) as usize, self.moves);
                        let curr_entry = self.moves[index];
                        self.moves.pop_move(index);
                        self.quiet_size -= 1;
                        if curr_entry.get_mv() == self.tt_move {
                            continue;
                        }

                        return Some(curr_entry);
                    }
                    if self.only_captures {
                        self.curr_stage = DONE;
                    } else {
                        self.curr_stage = BadCaptures;
                    }
                }
                BadCaptures => {
                    if (self.moves.is_empty()) {
                        self.curr_stage = DONE;
                        continue;
                    }
                    let best_index = get_best_index(0, self.split, self.moves);
                    let curr_entry = self.moves[best_index];
                    self.moves.pop_move(best_index);


                    return Some(curr_entry);
                }
                DONE => {
                    return None;
                }
            }
        }
    }
}



