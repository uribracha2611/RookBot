use std::{ffi::os_str::Display, fmt};

use arrayvec::ArrayVec;

use crate::engine::{
    board::{
        board::Board,
        piece::PieceType,
        see::{get_piece_value, static_exchange_evaluation},
    },
    movegen::{
        constants::MAX_MOVES,
        generate::{GENTYPE, generate_moves},
        movedata::MoveData,
        movelist::{MoveList, MoveListItem},
    },
    search::{
        move_ordering::{BASE_CAPTURE, capture_formula, get_capture_scores, get_moves_score},
        types::SearchRefs,
    },
};

const BEST_CAP_FORMULA: i32 = 90; // (mvv-lva of captured queen by king quuen is value 9 king is
// 0 since it can't be captured so calculation is 10 * captured piece-piece that captures=9*10-0=90)
const MIN_BAD_CAP_VALUE: i32 = -BASE_CAPTURE + BEST_CAP_FORMULA;
#[derive(PartialEq)]
pub enum MovegenStages {
    TtMove,
    GenerateCaptures,
    GoodCap,
    KillerMoves,
    GenerateQuiets,
    QuietMoves,
    BadCap,
    Done,
}
impl fmt::Display for MovegenStages {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            MovegenStages::TtMove => "TtMove",
            MovegenStages::GenerateCaptures => "GenerateCaptures",
            MovegenStages::GoodCap => "GoodCap",
            MovegenStages::KillerMoves => "KillerMoves",
            MovegenStages::GenerateQuiets => "GenerateQuiets",
            MovegenStages::QuietMoves => "QuietMoves",
            MovegenStages::BadCap => "BadCap",
            MovegenStages::Done => "Done",
        };

        f.write_str(name)
    }
}
pub struct MovePicker {
    index: usize,
    ply: usize,
    skip_quiets: bool,
    moves: MoveList,
    pub stage: MovegenStages,
    tt_move: Option<MoveData>,
    seen_moves: ArrayVec<MoveData, MAX_MOVES>,
    killer_index: usize,
}
fn select_move(moves: &[MoveListItem]) -> Option<usize> {
    moves
        .iter()
        .enumerate()
        .max_by_key(|(_, mv_data)| mv_data.get_score())
        .map(|(i, _)| i)
}

impl MovePicker {
    pub fn get_move(&mut self, board: &Board, refs: &SearchRefs) -> Option<&MoveListItem> {
        let index = loop {
            let best_relative_index = select_move(&self.moves[self.index..])?;
            let best_index = self.index + best_relative_index;

            let score = self.moves[best_index].get_score();
            let mv = self.moves[best_index].get_mv();

            if score >= BASE_CAPTURE && static_exchange_evaluation(board, mv) < 0 {
                let new_score = -BASE_CAPTURE + capture_formula(board, mv);
                self.moves[best_index].set_score(new_score);
                continue;
            }

            let current_index = self.index;

            self.moves.swap(current_index, best_index);
            self.index += 1;

            let score = self.moves[current_index].get_score();
            let mv = self.moves[current_index].get_mv();

            if self.skip_quiets && score < BASE_CAPTURE {
                return None;
            }

            if self.tt_move != Some(mv) && !refs.killer_moves[self.ply].contains(&Some(mv)) {
                break current_index;
            }
        };

        debug_assert!(
            !self.seen_moves.contains(&self.moves[index].get_mv()),
            "mv from is {} mv to is {} is it a capture {} stage is {} score is {}",
            self.moves[index].get_mv().from(),
            self.moves[index].get_mv().to(),
            self.moves[index].get_mv().is_capture(),
            self.stage,
            self.moves[index].get_score()
        );

        Some(&self.moves[index])
    }

    pub fn new(ply: usize, skip_quiets: bool, tt_move: Option<MoveData>) -> MovePicker {
        MovePicker {
            moves: MoveList::default(),
            index: 0,
            ply,
            seen_moves: ArrayVec::new(),
            skip_quiets,
            stage: MovegenStages::TtMove,
            tt_move,
            killer_index: 0,
        }
    }
    pub fn next(&mut self, board: &mut Board, refs: &SearchRefs) -> Option<MoveData> {
        if self.stage == MovegenStages::TtMove {
            self.stage = MovegenStages::GenerateCaptures;
            if let Some(tt_move) = self.tt_move
                && board.is_move_legal(tt_move)
            {
                self.seen_moves.push(tt_move);
                #[cfg(debug_assertions)]
                {
                    let mut temp_moves = MoveList::new();
                    generate_moves(board, GENTYPE::AllMoves, &mut temp_moves);
                    if !temp_moves.contains(&MoveListItem::from_mv(tt_move)) {
                        eprintln!(
                            "is legal thinks tt_move is legal when it's illegal from: {},to: {}, is_capture: {},board fen: {}",
                            tt_move.from(),
                            tt_move.to(),
                            board.to_fen(),
                            tt_move.is_capture()
                        );
                        debug_assert!(false)
                    }
                }

                return self.tt_move;
            }
            #[cfg(debug_assertions)]
            {
                if let Some(tt_move) = self.tt_move {
                    let mut temp_moves = MoveList::new();
                    generate_moves(board, GENTYPE::AllMoves, &mut temp_moves);
                    if temp_moves.contains(&MoveListItem::from_mv(tt_move)) {
                        eprintln!(
                            "is legal thinks tt_move is ilegal when it's legal from: {},to: {}, is_capture: {},board fen: {}",
                            tt_move.from(),
                            tt_move.to(),
                            board.to_fen(),
                            tt_move.is_capture()
                        );
                        debug_assert!(false)
                    }
                }
            }
        }

        if self.stage == MovegenStages::GenerateCaptures {
            generate_moves(board, super::generate::GENTYPE::CapOnly, &mut self.moves);

            #[cfg(debug_assertions)]
            for mv in self.moves.iter_mv() {
                if !board.is_move_legal(mv) {
                    eprintln!(
                        "is legal is wrong about capture move : from={:?}, to={:?} board fen={}",
                        mv.from(),
                        mv.to(),
                        board.to_fen(),
                    );

                    debug_assert!(false);
                }
            }

            let move_count = self.moves.len();
            get_capture_scores(&mut self.moves, board, 0, move_count);
            self.stage = MovegenStages::GoodCap
        }
        if self.stage == MovegenStages::GoodCap {
            if let Some(mv_data) = self.get_move(board, refs) {
                debug_assert!(mv_data.get_mv().is_capture());
                let mv = mv_data.get_mv();
                let score = mv_data.get_score();

                if score >= BASE_CAPTURE {
                    #[cfg(debug_assertions)]
                    self.seen_moves.push(mv);

                    return Some(mv);
                }
                self.index -= 1;
            }

            self.stage = if self.skip_quiets {
                MovegenStages::Done
            } else {
                MovegenStages::KillerMoves
            }
        }

        if self.stage == MovegenStages::KillerMoves {
            if !self.skip_quiets {
                while self.killer_index < 2 {
                    //we can assume there are no duplicate killer moves see StoreKillerMoves for
                    //more details
                    if let Some(curr_killer) = refs.killer_moves[self.ply][self.killer_index]
                        && Some(curr_killer) != self.tt_move
                        && board.is_move_legal(curr_killer)
                    {
                        debug_assert!(!self.seen_moves.contains(&curr_killer));
                        self.seen_moves.push(curr_killer);
                        self.killer_index += 1;
                        return Some(curr_killer);
                    }
                    self.killer_index += 1;
                }
            }
            self.stage = if self.skip_quiets {
                MovegenStages::Done
            } else {
                MovegenStages::GenerateQuiets
            }
        }
        if self.stage == MovegenStages::GenerateQuiets {
            if self.skip_quiets {
                self.stage = MovegenStages::Done;
            } else {
                self.stage = MovegenStages::QuietMoves;
                let curr_end = self.moves.len();
                generate_moves(board, super::generate::GENTYPE::QuietOnly, &mut self.moves);
                let new_end = self.moves.len();
                #[cfg(debug_assertions)]
                for mv in self.moves.iter().skip(curr_end).take(new_end - curr_end) {
                    if !board.is_move_legal(mv.get_mv()) {
                        eprintln!(
                            "is legal is wrong about quiet move : from={:?}, to={:?} board fen={}",
                            mv.get_mv().from(),
                            mv.get_mv().to(),
                            board.to_fen(),
                        );

                        debug_assert!(false);
                    }
                    if mv.get_mv().is_capture() {
                        eprintln!(
                            "Unexpected capture at quiet move gen: from={:?}, to={:?}, score={} board fen={}, start of quiet moves is {} end of quiet moves is {}",
                            mv.get_mv().from(),
                            mv.get_mv().to(),
                            mv.get_score(),
                            board.to_fen(),
                            curr_end,
                            new_end
                        );

                        debug_assert!(false);
                    }
                }
                get_moves_score(&mut self.moves, 0, board, refs, curr_end, new_end);
            }
        }
        if self.stage == MovegenStages::QuietMoves {
            if !self.skip_quiets
                && let Some(mv_data) = self.get_move(board, refs)
                && mv_data.get_score() > MIN_BAD_CAP_VALUE
            {
                let mv = mv_data.get_mv();
                let score = mv_data.get_score();
                #[cfg(debug_assertions)]
                if mv.is_capture() {
                    eprintln!(
                        "Unexpected capture: from={:?}, to={:?}, score={} board fen={}",
                        mv.from(),
                        mv.to(),
                        score,
                        board.to_fen()
                    );

                    debug_assert!(false);
                }

                self.seen_moves.push(mv);
                return Some(mv);
            }
            self.stage = if self.skip_quiets {
                MovegenStages::Done
            } else {
                MovegenStages::BadCap
            }
        }
        if self.stage == MovegenStages::BadCap {
            if let Some(mv_data) = self.get_move(board, refs) {
                let mv = mv_data.get_mv();

                self.seen_moves.push(mv);
                debug_assert!(mv.is_capture());
                return Some(mv);
            }
            self.stage = MovegenStages::Done;
        }

        None
    }
}
