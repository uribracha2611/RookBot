use crate::board::board::Board;

pub mod board;
pub mod movegen;

#[cfg(test)]
pub mod board_tests;
mod perft;

use clap::{Command, Arg};

use crate::perft::perft;

fn main() {
    let matches = Command::new("Perft CLI")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Runs perft function with given depth and FEN string")
        .arg(
            Arg::new("depth")
                .help("Sets the depth for perft")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("fen")
                .help("Sets the FEN string for the board")
                .required(true)
                .index(2),
        )
        .get_matches();
    
    let depth = matches.get_one::<String>("depth").unwrap().parse::<u32>().expect("Invalid depth");
    let fen = matches.get_one::<String>("fen").unwrap();
    
    let mut board = Board::from_fen(fen);
    let s = perft(&mut board, depth);
    println!("{}", s);
    // let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    // let s = perft(&mut board, 3);
    // println!("{}", s);
    // 
}