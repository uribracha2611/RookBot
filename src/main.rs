use crate::board::board::Board;

pub mod board;
pub mod movegen;

#[cfg(test)]
pub mod board_tests;


pub fn main() {
    let board=Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
}