use std::collections::HashSet;
use std::time::Instant;
use crate::board::board::Board;
use crate::perft::run_epd_file;
use crate::search::search::search;

pub mod board;
pub mod movegen;
mod search;
pub mod perft;


fn main() {
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    let start = Instant::now();
    let mv = search(&mut board, 6);
    let duration = start.elapsed();

    println!("Best move: {:?} and score is {:?}", mv.get_move(), mv.get_eval());
    println!("Search took: {:?}", duration.as_millis());
}