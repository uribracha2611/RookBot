use std::fs::File;
use std::io;
use std::io::BufRead;
use crate::movegen::movelist::MoveList;
use crate::movegen::movedata::MoveData;
use crate::board::board::Board;
use crate::movegen::generate::generate_moves;

pub fn perft(board: &mut Board, depth: u32) -> String {

    let mut result = String::new();
    let mut total_nodes = 0;

    let move_list= generate_moves(board,false);
    for mv in move_list.iter(){

        board.make_move(mv);
        let nodes = perft_recursive(board, depth - 1);
        board.unmake_move(mv);

        result.push_str(&format!("{} {}\n", mv.to_algebraic(), nodes));
        total_nodes += nodes;
    }

    result.push_str(&format!("\n{}", total_nodes));
    result
}

fn perft_recursive(board: &mut Board, depth: u32) -> u32 {
    if depth == 0 {
        return 1;
    }

     let  move_list = generate_moves(board,false);

    let mut nodes = 0;
    for mv in move_list.iter() {
        board.make_move(mv);
        nodes += perft_recursive(board, depth - 1);
        board.unmake_move(mv);
    }

    nodes
}
pub fn perft_bulk(board: &mut Board, depth: u32) -> u32 {
    if depth == 0 {
        return 1;
    }

    let move_list = generate_moves(board,false);
    let mut nodes = 0;

    for mv in move_list.iter() {
        board.make_move(mv);
        nodes += perft_bulk(board, depth - 1);
        board.unmake_move(mv);
    }

    nodes
}
use std::time::Instant;

pub fn perft_with_timing(fen: &str, depth: u32) -> String {
    let mut board = Board::from_fen(fen);
    let start_time = Instant::now();
    let move_count = perft_bulk(&mut board, depth);
    let duration = start_time.elapsed().as_millis();

    format!("time taken (in ms): {}, depth: {}, move count: {}", duration, depth, move_count)
}
use std::panic;



use std::sync::{Arc, Mutex};

pub fn check_epd_line(line: &str) -> Result<(), String> {
    let parts: Vec<&str> = line.split(';').collect();
    if parts.len() < 2 {
        return Err("Invalid EPD line format".to_string());
    }

    let fen = parts[0].trim();
    let board = Arc::new(Mutex::new(Board::from_fen(fen)));

    for depth_and_result in parts.iter().skip(1) {
        let depth_and_result: Vec<&str> = depth_and_result.trim().split_whitespace().collect();
        if depth_and_result.len() < 2 {
            continue;
        }

        let depth: u32 = depth_and_result[0][1..].parse().map_err(|_| "Invalid depth".to_string())?;
        let expected_result: u32 = depth_and_result[1].trim_matches('"').parse().map_err(|_| "Invalid result".to_string())?;

        // Wrap the board in Arc<Mutex> to safely share it across the panic boundary
        let board_clone = Arc::clone(&board);

        let result = panic::catch_unwind(|| {
            let mut board_lock = board_clone.lock().map_err(|_| "Mutex lock failed".to_string())?;
            let result = perft_bulk(&mut board_lock, depth);
            if result != expected_result {
                return Err(format!("Mismatch for FEN: {} at depth {}: expected {}, got {}", fen, depth, expected_result, result));
            }
            Ok(())
        });

        match result {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                eprintln!("Panic occurred! Depth: {}, FEN: {}", depth, fen);
                return Err("Panic in perft_bulk".to_string());
            }
        }
    }

    Ok(())
}



pub fn run_epd_file(file_path: &str) -> Result<(), io::Error> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    for (i,line) in reader.lines().enumerate() {
        let line = line?;
        match check_epd_line(&line) {
            Ok(_) => {
                println!("Line {}: OK", i + 1);
            },
            Err(e) => {Err(io::Error::new(io::ErrorKind::InvalidData, e))?},
        }
    }

    Ok(())
}
