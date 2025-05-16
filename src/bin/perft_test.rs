use std::fs::File;
use std::io::{BufRead, BufReader};
use RookBot::engine::board::board::Board;
use RookBot::engine::perft::{check_epd_line, perft, perft_bulk, run_epd_file};

fn parse_and_check_line(line: &str) {
    let mut parts = line.split(';');

    // First part is the FEN string
    let fen = parts.next().unwrap().trim();

    for part in parts {
        let part = part.trim();
        let mut board =Board::from_fen(fen);
        if let Some((depth_str, expected_str)) = part.strip_prefix('D').and_then(|s| s.split_once(' ')) {
            if let (Ok(depth), Ok(expected)) = (depth_str.parse::<u32>(), expected_str.parse::<u64>()) {
                let result = perft_bulk(&mut board, depth);
                if result != expected as u32 {
                    panic!("Mismatch at depth {}: expected {}, got {} for FEN: {}", depth, expected, result, fen);
                }
            }
        }
    }
}

fn main() {
    run_epd_file("standard.epd").unwrap();
    }

