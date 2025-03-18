use std::cmp::Reverse;
use std::collections::BinaryHeap;
use crate::board::board::Board;
use crate::board::piece::{Piece, PieceColor, PieceType};
use crate::movegen::generate::{find_hidden_attackers, get_attackers_vec};
use crate::movegen::movedata::MoveData;
use crate::movegen::precomputed::DIR_SQUARES;
use crate::search::move_ordering::get_capture_score_only;

// Define piece values
pub(crate) const PAWN_VALUE: i32 = 100;
pub(crate) const KNIGHT_VALUE: i32 = 320;
pub(crate) const BISHOP_VALUE: i32 = 330;
pub(crate) const ROOK_VALUE: i32 = 500;
pub(crate) const QUEEN_VALUE: i32 = 900;
pub(crate) const KING_VALUE: i32 = 20000;


// Generate attackers for a given square


pub fn static_exchange_evaluation(board: &Board,capture_square:i32,piece_captured:Piece,piece_captures:Piece,initial_square:i32)->i32{

    let mut scores =Vec::new();
    let mut turn =board.turn;
    let initial_turn=turn;
    let mut curr_turn_attackers =get_attackers_vec(board, capture_square as u8, turn);
    let mut opp_attackers =get_attackers_vec(board, capture_square as u8, turn.opposite());
    curr_turn_attackers.retain(|x| (x.0.piece_type!=piece_captures.piece_type || x.1!= initial_square as u8));

    let score=get_piece_value(piece_captured.piece_type);

    scores.push(score);

    if(piece_captures.piece_type!=PieceType::KING && piece_captures.piece_type!=PieceType::KNIGHT) {
        let delta = DIR_SQUARES[initial_square as usize][capture_square as usize];

        let hidden_attacker = find_hidden_attackers(board, delta, capture_square);
        if hidden_attacker.is_some()
        {
            add_piece_to_attack(hidden_attacker.unwrap(),&mut curr_turn_attackers,&mut opp_attackers,initial_turn);

        }
    }
    let mut curr_piece_at_square =piece_captures;
    turn=turn.opposite();
    let mut score_index =1;
    while (!curr_turn_attackers.is_empty() && turn==initial_turn) || (!opp_attackers.is_empty() && turn!=initial_turn)  {

        let piece_captures=if initial_turn==turn{
            curr_turn_attackers.remove(0)
        }
        else {
            opp_attackers.remove(0)
        };
        let capture_value=get_piece_value(curr_piece_at_square.piece_type)-scores[score_index-1];
        scores.push(capture_value);
        curr_piece_at_square=piece_captures.0;


        if(piece_captures.0.piece_type!=PieceType::KING && piece_captures.0.piece_type!=PieceType::KNIGHT) {
            let delta = DIR_SQUARES[initial_square as usize][piece_captures.1 as usize];

            let hidden_attacker = find_hidden_attackers(board, delta, capture_square);
            if hidden_attacker.is_some()
            {
                add_piece_to_attack(hidden_attacker.unwrap(), &mut curr_turn_attackers, &mut opp_attackers, initial_turn);
            }
        }

        score_index+=1;
        turn=turn.opposite();


    }
    while score_index>1
    {
        score_index-=1;
        if(scores[score_index-1]>-scores[score_index]){
            scores[score_index-1]=-scores[score_index];
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
fn add_piece_to_attack(hidden_attacker:(Piece,u8),our_attackers:&mut Vec<(Piece,u8)>,their_attackers:&mut Vec<(Piece,u8)>,initial_turn:PieceColor){
    if hidden_attacker.0.piece_color==initial_turn{
        our_attackers.push(hidden_attacker);
        our_attackers.sort_by_key(|x| get_piece_value(x.0.piece_type));
    }
    else{
        their_attackers.push(hidden_attacker);
        their_attackers.sort_by_key(|x| get_piece_value(x.0.piece_type));
    }
}