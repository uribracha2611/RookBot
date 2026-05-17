use crate::engine::{
    board::{board::Board, piece::PieceColor},
    datagen::format::{BoardPacked, Game, PackedMove},
    movegen::generate::{generate_moves, update_check},
    search::{
        clock::{self, TimeManager},
        search::search,
        transposition_table::TranspositionTable,
    },
};

pub fn run_game(
    initial_board: &mut Board,
    node_count: u64,
    tb_white: &mut TranspositionTable,
    tb_black: &mut TranspositionTable,
) -> Game {
    let mut win_adj_count = 0;
    let mut draw_adj_count = 0;
    let mut is_white_adj = true;
    let mut wdl: u8 = 1;
    let mut moves: Vec<PackedMove> = Vec::new();
    let mut time_manager = TimeManager::default();
    let mut board = initial_board.clone();

    tb_white.clear();
    tb_black.clear();

    time_manager.set_clock(clock::ClockOption::NODES(node_count));
    loop {
        let curr_tt = if board.turn == PieceColor::WHITE {
            &mut *tb_white
        } else {
            &mut *tb_black
        };

        if board.is_board_draw() {
            wdl = 1;
            break;
        }
        if win_adj_count >= 6 {
            let color = if is_white_adj {
                PieceColor::WHITE
            } else {
                PieceColor::BLACK
            };
            wdl = compute_wdl_for_win(color);
            break;
        }
        if draw_adj_count >= 16 && moves.len() >= 40 {
            wdl = 1;
            break;
        }
        update_check(&mut board);
        let no_moves = generate_moves(&mut board, false).is_empty();
        if no_moves {
            if board.game_state.is_check {
                wdl = compute_wdl_for_win(board.turn.opposite());
            } else {
                wdl = 1;
            }
            break;
        }
        let result = search(&mut board, curr_tt, &time_manager);

        if result.principal_variation.is_empty() {
            break;
        }

        if result.eval.abs() > 1000 {
            let color = if result.eval > 1000 {
                board.turn
            } else {
                board.turn.opposite()
            };
            wdl = compute_wdl_for_win(color);
        }
        let best_move = result.principal_variation[0];

        let to_white_side_factor = if board.turn == PieceColor::WHITE {
            1
        } else {
            -1
        };
        let curr_move_packed = PackedMove::new(
            best_move.move_to_viri_format(board.turn),
            (result.eval as i16) * to_white_side_factor,
        );

        moves.push(curr_move_packed);
        board.make_move(best_move);

        if result.eval.abs() <= 10 {
            draw_adj_count += 1;
        } else {
            draw_adj_count = 0;
        }

        if result.eval.abs() >= 1000 {
            win_adj_count += 1;
            is_white_adj = result.eval >= 1000;
        } else {
            win_adj_count = 0;
            is_white_adj = false;
        }
    }

    update_check(&mut board);
    let num_of_moves = generate_moves(&mut board, false).len();
    wdl = if board.game_state.is_check && num_of_moves == 0 {
        compute_wdl_for_win(board.turn.opposite())
    } else if board.is_board_draw() || num_of_moves == 0 {
        1
    } else {
        wdl
    };

    let game_initial_pos = BoardPacked::pack(initial_board, 0, wdl, 0);
    Game::new(game_initial_pos, moves)
}
#[inline(always)]
pub fn compute_wdl_for_win(color: PieceColor) -> u8 {
    if color == PieceColor::WHITE { 2 } else { 0 }
}
