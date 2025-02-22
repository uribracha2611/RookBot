use std::sync::{LazyLock, Mutex};
use crate::movegen::movedata::MoveData;
use crate::search::constants::MATE_VALUE;

const MB_SIZE: usize = 128 * 1024 * 1024; // 64 MB

#[derive(Clone, Copy,Eq, PartialEq)]
pub(crate) enum EntryType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy,Eq, PartialEq)]
pub(crate) struct Entry {
    pub(crate) hash: u64,
    pub(crate) depth: u8,
    pub(crate) eval: i32,
    pub(crate) entry_type: EntryType,
    pub(crate) best_move: MoveData,
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

    pub fn store(&mut self, hash: u64, depth: u8, eval: i32, entry_type: EntryType, best_move: MoveData) {
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
        if let Some(entry) = self.table[index] {
            if entry.hash == hash && entry.depth >= depth {
                match entry.entry_type {
                    EntryType::Exact => return Some(entry),
                    EntryType::LowerBound => {
                        if entry.eval >= beta {
                            return Some(entry);
                        }
                    }
                    EntryType::UpperBound => {
                        if entry.eval <= alpha {
                            return Some(entry);
                        }
                    }
                }
            }
        }
        None
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
pub fn reset_transposition_table() {
    let mut table = TRANSPOSITION_TABLE.lock().unwrap();
    *table = TranspositionTable::new(MB_SIZE / std::mem::size_of::<Entry>());
}

pub static TRANSPOSITION_TABLE: LazyLock<Mutex<TranspositionTable>> = LazyLock::new(|| {
    Mutex::new(TranspositionTable::new(MB_SIZE / std::mem::size_of::<Entry>()))
});

pub fn setup_transposition_table() {
    LazyLock::force(&TRANSPOSITION_TABLE);
}
pub fn is_mate_score(score: i32) -> bool {
    score.abs() > MATE_VALUE
}
pub fn correct_mate_score_for_storage(score: i32,ply:i32) -> i32 {
    if(is_mate_score(score)) {
        let sign = score.signum();
      (score * sign + ply) * sign
    }
    else {
        score
    }
}
pub fn correct_mate_score_for_display(score: i32,ply:i32) -> i32 {
    if(is_mate_score(score)) {
        let sign = score.signum();
        (score * sign - ply) * sign
    }
    else {
        score
    }
}