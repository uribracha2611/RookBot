use crate::constants::FENS_FOR_BENCH;
use crate::engine::board::board::Board;
use crate::engine::board::piece::PieceColor::{BLACK, WHITE};
use crate::engine::movegen::movedata::MoveData;

use crate::engine::perft::perft_bulk;
use crate::engine::search::search::search;
use crate::engine::search::transposition_table::TranspositionTable;
use crate::engine::search::types::SearchInput;
use std::time::{Duration, Instant};

fn calculate_time_limit(time: Duration, inc: u64) -> Duration {
    if time.is_zero() {
        return Duration::from_millis(1);
    }

    let usable_ms = time.as_millis() as u64;

    let target_ms = (usable_ms / 20) + (inc / 2);
    let final_ms = target_ms.min(usable_ms).max(1);

    Duration::from_millis(final_ms)
}
const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn handle_command(
    command: &str,
    board: &mut Board,
    tt_table: &mut TranspositionTable,
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
            *tt_table = TranspositionTable::from_mb(64);
        }
        "d" => {
            println!("{}", board.to_stockfish_string());
        }
        "position" => {
            let remaining_string = command.split(" ").collect::<Vec<&str>>()[1..].join(" ");
            handle_position(remaining_string, board)
        }
        "go" => {
            handle_go(command, board, tt_table);
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

        "bench" => {
            bench();
        }
        _ => {
            println!("Unknown command: {}", command);
        }
    }
}

fn bench() {
    let mut total_time: Duration = Duration::from_millis(0);
    let mut total_nodes: u64 = 0;


    for pos in FENS_FOR_BENCH {
        let mut tt_table = TranspositionTable::from_mb(64);
        let mut board = Board::from_fen(pos);
        let now = Instant::now();
        let res = search(&mut board, &mut SearchInput::depth_input(7), &mut tt_table);
        total_time += now.elapsed();
        total_nodes += res.nodes_evaluated as u64;
    }

    // Calculate total time in seconds as a float.
    let total_time_seconds: f64 = total_time.as_millis() as f64 / 1000.0;

    // Calculate NPS as a float first for precision.
    let nps_float: f64 = total_nodes as f64 / total_time_seconds;

    // Cast the float result to a u64, which truncates the decimal.
    let nps_integer: u64 = nps_float as u64;

    println!("total nodes searched is {} time taken is {} ms nps is {}",
             total_nodes, total_time.as_millis(), nps_integer);
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
        board.make_move(move_from_algebric);
    }
}
pub fn handle_go(
    command: &str,
    board: &mut Board,
    tt_table: &mut TranspositionTable,
) {
    let mut depth = None;
    let mut movetime = None;
    let mut wtime = None;
    let mut btime = None;
    let mut winc = None;
    let mut binc = None;
    let mut nodes = None;


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
            "nodes" => {
                if i + 1 < parts.len() {
                    nodes = Some(parts[i + 1].parse::<u64>().unwrap());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }


    let mut search_input = if let Some(wtime) = wtime && board.turn == WHITE {
        let winc_val = winc.unwrap_or_default().as_millis() as u64;
        SearchInput::time_input(calculate_time_limit(wtime, winc_val))
    } else if let Some(btime) = btime && board.turn == BLACK {
        let binc_val = binc.unwrap_or_default().as_millis() as u64;
        SearchInput::time_input(calculate_time_limit(btime, binc_val))
    } else if let Some(movetime) = movetime {
        SearchInput::time_input(movetime)
    } else if let Some(depth) = depth {
        SearchInput::depth_input(depth as u8)
    } else if let Some(nodes) = nodes {
        SearchInput::node_count_input(nodes)
    } else {
        panic!("only depth, movetime, winc, binc and nodes supported so far for go command");
    };


    let time_test = Instant::now();


    let result = search(
        board,
        &mut search_input,
        tt_table,
    );


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
