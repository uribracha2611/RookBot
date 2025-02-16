use std::collections::HashSet;
use std::time::Instant;
use crate::board::board::Board;
use crate::movegen::generate::generate_moves;
use crate::movegen::magic::precomputed::precompute_magics;
use crate::movegen::precomputed::precompute_movegen;
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
    precompute_movegen();
    precompute_magics();
    
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
");
    let search_input=SearchInput { depth:8};
    
    let start = Instant::now();
    
    let result = search(&mut board, &search_input);
    let duration = start.elapsed();
    
    let principal_variation: Vec<String> = result.get_principal_variation().iter().map(|mv| mv.to_algebraic()).collect();
    let pv_string = principal_variation.join(" ");
    let nps = result.get_nodes_evaluated() as f64 / duration.as_secs_f64();

    println!("info depth {} nodes {} time {} nps {} score {}  pv {}",
             search_input.depth ,
             result.get_nodes_evaluated(),
             duration.as_millis(),
        nps as i32,
        result.eval,
             pv_string);
    for mv in result.get_principal_variation() {
        let move_list = generate_moves(&mut board, false);
        if !move_list.is_move_in_list(mv) {
            panic!("Illegal move found in PV: {}", mv.to_algebraic());
        }
        board.make_move(mv);
    }
    // let r=run_epd_file("src/standard.epd");
    // if r.is_err(){
    //     println!("Error running epd file, error: {}",r.err().unwrap());
    // 
    // }
    // else {
    //     print!("Success running epd file");
    // }

}
