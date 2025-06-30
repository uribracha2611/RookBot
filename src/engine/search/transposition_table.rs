use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::constants::MATE_VALUE;
use std::sync::{LazyLock, Mutex};

const MB_SIZE: usize = 128 * 1024 * 1024; // 128 MB

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum EntryType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Entry {
    pub hash: u64,
    pub depth: u8,
    pub eval: i32,
    pub entry_type: EntryType,
    pub best_move: MoveData,
}

pub struct TranspositionTable {
    table: Vec<Option<Entry>>,
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        TranspositionTable {
            table: vec![None; size],
        }
    }
    pub fn from_mb(mb_size: usize) -> Self {
        TranspositionTable::new(mb_size * 1024 * 1024 / std::mem::size_of::<Entry>())
    }

    pub fn store(
        &mut self,
        hash: u64,
        depth: u8,
        eval: i32,
        entry_type: EntryType,
        best_move: MoveData,
    ) {
        let index = (hash as usize) % self.table.len();

        if let Some(existing_entry) = &self.table[index] {
            if existing_entry.depth >= depth {
                return; // Do not store if the existing entry has a greater or equal depth
            }
        }
        self.table[index] = Some(Entry {
            hash,
            depth,
            eval,
            entry_type,
            best_move,
        });
    }

    pub fn retrieve(&self, hash: u64, depth: u8, alpha: i32, beta: i32) -> Option<Entry> {
        let index = (hash as usize) % self.table.len();
        let elem = self.table[index];
        unsafe {
            if elem.is_some() {
                if elem.unwrap_unchecked().hash == hash {
                    elem
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    pub fn get_TT_move(&self, hash: u64) -> Option<MoveData> {
        let index = (hash as usize) % self.table.len();
        if let Some(entry) = self.table[index] {
            if entry.hash == hash {
                return Some(entry.best_move);
            }
        }
        None
    }
}

pub fn is_mate_score(score: i32) -> bool {
    score.abs() > MATE_VALUE
}
pub fn correct_mate_score_for_storage(score: i32, ply: i32) -> i32 {
    if is_mate_score(score) {
        let sign = score.signum();
        (score * sign + ply) * sign
    } else {
        score
    }
}
pub fn correct_mate_score_for_display(score: i32, ply: i32) -> i32 {
    if is_mate_score(score) {
        let sign = score.signum();
        (score * sign - ply) * sign
    } else {
        score
    }
}
