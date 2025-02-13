use std::collections::HashSet;
use std::time::Instant;
use crate::board::board::Board;
use crate::movegen::generate::generate_moves;
use crate::perft::run_epd_file;
use crate::search::search::search;
use crate::search::transposition_table::setup_transposition_table;
use crate::search::types::SearchInput;

pub mod board;
pub mod movegen;
mod search;
pub mod perft;


fn main() {
    setup_transposition_table();
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    let start = Instant::now();

    let result = search(&mut board, SearchInput { depth:8});
    let duration = start.elapsed();

    let principal_variation: Vec<String> = result.get_principal_variation().iter().map(|mv| mv.to_algebraic()).collect();
    let pv_string = principal_variation.join(" ");

    println!("info depth {} nodes {} time {} score {}  pv {}",
            8,
             result.get_nodes_evaluated(),
             duration.as_millis(),
        result.eval,
             pv_string);
    for mv in result.get_principal_variation() {
        let move_list = generate_moves(&mut board);
        if !move_list.is_move_in_list(mv) {
            panic!("Illegal move found in PV: {}", mv.to_algebraic());
        }
        board.make_move(mv);
    }
}
