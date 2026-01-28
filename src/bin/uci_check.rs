use std::fs::File;
use std::io::{BufRead, BufReader};
use RookBot::engine::board::board::Board;
use RookBot::engine::search::transposition_table::TranspositionTable;
use RookBot::uci::handle_command;

pub fn main() {
    run_uci_file("analyze_result.txt");
}

pub fn run_uci_file(file_name: &str) {
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let mut tt_table = TranspositionTable::from_mb(64);
    let lines = BufReader::new(File::open(file_name).expect("file not found")).lines();
    for line in lines.map_while(Result::ok) {
        handle_command(&line, &mut board, &mut tt_table);
    }
    handle_command("go depth 1", &mut board, &mut tt_table)
}
