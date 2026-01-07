use crate::engine::movegen::constants::MAX_MOVES;
use crate::engine::movegen::movedata::MoveData;

#[derive(Copy, Clone)]
pub struct MoveList {
    moves: [Option<MoveData>; MAX_MOVES],
    count: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveList {
    pub fn iter(self) -> MoveListIterator {
        MoveListIterator {
            movelist: self,
            index: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.count
    }
    pub fn new() -> Self {
        MoveList {
            moves: [None; MAX_MOVES],
            count: 0,
        }
    }
    pub fn swap(&mut self, index1: usize, index2: usize) {
        if index1 < self.count && index2 < self.count {
            self.moves.swap(index1, index2);
        } else {
            panic!("Index out of bounds");
        }
    }

    pub fn add_move(&mut self, mv: MoveData) {
        if self.count < MAX_MOVES {
            self.moves[self.count] = Some(mv);
            self.count += 1;
        }
    }

    // Remove the '&' and just return MoveData
    pub fn get_move(&self, index: usize) -> MoveData {
        if index < self.count {
            // Since MoveData is Copy, this just clones the 16 bits
            self.moves[index].expect("Index within count but move was None")
        } else {
            panic!("Index {} out of bounds (count: {})", index, self.count);
        }
    }
    pub fn move_count(&self) -> usize {
        self.count
    }

    pub fn is_move_in_list(&self, mv: MoveData) -> bool {
        self.moves.iter().take(self.count).any(|m| m.unwrap() == mv)
    }
    pub fn find_move_by_start_end_square(self, from: u8, to: u8) -> Option<MoveData> {
        for i in 0..MAX_MOVES {
            let mv = self.moves[i].unwrap();
            if mv.from() == from && mv.to() == to {
                return Some(mv);
            }
        }
        None
    }
}

impl IntoIterator for MoveList {
    type Item = MoveData;
    type IntoIter = MoveListIterator;

    fn into_iter(self) -> Self::IntoIter {
        MoveListIterator {
            movelist: self,
            index: 0,
        }
    }
}

pub struct MoveListIterator {
    movelist: MoveList,
    index: usize,
}

impl Iterator for MoveListIterator {
    type Item = MoveData;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.movelist.count {
            // No need for & or unwrap logic here if get_move returns MoveData
            let result = self.movelist.get_move(self.index);
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}