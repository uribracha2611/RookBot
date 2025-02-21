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
mod uci;
pub mod constants;

use std::io::{self, BufRead};

use crate::uci::handle_command;

fn main() {
    let stdin = io::stdin();
    let mut board = Board::from_fen(constants::STARTPOS_FEN);


    for line in stdin.lock().lines() {
        if let Ok(command) = line {
            handle_command(&command, &mut board);
        }
    }
}

