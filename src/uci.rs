use crate::board::board::Board;
use crate::constants::STARTPOS_FEN;
use crate::movegen::magic::precomputed::precompute_magics;
use crate::movegen::movedata::MoveData;
use crate::movegen::precomputed::precompute_movegen;
use crate::search::transposition_table::reset_transposition_table;


pub fn handle_command(command:&str,board: &mut Board)
{
    let first_word=command.split(" ").collect::<Vec<&str>>()[0];
    match first_word {
        "uci" => {
            println!("uciok");
        }
        "isready" => {
            println!("readyok");
        },
        "quit" => {
            std::process::exit(0);
        },
        "ucinewgame" => {
            precompute_magics();
            precompute_movegen();
            reset_transposition_table();



        },
        "d"=>{
            println!("{}", board.to_stockfish_string());

        },
        "position"=>{
            let remaining_string=command.split(" ").collect::<Vec<&str>>()[1..].join(" ");
            handle_position(remaining_string,board)
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
                let fen = parts[1..].join(" "); // Join the rest of the parts to form the full FEN string
                *board = Board::from_fen(&fen); // Set the board using the FEN string
                // If moves follow, apply them
                if parts.len() > 2 && parts[2] == "moves" {
                    let moves = parts[3..].to_vec(); // Collect all moves
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
    for curr_move in moves{
        board.make_move(&MoveData::from_algebraic(curr_move, board));
    }
}