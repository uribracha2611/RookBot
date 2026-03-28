use RookBot::engine::board;
use RookBot::engine::board::board::Board;
use RookBot::engine::board::piece::PieceColor::{BLACK, WHITE};
use RookBot::engine::datagen::functions::run_game;
use RookBot::engine::movegen::generate::{generate_moves, update_check};
use RookBot::engine::search::clock::{ClockOption, TimeManager};
use RookBot::engine::search::search::search;
use RookBot::engine::search::transposition_table::TranspositionTable;
use RookBot::engine::search::types::SearchInput;
use clap::Parser;
use rand::prelude::*;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Error, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct DatagenArgs {
    #[arg(short, long, default_value_t = 5000)]
    node_count: u64,
    #[arg(short, long, value_name = "FILE")]
    book_path: String,
    #[arg(short, long)]
    game_count: i32,
    #[arg(short, long)]
    threads: usize,
}

struct ThreadResult {
    positions_generated: u64,
}

fn main() -> Result<(), Error> {
    let args = DatagenArgs::parse();
    let fens = Arc::new(parse_epdfile(&args.book_path)?);
    let games_per_thread = args.game_count / args.threads as i32;
    let mut extra_games = args.game_count % args.threads as i32;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let session_dir = format!("DATA/session_{}", timestamp);
    fs::create_dir_all(&session_dir)?;

    let total_games_finished = Arc::new(Mutex::new(0));
    let start_time = Instant::now();
    let mut handles = vec![];

    println!(
        "Starting Rookbot Datagen: {} threads, {} nodes/move",
        args.threads, args.node_count
    );
    println!("Output directory: {}", session_dir);

    let total_game_count = args.game_count;
    let checkpoint_interval = 1000.min(args.game_count / 10);

    for t_id in 0..args.threads {
        let mut thread_games = games_per_thread;
        if extra_games > 0 {
            thread_games += 1;
            extra_games -= 1;
        }
        let fens_ref = Arc::clone(&fens);
        let progress = Arc::clone(&total_games_finished);
        let node_limit = args.node_count;
        let thread_dir = session_dir.clone();

        let handle = thread::spawn(move || {
            let mut thread_pos_count = 0;
            let file_path = format!("{}/thread_{}.binpack", thread_dir, t_id);
            let file = File::create(file_path).unwrap();
            let mut writer = BufWriter::new(file);

            let mut i = 1;
            while i <= thread_games {
                let mut board = choose_random_opening(&fens_ref, 5, node_limit);

                let mut wrapper = std::panic::AssertUnwindSafe(&mut board);
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                    run_game(*wrapper, node_limit)
                }));

                match result {
                    Ok(game) => {
                        thread_pos_count += game.get_moves().len() as u64;
                        let _ = game.write_to_bin(&mut writer);

                        let mut count = progress.lock().unwrap();
                        *count += 1;
                        let new_count = *count;

                        if new_count % checkpoint_interval == 0 || new_count == total_game_count {
                            println!(
                                "Progress: {}/{} games finished...",
                                new_count, total_game_count
                            );
                        }
                        i += 1;
                    }
                    Err(_) => {
                        let moves = generate_moves(&mut board, false);
                        eprintln!(
                            "the value of moves is_empty is {} the amount of moves are {} and the fen is {}",
                            moves.is_empty(),
                            moves.len(),
                            board.to_fen()
                        );
                        eprintln!("Thread {} recovered from a panic. Retrying game...", t_id);
                        continue;
                    }
                }
            }

            let _ = writer.flush();
            ThreadResult {
                positions_generated: thread_pos_count,
            }
        });
        handles.push(handle);
    }

    let mut total_positions = 0;
    for handle in handles {
        if let Ok(res) = handle.join() {
            total_positions += res.positions_generated;
        }
    }

    merge_files(&session_dir, args.threads)?;

    let duration = start_time.elapsed();
    println!("\n--- Datagen Report ---");
    println!("Total Time:        {:.2?}", duration);
    println!("Total Positions:   {}", total_positions);
    println!("Final File:        {}/finaldata.bin", session_dir);
    println!("----------------------");

    Ok(())
}

fn merge_files(dir: &str, num_threads: usize) -> Result<(), Error> {
    println!("Merging thread files into finaldata.bin...");
    let final_path = format!("{}/finaldata.bin", dir);
    let mut final_file = File::create(final_path)?;

    for t_id in 0..num_threads {
        let thread_file_path = format!("{}/thread_{}.binpack", dir, t_id);
        if let Ok(mut thread_file) = File::open(&thread_file_path) {
            std::io::copy(&mut thread_file, &mut final_file)?;
        }
    }
    Ok(())
}

pub fn parse_epdfile(path: &String) -> Result<Vec<String>, Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

pub fn choose_random_fen(fens: &[String]) -> String {
    let mut rng = rand::rng();
    fens.choose(&mut rng).expect("Book is empty").clone()
}

pub fn choose_random_opening(fens: &[String], random_move_count: i32, node_limit: u64) -> Board {
    let mut rng = rand::rng();
    let mut basic_tt = TranspositionTable::from_mb(1);

    loop {
        let fen = choose_random_fen(fens) + " 0 0";
        let mut board = Board::from_fen(&fen);
        let mut failed = false;

        for _ in 0..random_move_count {
            update_check(&mut board);
            let moves = generate_moves(&mut board, false);

            if moves.is_empty() {
                failed = true;
                break;
            }

            let random_move = match moves.into_iter().choose(&mut rng) {
                Some(m) => m,
                None => {
                    failed = true;
                    break;
                }
            };
            board.make_move(random_move);
        }
        update_check(&mut board);
        let moves = generate_moves(&mut board, false);

        if moves.is_empty() {
            failed = true;
        }
        if failed {
            continue;
        }
        let mut time_management = TimeManager::default();
        time_management.set_clock(ClockOption::from_nodes(node_limit * 2));
        let result = search(&mut board, &mut basic_tt, &time_management);

        if result.eval.abs() < 400 {
            return board;
        }
    }
}
