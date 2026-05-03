use RookBot::engine::board::board::Board;
use RookBot::engine::search::clock::TimeManager;
use RookBot::engine::search::transposition_table::TranspositionTable;
use RookBot::uci::handle_go;

pub fn main() {
    let str = "6R1/8/7P/3k4/8/5N2/5K2/8 w - - 1 82";
    let mut tt_table = TranspositionTable::from_mb(64);
    let mut board = Board::from_fen(str);

    let mut time_manager = TimeManager::default();
    handle_go("go depth 11", &mut board, &mut tt_table, &mut time_manager)
}

