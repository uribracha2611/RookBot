use std::cmp::PartialEq;
use std::ops::{BitAnd, Not};
use crate::board::bitboard::Bitboard;
use crate::board::board::Board;
use crate::board::castling::types::{AllowedCastling, CastlingSide};
use crate::board::piece::{Piece, PieceColor, PieceType};
use crate::movegen::constants::{RANK_1, RANK_8};
use crate::movegen::magic::functions::{get_bishop_attacks, get_rook_attacks};
use crate::movegen::movedata::{CastlingMove, MoveData, MoveType, PromotionCapture};
use crate::movegen::movedata::MoveType::Castling;
use crate::movegen::movelist::MoveList;


pub fn generate_moves(board: &Board) -> MoveList {
    let mut move_list = MoveList::new();
    generate_knight_move(board, &mut move_list);
    generate_king_move(board, &mut move_list);
    generate_pawn_moves(board, &mut move_list);
    get_rook_moves(board, &mut move_list);
    get_bishop_moves(board, &mut move_list);
    get_queen_moves(board, &mut move_list);
    move_list
}

pub fn generate_knight_move(board: &Board, move_list: &mut MoveList) {
    let   knights = &mut board.get_piece_bitboard(board.turn, PieceType::KNIGHT);
    let opp_pieces = board.get_color_bitboard(board.turn.opposite());
    let blockers=board.get_color_bitboard(board.turn);
    while *knights != 0  {
        let from_sqr=knights.pop_lsb();
        let mut moves=crate::movegen::precomputed::KNIGHT_MOVES[from_sqr as usize] & !blockers;
        let mut captures= moves & opp_pieces;
        moves &= !board.get_color_bitboard(board.turn);
        while moves!=0 {
            let to_sqr = moves.pop_lsb();
            let curr_move = MoveData::new(from_sqr, to_sqr, board.squares[from_sqr as usize].unwrap(), MoveType::Normal);
            move_list.add_move(curr_move);
        }
        while captures!=0 {
            let to_sqr = captures.pop_lsb();
            let curr_move = MoveData::new(from_sqr, to_sqr, board.squares[from_sqr as usize].unwrap(), MoveType::Capture(board.squares[to_sqr as usize].unwrap()));
            move_list.add_move(curr_move);
        }
        
    }
   
}
pub fn generate_king_move(board: &Board, move_list: &mut MoveList) {
    let kings = &mut board.get_piece_bitboard(board.turn, PieceType::KING);
    let opp_pieces = board.get_color_bitboard(board.turn.opposite());
    let blockers = board.get_color_bitboard(board.turn);
        let from_sqr = kings.pop_lsb();
        let mut moves = crate::movegen::precomputed::KING_MOVES[from_sqr as usize] & !blockers;
        let mut captures = moves & opp_pieces;
        moves &= !board.get_color_bitboard(board.turn);
        while moves != 0 {
            let to_sqr = moves.pop_lsb();
            let curr_move = MoveData::new(from_sqr, to_sqr, board.squares[from_sqr as usize].unwrap(), MoveType::Normal);
            move_list.add_move(curr_move);
        }
        while captures != 0 {
            let to_sqr = captures.pop_lsb();
            let curr_move = MoveData::new(from_sqr, to_sqr, board.squares[from_sqr as usize].unwrap(), MoveType::Capture(board.squares[to_sqr as usize].unwrap()));
            move_list.add_move(curr_move);
        }
    let mut castling_options=[CastlingSide::Kingside, CastlingSide::Queenside];
    let castling_rights = if board.turn == PieceColor::WHITE {
        board.game_state.castle_white
    } else {
        board.game_state.castle_black
    };
    
    for side in castling_options.iter(){
        if castling_rights.is_allowed(side) && !board.is_check &&
            (board.get_all_pieces_bitboard() & side.required_empty(board.turn)==0 )
        {
            let king_start=side.king_start(board.turn);
            let king_end=side.king_end(board.turn);
            let castle_move=MoveData::new(king_start, king_end, board.squares[king_start as usize].unwrap(), MoveType::Castling(CastlingMove::new(*side, board.turn)));
            move_list.add_move(castle_move);
            
        } 
        
    }

}
fn get_promotion_bitboard(pawns:&Bitboard, color: PieceColor) -> Bitboard {
    if color == PieceColor::WHITE { RANK_8} else { RANK_1 }

}
fn get_pawn_dir(color: PieceColor) -> i8 {
    if color == PieceColor::WHITE { 1 } else { -1 }
}
fn get_pawn_attack_dir(color: PieceColor, left: bool) -> i8 {
    if color == PieceColor::WHITE {
        if left { 7 } else { 9 }
    } else {
        if left { -9 } else { -7 }
    }
}
pub fn generate_pawn_moves(board: &Board, move_list: &mut MoveList) {
    let pawns = &mut board.get_piece_bitboard(board.turn, PieceType::PAWN);
    let opp_pieces = board.get_color_bitboard(board.turn.opposite());
    let blockers = board.get_all_pieces_bitboard();
    let promotion_bitboard = get_promotion_bitboard(pawns, board.turn);

    let mut double_pushes = pawns.pawn_double_push(&board.turn, blockers & opp_pieces);
    let mut single_pushes = pawns.pawn_push(&board.turn) & !blockers & !opp_pieces;
    let mut single_pushes_promote = single_pushes & promotion_bitboard;
    single_pushes &= !promotion_bitboard;
    while single_pushes != 0
    {
        let end_sq = single_pushes.pop_lsb();
        let start_sq = (end_sq as i8 - (8 * get_pawn_dir(board.turn))) as u8;
        let curr_move = MoveData::new(start_sq, end_sq, board.squares[start_sq as usize].unwrap(), MoveType::Normal);
    }
    while single_pushes_promote != 0
    {
        let end_sq = single_pushes_promote.pop_lsb();
        let start_sq = (end_sq as i8 - (8 * get_pawn_dir(board.turn))) as u8;
        generate_promote_moves(board,start_sq, end_sq, move_list, board.turn);
    }
    while double_pushes != 0
    {
        let end_sq = double_pushes.pop_lsb();
        let start_sq = (end_sq as i8 - (16 * get_pawn_dir(board.turn))) as u8;
        let curr_move = MoveData::new(start_sq, end_sq,board.squares[start_sq as usize].unwrap() , MoveType::Normal);
        move_list.add_move(curr_move);
    }
    let is_left: [bool; 2] = [true, false];
    for left in is_left.iter() {
        let mut attacks = pawns.pawn_attack(board.turn, opp_pieces, *left);
        let mut promote_attacks = attacks & promotion_bitboard;
        attacks &= !promotion_bitboard;
        while attacks != 0
        {
            let end_sq = attacks.pop_lsb();
            let start_sq = (end_sq as i8 - (get_pawn_attack_dir(board.turn, *left))) as u8;
            let curr_move = MoveData::new(start_sq, end_sq, board.squares[start_sq as usize].unwrap(), MoveType::Capture(board.squares[end_sq as usize].unwrap()));
            move_list.add_move(curr_move);
        }
        while promote_attacks != 0
        {
            let end_sq = promote_attacks.pop_lsb();
            let start_sq = (end_sq as i8 - (get_pawn_attack_dir(board.turn, *left))) as u8;
            generate_promote_captures(board, start_sq, end_sq, move_list, board.turn, board.squares[end_sq as usize].unwrap());
        }
    }
    if let Some(en_passant_square) = board.game_state.en_passant_square
    {
        let en_passant_target = (en_passant_square as i8- (8 * get_pawn_dir(board.turn) ) ) as u8;
      let mut pawns_can_capture =pawns.pawn_attack(board.turn.opposite(), Bitboard::new(en_passant_target as u64), true) | pawns.pawn_attack(board.turn.opposite(), Bitboard::new(en_passant_target as u64), false);
        while pawns_can_capture != 0
        {
            let start_sq = pawns_can_capture.pop_lsb();
            let curr_move = MoveData::new(start_sq, en_passant_target, board.squares[start_sq as usize].unwrap(), MoveType::EnPassant(board.squares[en_passant_square as usize].unwrap(),en_passant_square));
            move_list.add_move(curr_move);
            
        }
    }
}
fn generate_promote_moves(board: &Board,start_square:u8, end_square:u8, move_list: &mut MoveList, color: PieceColor) {
    let promote_pieces = [PieceType::QUEEN, PieceType::ROOK, PieceType::BISHOP, PieceType::KNIGHT];
    for piece in promote_pieces.iter() {
        let curr_move = MoveData::new(start_square, end_square, board.squares[start_square as usize].unwrap(), MoveType::Promotion(Piece::new(color, *piece)));
        move_list.add_move(curr_move);
    }
}
pub fn generate_promote_captures(board:&Board,start_square:u8, end_square:u8, move_list: &mut MoveList, color: PieceColor, captured_piece: Piece) {
    let promote_pieces = [PieceType::QUEEN, PieceType::ROOK, PieceType::BISHOP, PieceType::KNIGHT];
    for piece in promote_pieces.iter() {
        let curr_move = MoveData::new(start_square, end_square, board.squares[start_square as usize].unwrap() , MoveType::PromotionCapture(PromotionCapture { captured_piece, promoted_piece: Piece::new(color, *piece) }));
        move_list.add_move(curr_move);
    }
}
pub fn get_rook_moves(board: &Board, move_list: &mut MoveList) {
    let rooks = &mut board.get_piece_bitboard(board.turn, PieceType::ROOK);
    let opp_pieces = board.get_color_bitboard(board.turn.opposite());
    let blockers = board.get_all_pieces_bitboard();
    while *rooks != 0 {
        let from_sqr = rooks.pop_lsb();
        let mut moves = get_rook_attacks(from_sqr as usize, blockers);
        let mut captures = moves & opp_pieces;
        moves &= !captures;
        while moves != 0 {
            let to_sqr = moves.pop_lsb();
            let curr_move = MoveData::new(from_sqr, to_sqr, board.squares[from_sqr as usize].unwrap(), MoveType::Normal);
            move_list.add_move(curr_move);
        }
        while captures != 0 {
            let to_sqr = captures.pop_lsb();
            let curr_move = MoveData::new(from_sqr, to_sqr, board.squares[from_sqr as usize].unwrap(), MoveType::Capture(board.squares[to_sqr as usize].unwrap()));
            move_list.add_move(curr_move);
        }
    }
}
pub fn get_bishop_moves(board: &Board, move_list: &mut MoveList) {
    let bishops = &mut board.get_piece_bitboard(board.turn, PieceType::BISHOP);
    let opp_pieces = board.get_color_bitboard(board.turn.opposite());
    let blockers = board.get_all_pieces_bitboard();
    while *bishops != 0 {
        let from_sqr = bishops.pop_lsb();
        let mut moves = get_bishop_attacks(from_sqr as usize, blockers);
        let mut captures = moves & opp_pieces;
        moves &= !captures;
        while moves != 0 {
            let to_sqr = moves.pop_lsb();
            let curr_move = MoveData::new(from_sqr, to_sqr, board.squares[from_sqr as usize].unwrap(), MoveType::Normal);
            move_list.add_move(curr_move);
        }
        while captures != 0 {
            let to_sqr = captures.pop_lsb();
            let curr_move = MoveData::new(from_sqr, to_sqr, board.squares[from_sqr as usize].unwrap(), MoveType::Capture(board.squares[to_sqr as usize].unwrap()));
            move_list.add_move(curr_move);
        }
    }
}
pub fn get_queen_moves(board: &Board, move_list: &mut MoveList) {
    let queens = &mut board.get_piece_bitboard(board.turn, PieceType::QUEEN);
    let opp_pieces = board.get_color_bitboard(board.turn.opposite());
    let blockers = board.get_all_pieces_bitboard();
    while *queens != 0 {
        let from_sqr = queens.pop_lsb();
        let mut moves = get_bishop_attacks(from_sqr as usize, blockers) | get_rook_attacks(from_sqr as usize, blockers);
        let mut captures = moves & opp_pieces;
        moves &= !captures;
        while moves != 0 {
            let to_sqr = moves.pop_lsb();
            let curr_move = MoveData::new(from_sqr, to_sqr, board.squares[from_sqr as usize].unwrap(), MoveType::Normal);
            move_list.add_move(curr_move);
        }
        while captures != 0 {
            let to_sqr = captures.pop_lsb();
            let curr_move = MoveData::new(from_sqr, to_sqr, board.squares[from_sqr as usize].unwrap(), MoveType::Capture(board.squares[to_sqr as usize].unwrap()));
            move_list.add_move(curr_move);
        }
    }
}
