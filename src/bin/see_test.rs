use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};
use RookBot::engine::board::board::Board;
use RookBot::engine::board::see::static_exchange_evaluation;
use RookBot::engine::movegen::movedata::MoveData;
use RookBot::engine::search::transposition_table::TranspositionTable;
use RookBot::uci::handle_command;

pub fn main() -> std::io::Result<()> {
    let mut counter = 0;
    let mut total_time = Duration::new(0, 0);
    let file = File::open("src/SEE.txt")?;
    let reader = BufReader::new(file);
    for l in reader.lines() {
        let line = l?;
        handle_single_line(line, &mut total_time, &mut counter);
    }

    println!(
        "Elapsed: {} microseconds  and position tested is {}",
        total_time.as_micros(), counter
    );
    Ok(())
}


pub fn handle_single_line(line: String, duration: &mut Duration, counter: &mut i32) {
    let parts = line.split('|').collect::<Vec<&str>>();
    let fen = parts[0].trim().to_owned() + " 0 0";
    let move_text = parts[1].trim();
    let see_expected_result = parts[2].trim().parse::<i32>().unwrap();
    let mut board = Board::from_fen(&fen);
    let curr_move = MoveData::from_algebraic(move_text, &board);
    handle_command("ucinewgame", &mut board, &mut TranspositionTable::from_mb(10));

    if curr_move.is_promotion() {
        return;
    }
    *counter += 1;
    let now = Instant::now();

    let see_result = static_exchange_evaluation(&board, curr_move);


    *duration += now.elapsed();
    assert_eq!(
        see_expected_result, see_result,
        "sse result is {} expected result is {}  fen is {} and move is {:?} ",
        see_result, see_expected_result, fen, curr_move
    );
}
