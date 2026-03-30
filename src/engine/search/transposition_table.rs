use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::constants::MATE_VALUE;

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
    pub best_move: Option<MoveData>,
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
        let bytes = mb_size * 1024 * 1024;
        let max_entries = bytes / std::mem::size_of::<Entry>();
        let mut size = max_entries.next_power_of_two();
        if size > max_entries && size > 1 {
            size /= 2;
        }
        TranspositionTable::new(size)
    }

    pub fn store(
        &mut self,
        hash: u64,
        depth: u8,
        eval: i32,
        entry_type: EntryType,
        best_move: Option<MoveData>,
    ) {
        let mut move_to_insert = best_move;
        let index = (hash & ((self.table.len() - 1) as u64)) as usize;
        if let Some(curr_entry) = self.table[index]
            && move_to_insert.is_none()
            && curr_entry.hash == hash
        {
            move_to_insert = curr_entry.best_move;
        }

        self.table[index] = Some(Entry {
            hash,
            depth,
            eval,
            entry_type,
            best_move: move_to_insert,
        });
    }

    pub fn retrieve(&self, hash: u64) -> Option<Entry> {
        let index = (hash & ((self.table.len() - 1) as u64)) as usize;

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

    pub fn get_tt_move(&self, hash: u64) -> Option<MoveData> {
        let index = (hash & ((self.table.len() - 1) as u64)) as usize;
        if let Some(entry) = self.table[index]
            && entry.hash == hash
        {
            return entry.best_move;
        }
        None
    }
}
