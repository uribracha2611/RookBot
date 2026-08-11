pub const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
use RookBot::engine::movegen::magic::precomputed::precompute_magics;
use RookBot::engine::search::clock::TimeManager;
use RookBot::engine::search::constants::init_lmr;
use RookBot::engine::search::transposition_table::TranspositionTable;
use RookBot::uci::handle_command;
use RookBot::{engine::board::board::Board, uci::UciOption};
use mimalloc::MiMalloc;
use std::io::{self, BufRead};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
fn main() {
    precompute_magics();
    init_lmr();
    let stdin = io::stdin();
    let mut board = Board::from_fen(STARTPOS_FEN);
    let mut transposition_table = TranspositionTable::from_mb(64);

    let mut time_manager = TimeManager::default();
    let mut uci_option = UciOption::default();
    for command in stdin.lock().lines().flatten() {
        handle_command(
            &command,
            &mut board,
            &mut transposition_table,
            &mut time_manager,
            &mut uci_option,
        );
    }
}
