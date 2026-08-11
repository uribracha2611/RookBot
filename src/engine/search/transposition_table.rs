use crate::engine::movegen::movedata::MoveData;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum EntryType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct InsertEntry {
    pub hash: u64,
    pub depth: u8,
    pub eval: i16,
    pub static_eval: i16,
    pub entry_type: EntryType,
    pub best_move: Option<MoveData>,
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ResultEntry {
    pub hash: u64,
    pub depth: u8,
    pub eval: i32,
    pub static_eval: i32,
    pub entry_type: EntryType,
    pub best_move: Option<MoveData>,
}

pub struct TranspositionTable {
    table: Vec<Option<InsertEntry>>,
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        TranspositionTable {
            table: vec![None; size],
        }
    }
    pub fn clear(&mut self) {
        self.table.fill(None);
    }
    pub fn from_mb(mb_size: usize) -> Self {
        let bytes = mb_size * 1024 * 1024;
        let max_entries = bytes / std::mem::size_of::<InsertEntry>();
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
        static_eval: i32,
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

        self.table[index] = Some(InsertEntry {
            hash,
            depth,
            eval: eval as i16,
            static_eval: static_eval as i16,
            entry_type,
            best_move: move_to_insert,
        });
    }
    pub fn retrieve(&self, hash: u64) -> Option<ResultEntry> {
        let index = (hash & ((self.table.len() - 1) as u64)) as usize;

        let entry = self.table[index]?;

        if entry.hash != hash {
            return None;
        }

        Some(ResultEntry {
            hash: entry.hash,
            depth: entry.depth,
            eval: entry.eval as i32,
            static_eval: entry.static_eval as i32,
            entry_type: entry.entry_type,
            best_move: entry.best_move,
        })
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
    pub fn prefetch(&self, hash: u64) {
        let index = (hash & ((self.table.len() - 1) as u64)) as usize;
        unsafe {
            use core::arch::x86_64::{_MM_HINT_T0, _mm_prefetch};
            _mm_prefetch(self.table.as_ptr().add(index) as *const i8, _MM_HINT_T0);
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn prefetch(&self, hash: u64) {
        let index = (hash & ((self.table.len() - 1) as u64)) as usize;
        unsafe {
            use core::arch::aarch64::__prefetch;
            __prefetch(self.table.as_ptr().add(index) as *const i8);
        }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    pub fn prefetch(&self, _hash: u64) {}
}
