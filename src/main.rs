use std::collections::HashSet;
use crate::board::board::Board;
use crate::perft::run_epd_file;

pub mod board;
pub mod movegen;
mod search;
pub mod perft;

fn main() {
    let test=run_epd_file("src/standard.epd");
    if test.is_err(){
        println!("Error running EPD file: {:?}",test.err());
    }
    else{
        println!("EPD file ran successfully");
    }


}