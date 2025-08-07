use crate::engine::board::board::Board;
use crate::engine::board::see::static_exchange_evaluation;
use crate::engine::movegen::constants::MAX_MOVES;
use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::constants::INFINITY;

#[derive(Copy, Clone)]
pub struct MoveEntry {
    mv: MoveData,
    see_score: i32,
    score: i32,
}
impl Default for MoveEntry {
    fn default() -> Self {
        MoveEntry { see_score: 0, mv: MoveData::default(), score: 0 }
    }
}
impl MoveEntry {
    pub fn get_mv(&self) -> MoveData
    {
        self.mv
    }
    pub fn get_score(&self) -> i32 {
        self.score
    }

    pub fn entry_from_mv(mv: MoveData) -> MoveEntry {
        MoveEntry { mv, see_score: 0, score: 0 }
    }
    pub fn calc_see_score(&mut self, board: &Board) {
        self.see_score = static_exchange_evaluation(board, &self.mv);
    }
    pub fn set_see_score(&mut self, see_score: i32) {
        self.see_score = see_score;
    }
    pub fn set_score(&mut self, score: i32) {
        self.score = score;
    }
    pub fn get_see_score(&self) -> i32 {
        self.see_score
    }
    pub fn default() -> MoveEntry {
        MoveEntry { mv: MoveData::default(), see_score: -INFINITY, score: -INFINITY }
    }
}
#[derive(Copy, Clone)]
pub struct MoveList {
    moves: [MoveEntry; MAX_MOVES],
    count: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveList {
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn iter(&self) -> MoveListIterator {
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
            moves: [MoveEntry::default(); MAX_MOVES],
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

    pub fn add_move(&mut self, mv: MoveData, index: &mut usize) {
        if self.count < MAX_MOVES {
            self.moves[*index] = MoveEntry::entry_from_mv(mv);
            *index += 1;
            self.count += 1;
        }
    }


    pub fn get_move(&self, index: usize) -> &MoveEntry {
        &self.moves[index]
    }

    pub fn pop_move(&mut self, index: usize) -> MoveEntry {
        let ret_move = self.moves[index];
        self.count -= 1;
        self.moves[index] = self.moves[self.count];
        ret_move
    }

    pub fn move_count(&self) -> usize {
        self.count
    }

    pub fn is_move_in_list(&self, mv: &MoveData) -> bool {
        self.moves.iter().take(self.count).any(|mv_entry| mv_entry.mv == *mv)
    }
    pub fn find_move_by_start_end_square(self, from: u8, to: u8) -> Option<MoveData> {
        for i in 0..MAX_MOVES {
            if let mv_entry = self.moves[i] {
                if mv_entry.mv.from == from && mv_entry.mv.to == to {
                    return Some(mv_entry.mv);
                }
            }
        }
        None
    }
}

impl std::ops::Index<usize> for MoveList {
    type Output = MoveEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.moves[index]
    }
}

impl std::ops::IndexMut<usize> for MoveList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.moves[index]
    }
}
impl<'a> IntoIterator for &'a MoveList {
    type Item = &'a MoveEntry;
    type IntoIter = MoveListIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MoveListIterator {
            movelist: self,
            index: 0,
        }
    }
}

pub struct MoveListIterator<'a> {
    movelist: &'a MoveList,
    index: usize,
}

impl<'a> Iterator for MoveListIterator<'a> {
    type Item = &'a MoveEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < MAX_MOVES {
            let result = self.movelist.get_move(self.index);
            self.index += 1;
            Option::from(result)
        } else {
            None
        }
    }
}
