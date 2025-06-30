use std::env;
use std::process;
use RookBot::engine::board::board::Board;
use RookBot::engine::movegen::movedata::MoveData;
use RookBot::engine::perft::perft;

fn main() {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();

    // Validate arguments
    if args.len() < 3 {
        eprintln!("Usage: rookbot.exe <depth> <fen> [moves...]");
        process::exit(1);
    }

    // Parse depth
    let depth: u32 = args[1].parse().unwrap_or_else(|_| {
        eprintln!("Invalid depth: {}", args[1]);
        process::exit(1);
    });

    // Parse FEN
    let fen = &args[2];
    let mut board = Board::from_fen(fen);

    // Parse and apply moves (if provided)
    if args.len() > 3 {
        for move_str in &args[3..] {
            let mv = MoveData::from_algebraic(move_str, &board);
            board.make_move(&mv);
        }
    }

    // Perform perft calculation
    let result = perft(&mut board, depth);

    // Print the result
    println!("{}", result);
}
