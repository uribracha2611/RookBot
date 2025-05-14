use crate::engine::movegen::constants::MAX_MOVES;
use crate::engine::movegen::movedata::MoveData;

#[derive(Copy, Clone)]
pub struct MoveList {
    moves: [MoveData; MAX_MOVES],
    count: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveList {
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
            moves: [MoveData::defualt(); MAX_MOVES],
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
            self.moves[self.count] = mv;
            self.count += 1;
        }
    }

    pub fn get_move(&self, index: usize) -> &MoveData {
        if index < self.count {
            &self.moves[index]
        } else {
            panic!("Index out of bounds");
        }
    }

    pub fn move_count(&self) -> usize {
        self.count
    }

    pub fn is_move_in_list(&self, mv: &MoveData) -> bool {
        self.moves
            .iter()
            .take(self.count)
            .any(|m| *m == *mv)
    }
    pub fn find_move_by_start_end_square(self,from:u8,to:u8)->Option<MoveData>{
        for i in 0..MAX_MOVES{
            if let mv=self.moves[i]{
                if mv.from==from && mv.to==to{
                    return Some(mv);
                }
            }
        }
        None
    }
}

impl std::ops::Index<usize> for MoveList {
    type Output = MoveData;

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
    type Item = &'a MoveData;
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
    type Item = &'a MoveData;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.movelist.count {
            let result = self.movelist.get_move(self.index);
            self.index += 1;
            Option::from(result)
        } else {
            None
        }
    }
}

    

