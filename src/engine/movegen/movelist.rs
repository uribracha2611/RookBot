use std::ops::{Deref, DerefMut};

use arrayvec::ArrayVec;

use crate::engine::movegen::constants::MAX_MOVES;
use crate::engine::movegen::movedata::MoveData;
#[derive(PartialEq)]
pub struct MoveListItem {
    mv: MoveData,
    score: i32,
}
impl MoveListItem {
    pub fn from_mv(mv: MoveData) -> Self {
        Self { mv, score: 0 }
    }
    pub fn set_score(&mut self, score: i32) {
        self.score = score;
    }
    pub fn get_score(&self) -> i32 {
        self.score
    }
    pub fn get_mv(&self) -> MoveData {
        self.mv
    }
}

pub struct MoveList {
    moves: ArrayVec<MoveListItem, MAX_MOVES>,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}
impl DerefMut for MoveList {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.moves
    }
}

impl MoveList {
    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    pub fn len(&self) -> usize {
        self.moves.len()
    }
    pub fn new() -> Self {
        MoveList {
            moves: ArrayVec::new(),
        }
    }
    pub fn iter_mv(&self) -> impl Iterator<Item = MoveData> {
        self.moves.iter().map(|data| data.mv)
    }

    pub fn swap(&mut self, index1: usize, index2: usize) {
        self.moves.swap(index1, index2);
    }

    pub fn add_move(&mut self, mv: MoveData) {
        self.moves.push(MoveListItem::from_mv(mv));
    }
}
impl Deref for MoveList {
    type Target = [MoveListItem];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.moves
    }
}
