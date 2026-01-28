use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use RookBot::engine::board::board::Board;
use RookBot::engine::search::search::eval;


fn main() -> io::Result<()> {
    let file = File::open("processed_eval.txt")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        handle_line(&line);
    }

    Ok(())
}
pub fn handle_line(line: &str) {
    let line_part = line.split(",").collect::<Vec<&str>>();
    let fen = line_part[0];
    let corr_eval = line_part[1].parse::<i32>().unwrap();
    let board = Board::from_fen(fen);
    let code_eval = eval(&board);
    if corr_eval != code_eval  {
        println!("correct eval: {} rookbot eval {}", corr_eval, code_eval);
    }
}