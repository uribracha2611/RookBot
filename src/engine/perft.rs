pub fn perft(board: &mut Board, depth: u32) -> String {
    let mut result = String::new();
    let mut total_nodes = 0;

    let move_list = generate_moves(board, ALLMoves);
    for mv in move_list.iter() {
        board.make_move(&mv.get_mv());
        let nodes = perft_recursive(board, depth - 1);
        board.unmake_move(&mv.get_mv());

        result.push_str(&format!("{} {}\n", mv.get_mv().to_algebraic(), nodes));
        total_nodes += nodes;
    }

    result.push_str(&format!("\n{}", total_nodes));
    result
}

fn perft_recursive(board: &mut Board, depth: u32) -> u32 {
    if depth == 0 {
        return 1;
    }

    let move_list = generate_moves(board, ALLMoves);

    let mut nodes = 0;
    for mv in move_list.iter() {
        board.make_move(&mv.get_mv());
        nodes += perft_recursive(board, depth - 1);
        board.unmake_move(&mv.get_mv());
    }

    nodes
}
pub fn perft_bulk(board: &mut Board, depth: u32) -> u32 {
    let move_list = generate_moves(board, ALLMoves);
    if depth == 1 {
        return move_list.len() as u32;
    }
    let mut nodes = 0;

    for mv in move_list.iter() {
        board.make_move(&mv.get_mv());
        nodes += perft_bulk(board, depth - 1);
        board.unmake_move(&mv.get_mv());
    }

    nodes
}
pub fn perft_bulk_with_zobrist_check(
    board: &mut Board,
    curr_move: &mut Vec<MoveData>,
    depth: u32,
) -> u32 {
    assert_eq!(
        board.game_state.zobrist_hash,
        board.calc_zobrist(),
        "wrong zobrist hash board zobrist is {} and actual zobrist is {} and movelist are {:?}",
        board.game_state.zobrist_hash,
        board.calc_zobrist(),
        curr_move
    );
    let move_list = generate_moves(board, ALLMoves);
    if depth == 1 {
        return move_list.len() as u32;
    }
    let mut nodes = 0;

    for mv in move_list.iter() {
        board.make_move(&mv.get_mv());
        curr_move.push(mv.get_mv().clone());
        assert_eq!(
            board.game_state.zobrist_hash,
            board.calc_zobrist(),
            "wrong zobrist hash board zobrist is {} and actual zobrist is {} and movelist are {:?}",
            board.game_state.zobrist_hash,
            board.calc_zobrist(),
            curr_move
        );
        nodes += perft_bulk_with_zobrist_check(board, curr_move, depth - 1);
        curr_move.pop();
        board.unmake_move(&mv.get_mv());
        assert_eq!(
            board.game_state.zobrist_hash,
            board.calc_zobrist(),
            "wrong zobrist hash board zobrist is {} and actual zobrist is {} and movelist are {:?}",
            board.game_state.zobrist_hash,
            board.calc_zobrist(),
            curr_move
        );
    }

    nodes
}

use std::fs::File;
use std::time::Instant;

pub fn perft_with_timing(fen: &str, depth: u32) -> String {
    let mut board = Board::from_fen(fen);
    let start_time = Instant::now();
    let move_count = perft_bulk(&mut board, depth);
    let duration = start_time.elapsed().as_millis();

    format!(
        "time taken (in ms): {}, depth: {}, move count: {}",
        duration, depth, move_count
    )
}
use crate::engine::board::board::Board;
use crate::engine::movegen::generate::generate_moves;
use crate::engine::movegen::movedata::MoveData;
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::{io, panic};
use crate::engine::movegen::generate::MoveGenerationType::ALLMoves;

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

        let depth: u32 = depth_and_result[0][1..]
            .parse()
            .map_err(|_| "Invalid depth".to_string())?;
        let expected_result: u32 = depth_and_result[1]
            .trim_matches('"')
            .parse()
            .map_err(|_| "Invalid result".to_string())?;

        // Wrap the board in Arc<Mutex> to safely share it across the panic boundary
        let board_clone = Arc::clone(&board);

        let result = panic::catch_unwind(|| {
            let mut board_lock = board_clone
                .lock()
                .map_err(|_| "Mutex lock failed".to_string())?;
            let mut curr_move = Vec::new();
            let result = perft_bulk_with_zobrist_check(&mut board_lock, &mut curr_move, depth);
            if result != expected_result {
                return Err(format!(
                    "Mismatch for FEN: {} at depth {}: expected {}, got {}",
                    fen, depth, expected_result, result
                ));
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

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        match check_epd_line(&line) {
            Ok(_) => {
                println!("Line {}: OK", i + 1);
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e))?,
        }
    }

    Ok(())
}
