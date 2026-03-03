use crate::engine::board::board::Board;
use crate::engine::board::piece::PieceColor::WHITE;
use crate::engine::datagen::format::{BoardPacked, Game, PackedMove};
use crate::engine::movegen::generate::{generate_moves, update_check};
use crate::engine::search::constants::MATE_VALUE;
use crate::engine::search::psqt::weight::W;
use crate::engine::search::search::search;
use crate::engine::search::transposition_table::TranspositionTable;
use crate::engine::search::types::SearchInput;

const ACTUAL_MATE: i32 = MATE_VALUE - 100;
pub fn run_game(initial_board: &Board, node_count: u64) -> Game {
    let mut board = initial_board.clone();
    let mut moves: Vec<PackedMove> = Vec::new();
    let mut tb_white = TranspositionTable::from_mb(4);
    let mut tb_black = TranspositionTable::from_mb(4);

    loop {
        let mut search_limit = SearchInput::node_count_input(node_count);
        let curr_tt = if board.turn == WHITE {
            &mut tb_white
        } else {
            &mut tb_black
        };

        let result = search(&mut board, &mut search_limit, curr_tt);

        if result.principal_variation.is_empty() || result.eval.abs() >= ACTUAL_MATE {
            break;
        }

        let best_move = result.principal_variation[0];

        let to_white_side_factor = if board.turn == WHITE { 1 } else { -1 };
        let curr_move_packed = PackedMove::new(
            best_move.move_to_viri_format(board.turn),
            (result.eval as i16) * to_white_side_factor,
        );

        moves.push(curr_move_packed);
        board.make_move(best_move);

        if result.eval.abs() > 1000 {
            break;
        }

        if board.is_board_draw() {
            break;
        }
    }

    let final_eval = if moves.is_empty() {
        0
    } else {
        moves.last().unwrap().get_score() as i32
    };
    update_check(&mut board);
    let num_of_moves = generate_moves(&mut board, false).len();
    let wdl = if board.game_state.is_check && num_of_moves == 0 {
        if board.turn == WHITE { 0 } else { 2 }
    } else if board.is_board_draw() || num_of_moves == 0 {
        1
    } else if final_eval > 2000 {
        2
    } else if final_eval < -2000 {
        0
    } else {
        1
    };

    let game_initial_pos = BoardPacked::pack(initial_board, 0, wdl, 0);

    Game::new(game_initial_pos, moves)
}
