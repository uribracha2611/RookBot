use crate::engine::board::board::Board;
use crate::engine::board::piece::PieceColor;
use crate::engine::movegen::magic::precomputed::precompute_magics;
use crate::engine::movegen::movedata::MoveData;
use crate::engine::movegen::precomputed::precompute_movegen;
use crate::engine::perft::perft_bulk;
use crate::engine::search::search::{search, timed_search};
use crate::engine::search::transposition_table::TranspositionTable;
use crate::engine::search::types::SearchInput;
use crate::opening_book::{get_move_from_opening_book, init_book};
use std::time::Duration;

const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn handle_command(
    command: &str,
    board: &mut Board,
    should_use_book: bool,
    mut tt_table: &mut TranspositionTable,
) {
    let first_word = command.split(" ").collect::<Vec<&str>>()[0];
    match first_word {
        "uci" => {
            println!("uciok");
        }
        "isready" => {
            println!("readyok");
        }
        "quit" => {
            std::process::exit(0);
        }
        "ucinewgame" => {
            precompute_magics();
            precompute_movegen();
            init_book();
            *tt_table = TranspositionTable::from_mb(64);
            return;
        }
        "d" => {
            println!("{}", board.to_stockfish_string());
        }
        "position" => {
            let remaining_string = command.split(" ").collect::<Vec<&str>>()[1..].join(" ");
            handle_position(remaining_string, board)
        }
        "go" => {
            handle_go(command, board, should_use_book, tt_table);
        }
        "perft" => {
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.len() > 1 {
                if let Ok(depth) = parts[1].parse::<u32>() {
                    let start_time = std::time::Instant::now();
                    let result = perft_bulk(board, depth);
                    println!("{}", result);
                    println!("Perft took {} ms", start_time.elapsed().as_millis());
                } else {
                    eprintln!("Invalid depth for perft command");
                }
            } else {
                eprintln!("Depth not specified for perft command");
            }
        }

        _ => {
            println!("Unknown command: {}", command);
        }
    }
}

fn handle_position(command: String, board: &mut Board) {
    let parts: Vec<&str> = command.split(" ").collect();
    match parts[0] {
        // If the command is "startpos", set the board to the starting position.
        "startpos" => {
            *board = Board::from_fen(STARTPOS_FEN); // Assuming STARTPOS_FEN is a constant for the standard starting position
                                                    // If moves follow, apply them
            if parts.len() > 1 && parts[1] == "moves" {
                let moves = parts[2..].to_vec(); // Collect all moves
                apply_moves(board, &moves); // Assuming you have a function to apply moves
            }
        }

        // If the command is "fen", extract the FEN from the command and set the board.
        "fen" => {
            if parts.len() > 1 {
                let fen = parts[1..7].join(" "); // Join the rest of the parts to form the full FEN string
                *board = Board::from_fen(&fen); // Set the board using the FEN string
                                                // If moves follow, apply them
                if parts.len() > 2 && parts.get(7) == Some(&"moves") {
                    let moves = parts[8..].to_vec(); // Collect all moves
                    apply_moves(board, &moves); // Apply the moves on top of the FEN
                }
            } else {
                // Handle invalid command if FEN part is missing
                eprintln!("Error: FEN string is missing in the 'fen' command.");
            }
        }

        // Handle any other cases, potentially printing an error or logging invalid command.
        _ => {
            eprintln!("Error: Unknown command: {}", parts[0]);
        }
    }
}

fn apply_moves(board: &mut Board, moves: &Vec<&str>) {
    for curr_move in moves {
        let move_from_algebric = MoveData::from_algebraic(curr_move, board);
        board.make_move(&move_from_algebric);
    }
}
pub fn handle_go(
    command: &str,
    board: &mut Board,
    should_use_book: bool,
    tt_table: &mut TranspositionTable,
) {
    let mut depth = None;
    let mut movetime = None;
    let mut wtime = None;
    let mut btime = None;
    let mut winc = None;
    let mut binc = None;
    if should_use_book {
        if let Some(mv) = (get_move_from_opening_book(board)) {
            println!("info string Book move {} score cp 0", mv.to_algebraic());
            println!("bestmove {}", mv.to_algebraic());
            return;
        }
    }

    let parts: Vec<&str> = command.split_whitespace().collect();
    let mut i = 1; // Skip the "go" part

    while i < parts.len() {
        match parts[i] {
            "depth" => {
                if i + 1 < parts.len() {
                    depth = Some(parts[i + 1].parse::<u32>().unwrap());
                    i += 1;
                }
            }
            "movetime" => {
                if i + 1 < parts.len() {
                    movetime = Some(Duration::from_millis(parts[i + 1].parse::<u64>().unwrap()));
                    i += 1;
                }
            }
            "wtime" => {
                if i + 1 < parts.len() {
                    wtime = Some(Duration::from_millis(parts[i + 1].parse::<u64>().unwrap()));
                    i += 1;
                }
            }
            "btime" => {
                if i + 1 < parts.len() {
                    btime = Some(Duration::from_millis(parts[i + 1].parse::<u64>().unwrap()));
                    i += 1;
                }
            }
            "winc" => {
                if i + 1 < parts.len() {
                    winc = Some(Duration::from_millis(parts[i + 1].parse::<u64>().unwrap()));
                    i += 1;
                }
            }
            "binc" => {
                if i + 1 < parts.len() {
                    binc = Some(Duration::from_millis(parts[i + 1].parse::<u64>().unwrap()));
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let time_limit = if let Some(move_time) = movetime {
        move_time
    } else {
        match board.turn {
            PieceColor::WHITE => wtime.unwrap_or(Duration::from_secs(60)),
            PieceColor::BLACK => btime.unwrap_or(Duration::from_secs(60)),
        }
    };
    let increment = match board.turn {
        PieceColor::WHITE => winc.unwrap_or(Duration::from_secs(0)),
        PieceColor::BLACK => binc.unwrap_or(Duration::from_secs(0)),
    };

    let mut board_clone = board.clone();
    let time_test = std::time::Instant::now();
    let result = if movetime.is_some() || wtime.is_some() || btime.is_some() {
        timed_search(
            &mut board_clone,
            time_limit,
            increment,
            movetime.is_some(),
            tt_table,
        )
    } else {
        let search_depth = depth.unwrap();
        search(
            board,
            &mut SearchInput {
                depth: search_depth as u8,
            },
            tt_table,
        )
    };
    let pv = result
        .principal_variation
        .iter()
        .map(|x| x.to_algebraic())
        .collect::<Vec<String>>()
        .join(" ");
    let best_move = result.principal_variation.first().unwrap();
    let score = result.eval;
    let depth = result.depth;
    let nodes = result.nodes_evaluated;
    println!(
        "info depth {} nodes {} score cp {} pv {} time {}",
        depth,
        nodes,
        score,
        pv,
        time_test.elapsed().as_millis()
    );
    println!("bestmove {}", best_move.to_algebraic());
}
