use crate::engine::board::board::Board;
use crate::engine::board::piece::{Piece, PieceColor, PieceType};
use crate::engine::movegen::generate::{find_hidden_attackers, get_attackers_vec};
use crate::engine::movegen::movedata::MoveData;
use crate::engine::movegen::precomputed::DIR_SQUARES;

// Define piece values
pub const PAWN_VALUE: i32 = 100;
pub const KNIGHT_VALUE: i32 = 300;
pub const BISHOP_VALUE: i32 = 300;
pub const ROOK_VALUE: i32 = 500;
pub const QUEEN_VALUE: i32 = 900;
pub const KING_VALUE: i32 = 20000;

// Generate attackers for a given square

pub fn static_exchange_evaluation(board: &Board, curr_mv: &MoveData) -> i32 {
    let MoveData {
        to: capture_square,
        piece_to_move: piece_captures,
        from: initial_square,
        ..
    } = *curr_mv;

    let mut scores = Vec::new();
    let mut turn = board.turn;
    let initial_turn = turn;
    let mut curr_turn_attackers = get_attackers_vec(board, capture_square, turn);
    let mut opp_attackers = get_attackers_vec(board, capture_square, turn.opposite());
    curr_turn_attackers
        .retain(|x| (x.0.piece_type != piece_captures.piece_type || x.1 != initial_square));

    let score = match curr_mv.get_captured_piece() {
        None => 0,
        Some(p) => get_piece_value(p.piece_type),
    };
    scores.push(score);

    if piece_captures.piece_type != PieceType::KING
        && piece_captures.piece_type != PieceType::KNIGHT
    {
        let delta = DIR_SQUARES[initial_square as usize][capture_square as usize];

        let hidden_attacker = find_hidden_attackers(board, delta, initial_square);
        if hidden_attacker.is_some() {
            add_piece_to_attack(
                hidden_attacker.unwrap(),
                &mut curr_turn_attackers,
                &mut opp_attackers,
                initial_turn,
                (piece_captures, initial_square as u8),
            );
        }
    }
    let mut curr_piece_at_square = piece_captures;
    turn = turn.opposite();
    let mut score_index = 1;
    while (!curr_turn_attackers.is_empty() && turn == initial_turn)
        || (!opp_attackers.is_empty() && turn != initial_turn)
    {
        let new_piece_captures = if initial_turn == turn {
            curr_turn_attackers.remove(0)
        } else {
            opp_attackers.remove(0)
        };
        let capture_value =
            get_piece_value(curr_piece_at_square.piece_type) - scores[score_index - 1];
        scores.push(capture_value);
        curr_piece_at_square = new_piece_captures.0;

        if new_piece_captures.0.piece_type != PieceType::KING
            && new_piece_captures.0.piece_type != PieceType::KNIGHT
        {
            let delta = DIR_SQUARES[new_piece_captures.1 as usize][capture_square as usize];

            let hidden_attacker = find_hidden_attackers(board, delta, new_piece_captures.1);
            if hidden_attacker.is_some() {
                add_piece_to_attack(
                    hidden_attacker.unwrap(),
                    &mut curr_turn_attackers,
                    &mut opp_attackers,
                    initial_turn,
                    (piece_captures, initial_square as u8),
                );
            }
        }

        score_index += 1;
        turn = turn.opposite();
    }
    while score_index > 1 {
        score_index -= 1;
        if scores[score_index - 1] > -scores[score_index] {
            scores[score_index - 1] = -scores[score_index];
        }
    }
    scores[0]
}
pub fn get_piece_value(piece: PieceType) -> i32 {
    match piece {
        PieceType::PAWN => PAWN_VALUE,
        PieceType::KNIGHT => KNIGHT_VALUE,
        PieceType::BISHOP => BISHOP_VALUE,
        PieceType::ROOK => ROOK_VALUE,
        PieceType::QUEEN => QUEEN_VALUE,
        PieceType::KING => KING_VALUE,
    }
}
fn add_piece_to_attack(
    hidden_attacker: (Piece, u8),
    our_attackers: &mut Vec<(Piece, u8)>,
    their_attackers: &mut Vec<(Piece, u8)>,
    initial_turn: PieceColor,
    initial_piece: (Piece, u8),
) {
    if hidden_attacker == initial_piece {
        return;
    }
    if hidden_attacker.0.piece_color == initial_turn {
        if !our_attackers.contains(&hidden_attacker) {
            our_attackers.push(hidden_attacker);
            our_attackers.sort_by_key(|x| get_piece_value(x.0.piece_type));
        }
    } else if !their_attackers.contains(&hidden_attacker) {
        their_attackers.push(hidden_attacker);
        their_attackers.sort_by_key(|x| get_piece_value(x.0.piece_type));
    }
}
