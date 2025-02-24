use std::collections::VecDeque;
use std::sync::atomic::AtomicU64;
use std::vec;
use crate::board::bitboard::Bitboard;
use crate::board::board::Board;
use crate::board::castling::types::CastlingSide;
use crate::board::piece::{Piece, PieceColor, PieceType};
use crate::board::piece::PieceType::PAWN;
use crate::board::position::Position;
use crate::board::see::get_piece_value;
use crate::movegen::constants::{BISHOP_OFFSETS, RANK_1, RANK_2, RANK_7, RANK_8, ROOK_OFFSETS};
use crate::movegen::magic::functions::{get_bishop_attacks, get_rook_attacks};
use crate::movegen::movedata::{CastlingMove, MoveData, MoveType, PromotionCapture};
use crate::movegen::movelist::MoveList;
use crate::movegen::precomputed::{ALIGN_MASK, DIR_RAY_MASK, NUM_SQUARES_FROM_SQUARE, SQR_A_B_MASK};

pub fn generate_all_opp_attacks(board: &Board) ->Bitboard
{
    let opp_color=board.turn.opposite();
    let mut all_attacks=Bitboard::new(0);
    for piece_type in [PieceType::PAWN, PieceType::KNIGHT, PieceType::BISHOP, PieceType::ROOK, PieceType::QUEEN, PieceType::KING].iter(){

            all_attacks |= generate_piece_attack_bitboard(board, &opp_color, piece_type);
        }
    all_attacks
    }


pub fn get_attacking_pieces(board: &Board, square: u8, piece_color: PieceColor) -> Bitboard {
    let mut attackers = Bitboard::new(0);
    let blockers = board.get_all_pieces_bitboard();
    let opp_pawns=board.get_piece_bitboard(piece_color.opposite(), PieceType::PAWN);
    let square_bb=Bitboard::create_from_square(square);
    // Check for pawn attacks
    let pawn_attackers = square_bb.pawn_attack(piece_color, opp_pawns, true) | square_bb.pawn_attack(piece_color, opp_pawns, false);

    attackers|=pawn_attackers;
    // Check for knight attacks
    attackers |= crate::movegen::precomputed::KNIGHT_MOVES[square as usize] & board.get_piece_bitboard(piece_color.opposite(), PieceType::KNIGHT);

    // Check for bishop attacks
    attackers |= get_bishop_attacks(square as usize, blockers) & board.get_piece_bitboard(piece_color.opposite(), PieceType::BISHOP);

    // Check for rook attacks
    attackers |= get_rook_attacks(square as usize, blockers) & board.get_piece_bitboard(piece_color.opposite(), PieceType::ROOK);

    // Check for queen attacks
    attackers |= (get_bishop_attacks(square as usize, blockers) | get_rook_attacks(square as usize, blockers)) & board.get_piece_bitboard(piece_color.opposite(), PieceType::QUEEN);

    // Check for king attacks
    attackers |= crate::movegen::precomputed::KING_MOVES[square as usize] & board.get_piece_bitboard(piece_color.opposite(), PieceType::KING);

    attackers
}
pub fn get_attackers_vec(board: &Board, square: u8, piece_color: PieceColor) -> Vec<(Piece,u8)> {
    let mut attackers = Vec::new();
    let blockers = board.get_all_pieces_bitboard();
    let opp_pawns = board.get_piece_bitboard(piece_color.opposite(), PieceType::PAWN);
    let square_bb = Bitboard::create_from_square(square);

    // Check for pawn attacks
    let mut pawn_attackers = square_bb.pawn_attack(piece_color, opp_pawns, true) | square_bb.pawn_attack(piece_color, opp_pawns, false);

    while pawn_attackers != 0 {
        let start_square= pawn_attackers.pop_lsb();
        attackers.push((Piece::new(piece_color,PAWN),start_square));
    }


    // Check for knight attacks
    let mut knight_attackers = crate::movegen::precomputed::KNIGHT_MOVES[square as usize] & board.get_piece_bitboard(piece_color.opposite(), PieceType::KNIGHT);

    while knight_attackers != 0 {
        let start_square=knight_attackers.pop_lsb();
        attackers.push((Piece::new(piece_color,PieceType::KNIGHT),start_square));
        
    }


    // Check for bishop attacks
    let mut bishop_attackers = get_bishop_attacks(square as usize, blockers) & board.get_piece_bitboard(piece_color.opposite(), PieceType::BISHOP);

    while bishop_attackers != 0 {
        let start_square=bishop_attackers.pop_lsb();
        attackers.push((Piece::new(piece_color,PieceType::BISHOP),start_square));
    }

    // Check for rook attacks
    let mut rook_attackers = get_rook_attacks(square as usize, blockers) & board.get_piece_bitboard(piece_color.opposite(), PieceType::ROOK);

    while rook_attackers != 0 {
      let start_square=rook_attackers.pop_lsb();
        attackers.push((Piece::new(piece_color,PieceType::ROOK),start_square));
    }

    // Check for queen attacks
    let mut queen_attackers = (get_bishop_attacks(square as usize, blockers) | get_rook_attacks(square as usize, blockers)) & board.get_piece_bitboard(piece_color.opposite(), PieceType::QUEEN);

    while queen_attackers != 0 {
        let start_square=queen_attackers.pop_lsb();
        attackers.push((Piece::new(piece_color,PieceType::QUEEN),start_square));
    }
    attackers.sort_by_key(|x| get_piece_value(x.0.piece_type));
    attackers






}
pub fn find_hidden_attackers(board: &Board,delta:Position,square:i32)->Option<(Piece,u8)>{
    let delta_search=-delta;
    let is_ortho=delta_search.is_orthogonal();
    let initial_pos=Position::from_sqr(square as i8).unwrap();
    let i=0;
    loop {
        let curr_pos=initial_pos+delta_search*i;
        if curr_pos.to_sqr().is_none(){
            return None
        }
        let curr_sqr=curr_pos.to_sqr().unwrap();
       if  let Some(curr_piece)=board.squares[curr_sqr as usize]
       {
           if(curr_piece.is_diag() && !is_ortho) || (curr_piece.is_ortho() && is_ortho){
               return Some((curr_piece, curr_sqr as u8));
           }
           else{
               return None;
           }
           
       }
        
    }
    
}


fn generate_piece_attack_bitboard(board: &Board, piece_color: &PieceColor, piece_type: &PieceType) -> Bitboard {
    let blockers = board.get_all_pieces_bitboard() & !board.get_piece_bitboard(piece_color.opposite(), PieceType::KING);
    let piece_bitboard = board.get_piece_bitboard(*piece_color, *piece_type);

    match piece_type {
        PieceType::PAWN => {
            piece_bitboard.pawn_attack(*piece_color, Bitboard::new (u64::MAX), true) | piece_bitboard.pawn_attack(*piece_color, Bitboard::new (u64::MAX), false)
        }
        PieceType::KNIGHT => {
            let mut attacks = Bitboard::new(0);
            let mut knights = piece_bitboard;
            while knights != 0 {
                let from_sqr = knights.pop_lsb();
                attacks |= crate::movegen::precomputed::KNIGHT_MOVES[from_sqr as usize];
            }
            attacks
        }
        PieceType::BISHOP => {
            let mut attacks = Bitboard::new(0);
            let mut bishops = piece_bitboard;
            while bishops != 0 {
                let from_sqr = bishops.pop_lsb();
                attacks |= get_bishop_attacks(from_sqr as usize, blockers);
            }
            attacks
        }
        PieceType::ROOK => {
            let mut attacks = Bitboard::new(0);
            let mut rooks = piece_bitboard;
            while rooks != 0 {
                let from_sqr = rooks.pop_lsb();
                attacks |= get_rook_attacks(from_sqr as usize, blockers);
            }
            attacks
        }
        PieceType::QUEEN => {
            let mut attacks = Bitboard::new(0);
            let mut queens = piece_bitboard;
            while queens != 0 {
                let from_sqr = queens.pop_lsb();
                attacks |= get_bishop_attacks(from_sqr as usize, blockers) | get_rook_attacks(from_sqr as usize, blockers);
            }
            attacks
        }
        PieceType::KING => {
            let mut attacks = Bitboard::new(0);
            let mut kings = piece_bitboard;
            while kings != 0 {
                let from_sqr = kings.pop_lsb();
                attacks |= crate::movegen::precomputed::KING_MOVES[from_sqr as usize];
            }
            attacks
        }
    }
}
pub fn update_check_status(board: &mut Board) {
    board.is_double_check = false;
    let king_square = board.curr_king;
   
  
    board.is_check = board.attacked_square.contains_square(king_square);
    if board.is_check {
        let attackers = get_attacking_pieces(board, king_square, board.turn);
        if attackers.pop_count() > 1 {
            board.is_double_check = true;
        } else {
            let checker_square = attackers.get_single_set_bit();
            let mut valid_moves = Bitboard::new(0);
            let checker_piece = board.squares[checker_square as usize].unwrap();

            // If the checker is not a sliding piece, the only valid move is to capture the checker
            if checker_piece.piece_type != PieceType::BISHOP &&
               checker_piece.piece_type != PieceType::ROOK &&
               checker_piece.piece_type != PieceType::QUEEN {
                valid_moves.set_square(checker_square);
                board.check_ray = Bitboard::new(1 << checker_square)
            } else {
                // If the checker is a sliding piece, calculate the alignment mask between the king and the checker
                let align_mask = SQR_A_B_MASK[king_square as usize][checker_square as usize];


                // Add the checker square to the valid moves
                valid_moves.set_square(checker_square);

                // Add the alignment mask to the valid moves
                valid_moves |= align_mask;

                board.check_ray = valid_moves;
            }
        }
    } else {
        board.check_ray = Bitboard::new(u64::MAX);
    }
}
pub fn generate_moves(board: &mut Board, only_captures:bool) -> MoveList {
    board.curr_king=board.get_piece_bitboard(board.turn, PieceType::KING).get_single_set_bit();
    board.attacked_square=generate_all_opp_attacks(board);
    update_check_status(board);
    board.pinned_ray=find_pinned_pieces(board);
    
    let mut move_list = MoveList::new();
    generate_king_move(board, &mut move_list,only_captures);
    if !board.is_double_check {
        generate_knight_move(board, &mut move_list,only_captures);

        generate_pawn_moves(board, &mut move_list,only_captures);
        get_rook_moves(board, &mut move_list,only_captures);
        get_bishop_moves(board, &mut move_list,only_captures);
        get_queen_moves(board, &mut move_list,only_captures);
    }
    move_list
}
fn is_pinned(board: &Board,sqr:u8)->bool{
     board.pinned_ray.contains_square(sqr)
}
pub fn generate_knight_move(board: &Board, move_list: &mut MoveList,only_captures:bool) {
    let   knights = &mut board.get_piece_bitboard(board.turn, PieceType::KNIGHT);
    let opp_pieces = board.get_color_bitboard(board.turn.opposite());
    let blockers=board.get_color_bitboard(board.turn);
    let  quiet_bitboard= if  only_captures {opp_pieces} else {Bitboard::new(u64::MAX)};
    while *knights != 0  {
        let from_sqr=knights.pop_lsb();
        let mut moves=crate::movegen::precomputed::KNIGHT_MOVES[from_sqr as usize] & !blockers & board.check_ray & quiet_bitboard;
        if is_pinned(board, from_sqr) {
            moves &= ALIGN_MASK[from_sqr as usize][board.curr_king as usize];
        }
        let mut captures= moves & opp_pieces;
        moves &= !captures;
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
pub fn generate_king_move(board: &Board, move_list: &mut MoveList,only_captures:bool) {
    let kings = &mut board.get_piece_bitboard(board.turn, PieceType::KING);
    let opp_pieces = board.get_color_bitboard(board.turn.opposite());
    let blockers = board.get_color_bitboard(board.turn);
    let quiet_bitboard = if only_captures { opp_pieces } else { Bitboard::new(u64::MAX) };
    let from_sqr = kings.pop_lsb();
    let mut moves = crate::movegen::precomputed::KING_MOVES[from_sqr as usize] & !blockers & !board.attacked_square & quiet_bitboard;
    let mut captures = moves & opp_pieces;
    moves &= !captures;
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
    let castling_options = [CastlingSide::Kingside, CastlingSide::Queenside];
    let castling_rights = if board.turn == PieceColor::WHITE {
        board.game_state.castle_white
    } else {
        board.game_state.castle_black
    };

    if (!only_captures) {
        for side in castling_options.iter() {
            if castling_rights.is_allowed(side) && !board.is_check &&
                (board.get_all_pieces_bitboard() & side.required_empty(board.turn) == 0) && (board.attacked_square & side.king_moves_trough(board.turn) == 0)
            {
                let king_start = side.king_start(board.turn);
                let king_end = side.king_end(board.turn);
                let castle_move = MoveData::new(king_start, king_end, board.squares[king_start as usize].unwrap(), MoveType::Castling(CastlingMove::new(*side, board.turn)));
                move_list.add_move(castle_move);
            }
        }
    }
}
pub fn find_pinned_pieces(board: &Board) -> Bitboard {
    let mut pinned_ray = Bitboard::new(0);
    let king_square = Position::from_sqr(board.curr_king as i8).unwrap();
    let opponent_color = board.turn.opposite();

    // Iterate over all 8 directions
    for (dir_index,dir) in ROOK_OFFSETS.iter().chain(BISHOP_OFFSETS.iter()).enumerate() {
        let king_ray = DIR_RAY_MASK[king_square.to_sqr().unwrap() as usize][dir_index];

        // Get opponent's sliding pieces in this direction
        let opponent_sliders = match dir_index {
            0..=3 => board.get_piece_bitboard(opponent_color, PieceType::ROOK) | board.get_piece_bitboard(opponent_color, PieceType::QUEEN),
            _ => board.get_piece_bitboard(opponent_color, PieceType::BISHOP) | board.get_piece_bitboard(opponent_color, PieceType::QUEEN),
        };

        // Find overlapping rays
        let overlap_ray = king_ray & opponent_sliders;

        // Determine pinned pieces
        if overlap_ray != Bitboard::new(0) {
            let mut pieces_between = Bitboard::new(0);
            let mut found_slider = false;
            let mut found_friendly = false;



            for index in 0..NUM_SQUARES_FROM_SQUARE[dir_index][king_square.to_sqr().unwrap() as usize]
            {
                let sq =  (king_square + (*dir * (index+1) )).to_sqr().unwrap() ;
                if board.squares[sq as usize].is_some() {
                    let piece = board.squares[sq as usize].unwrap();
                    if piece.piece_color == board.turn {
                        if found_friendly {
                            found_friendly = false;
                            break;
                        }
                        found_friendly = true;
                    } else if piece.piece_color == opponent_color && is_correct_slider(piece, dir_index) {
                        found_slider = true;
                            break;


                    }
                    else{
                        break;
                    }
                    pieces_between.set_square(sq as u8);
                }
            }

            if found_friendly && found_slider {
                pinned_ray |= pieces_between;
            }
        }
    }

    pinned_ray
}
fn get_promotion_bitboard(pawns:&Bitboard, color: PieceColor) -> Bitboard {
    if color == PieceColor::WHITE { RANK_8} else { RANK_1 }

}
fn is_correct_slider(slider:Piece, dir_index:usize)->bool{
    match dir_index {
        0..=3=>slider.piece_type==PieceType::ROOK || slider.piece_type==PieceType::QUEEN,
        _=>slider.piece_type==PieceType::BISHOP || slider.piece_type==PieceType::QUEEN,
    }
}
pub(crate) fn get_pawn_dir(color: PieceColor) -> i8 {
    if color == PieceColor::WHITE { 1 } else { -1 }
}
fn get_pawn_attack_dir(color: PieceColor, left: bool) -> i8 {
    if color == PieceColor::WHITE {
        if left { 7 } else { 9 }
    } else if left { -7 } else { -9 }
}
pub fn generate_pawn_moves(board: &Board, move_list: &mut MoveList,only_captures:bool) {
    let pawns = &mut board.get_piece_bitboard(board.turn, PieceType::PAWN);
    let opp_pieces = board.get_color_bitboard(board.turn.opposite());
    let blockers = board.get_all_pieces_bitboard();
    let promotion_bitboard = get_promotion_bitboard(pawns, board.turn);
if(!only_captures) {
    let mut double_pushes = pawns.pawn_double_push(&board.turn, blockers) & board.check_ray;
    let mut single_pushes = pawns.pawn_push(&board.turn) & !blockers & !opp_pieces & board.check_ray;
    let mut single_pushes_promote = single_pushes & promotion_bitboard;
    single_pushes &= !promotion_bitboard;
    while single_pushes != 0
    {
        let end_sq = single_pushes.pop_lsb();
        let start_sq = (end_sq as i8 - (8 * get_pawn_dir(board.turn))) as u8;
        if !is_pinned(board, start_sq) || ALIGN_MASK[start_sq as usize][board.curr_king as usize] == ALIGN_MASK[end_sq as usize][board.curr_king as usize] {
            let curr_move = MoveData::new(
                start_sq,
                end_sq,
                board.squares[start_sq as usize].unwrap(),
                MoveType::Normal
            );
            move_list.add_move(curr_move)
        }
    }
    while single_pushes_promote != 0
    {
        let end_sq = single_pushes_promote.pop_lsb();
        let start_sq = (end_sq as i8 - (8 * get_pawn_dir(board.turn))) as u8;
        if !is_pinned(board, start_sq) || ALIGN_MASK[start_sq as usize][board.curr_king as usize] == ALIGN_MASK[end_sq as usize][board.curr_king as usize] {
            generate_promote_moves(board, start_sq, end_sq, move_list, board.turn);
        }
    }
    while double_pushes != 0
    {
        let end_sq = double_pushes.pop_lsb();
        let start_sq = (end_sq as i8 - (16 * get_pawn_dir(board.turn))) as u8;
        let curr_move = MoveData::new(start_sq, end_sq, board.squares[start_sq as usize].unwrap(), MoveType::Normal);
        if !is_pinned(board, start_sq) || ALIGN_MASK[start_sq as usize][board.curr_king as usize] == ALIGN_MASK[end_sq as usize][board.curr_king as usize] {
            move_list.add_move(curr_move);
        }
    }
}
    let is_left: [bool; 2] = [true, false];
    for left in is_left.iter() {
        let mut attacks = pawns.pawn_attack(board.turn, opp_pieces, *left) & board.check_ray;
        let mut promote_attacks = attacks & promotion_bitboard;
        attacks &= !promotion_bitboard;
        while attacks != 0
        {
            let end_sq = attacks.pop_lsb();
            let start_sq = (end_sq as i8 - (get_pawn_attack_dir(board.turn, *left))) as u8;
            if !is_pinned(board, start_sq) || ALIGN_MASK[start_sq as usize][board.curr_king as usize] == ALIGN_MASK[end_sq as usize][board.curr_king as usize] {
               let curr_move = MoveData::new(
    start_sq,
    end_sq,
    board.squares[start_sq as usize].unwrap(),
    MoveType::Capture(board.squares[end_sq as usize].unwrap())
);
                move_list.add_move(curr_move);
            }
        }
        while promote_attacks != 0
        {
            let end_sq = promote_attacks.pop_lsb();
            let start_sq = (end_sq as i8 - (get_pawn_attack_dir(board.turn, *left))) as u8;
            if !is_pinned(board, start_sq) || ALIGN_MASK[start_sq as usize][board.curr_king as usize] == ALIGN_MASK[end_sq as usize][board.curr_king as usize] {
                generate_promote_captures(board, start_sq, end_sq, move_list, board.turn, board.squares[end_sq as usize].unwrap());
            }
        }
    }
    if let Some(en_passant_square) = board.game_state.en_passant_square
    {
        let en_passant_target = (en_passant_square as i8 - (8 * get_pawn_dir(board.turn))) as u8;
        let mut en_passent_bitboard=Bitboard::create_from_square(en_passant_square );
        let  mut pawns_can_capture = en_passent_bitboard.pawn_attack(board.turn.opposite(), *pawns, true) | en_passent_bitboard.pawn_attack(board.turn.opposite(), *pawns, false);
       // let mut pawns_can_capture = pawns_attack_pattern & *pawns;
        while pawns_can_capture != 0
        {
            let start_sq = pawns_can_capture.pop_lsb();
            if !is_pinned(board, start_sq) || ALIGN_MASK[start_sq as usize][board.curr_king as usize] == ALIGN_MASK[en_passant_target as usize][board.curr_king as usize] {
                if !in_check_after_en_passant(board, start_sq, en_passant_square, en_passant_target) {
                    let curr_move = MoveData::new(start_sq, en_passant_square, board.squares[start_sq as usize].unwrap(), MoveType::EnPassant(board.squares[en_passant_target as usize].unwrap(), en_passant_target));
                    move_list.add_move(curr_move);
                }
            }
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
pub fn get_rook_moves(board: &Board, move_list: &mut MoveList,only_captures:bool) {
    let rooks = &mut board.get_piece_bitboard(board.turn, PieceType::ROOK);
    let opp_pieces = board.get_color_bitboard(board.turn.opposite());
    let blockers = board.get_all_pieces_bitboard();
    let quiet_bitboard= if  only_captures {opp_pieces} else {Bitboard::new(u64::MAX)};
    while *rooks != 0 {
        let from_sqr = rooks.pop_lsb();
        let mut moves = get_rook_attacks(from_sqr as usize, blockers) & board.check_ray & !board.get_color_bitboard(board.turn) & quiet_bitboard;
        if is_pinned(board, from_sqr) {
            moves &= ALIGN_MASK[from_sqr as usize][board.curr_king as usize];
        }
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
pub fn get_bishop_moves(board: &Board, move_list: &mut MoveList,only_captures:bool) {
    let bishops = &mut board.get_piece_bitboard(board.turn, PieceType::BISHOP);
    let opp_pieces = board.get_color_bitboard(board.turn.opposite());
    let blockers = board.get_all_pieces_bitboard();
    let our_pieces=board.get_color_bitboard(board.turn);
    let quiet_bitboard= if  only_captures {opp_pieces} else {Bitboard::new(u64::MAX)};
    while *bishops != 0 {
        let from_sqr = bishops.pop_lsb();
        let mut moves = get_bishop_attacks(from_sqr as usize, blockers) & board.check_ray & !our_pieces & quiet_bitboard;
        if is_pinned(board, from_sqr) {
            moves &= ALIGN_MASK[from_sqr as usize][board.curr_king as usize];
        }
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
pub fn get_queen_moves(board: &Board, move_list: &mut MoveList,only_captures:bool) {
    let queens = &mut board.get_piece_bitboard(board.turn, PieceType::QUEEN);
    let opp_pieces = board.get_color_bitboard(board.turn.opposite());
    let blockers = board.get_all_pieces_bitboard();
    let our_pieces=board.get_color_bitboard(board.turn);
    
    let quiet_bitboard= if  only_captures {opp_pieces} else {Bitboard::new(u64::MAX)};
    while *queens != 0 {
        let from_sqr = queens.pop_lsb();
        let mut moves = (get_bishop_attacks(from_sqr as usize, blockers) | get_rook_attacks(from_sqr as usize, blockers)) & board.check_ray & !our_pieces & quiet_bitboard;
        moves &=!our_pieces;
        if is_pinned(board, from_sqr) {
            moves &= ALIGN_MASK[from_sqr as usize][board.curr_king as usize];
        }
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
pub fn in_check_after_en_passant(board: &Board, start_square: u8, target_square: u8, ep_capture_square: u8) -> bool {
    let enemy_ortho = board.get_piece_bitboard(board.turn.opposite(), PieceType::ROOK) |
        board.get_piece_bitboard(board.turn.opposite(), PieceType::QUEEN);

    if enemy_ortho != Bitboard::new(0) {
        let masked_blockers = board.get_all_pieces_bitboard() ^
            (Bitboard::create_from_square(ep_capture_square) |
                Bitboard::create_from_square(start_square) |
                Bitboard::create_from_square(target_square));
        let rook_attacks = get_rook_attacks(board.curr_king as usize, masked_blockers);
        return (rook_attacks & enemy_ortho) != Bitboard::new(0);
    }

    false
}