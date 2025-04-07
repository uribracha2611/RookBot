use std::collections::HashMap;
use crate::board::castling::types::{AllowedCastling, CastlingSide};
use crate::board::piece::PieceType;
use crate::movegen::magic::functions::{get_bishop_attacks, get_rook_attacks};
use crate::movegen::movedata::MoveData;
use crate::movegen::precomputed::KNIGHT_MOVES;
use crate::search::psqt::constants::GAMEPHASE_INC;
use crate::search::psqt::function;
use crate::search::psqt::function::get_psqt;
use crate::search::psqt::weight::W;
use super::{bitboard, bitboard::Bitboard, gamestate::GameState, piece::{Piece, PieceColor}};
use fxhash::FxHashMap;
use crate::search::Zobrist::constants::{ZOBRIST_EN_PASSANT, ZOBRIST_KEYS, ZOBRIST_SIDE_TO_MOVE};

#[derive( Clone)]
pub struct Board {
    pub squares: [Option<Piece>; 64],
    pub turn: PieceColor,
     color_bitboards: [Bitboard; 2],
     piece_bitboards: [[Bitboard; 6]; 2],
     all_pieces_bitboard: Bitboard,
    pub game_state: GameState,
    pub is_check: bool,
    pub is_double_check: bool,
    pub attacked_square:Bitboard,
    pub curr_king:u8,
    pub check_ray:Bitboard,
    pub pinned_ray:Bitboard,
    pub psqt_white:W,
    pub psqt_black:W,
    pub game_phase:i32,
    history:Vec<GameState>,
    repetition_table:FxHashMap<u64,u32>
}

impl Board {



    fn remove_piece(&mut self, square: u8, piece: Piece) {
        let index = 6 * piece.piece_color.to_index() + piece.piece_type.to_index();
        // Update Zobrist hash before removing the piece
        self.game_state.zobrist_hash ^= ZOBRIST_KEYS[index][square as usize];
        if piece.piece_color==PieceColor::WHITE {
            self.psqt_white-=get_psqt(square as usize,piece);
            self.game_phase-=GAMEPHASE_INC[piece.piece_type as usize];
        }
        else{
            self.psqt_black-=get_psqt(square as usize,piece);
            self.game_phase-=GAMEPHASE_INC[piece.piece_type as usize];
        }
        self.squares[square as usize] = None;
        self.get_color_bitboard_mut(piece.piece_color).clear_square(square);
        self.get_piece_bitboard_mut(piece.piece_color, piece.piece_type).clear_square(square);
        self.all_pieces_bitboard.clear_square(square);
    }

   
    fn add_piece(&mut self, square: u8, piece: Piece) {
        let index = 6 * piece.piece_color.to_index() + piece.piece_type.to_index();
        // Update Zobrist hash before adding the piece
        self.game_state.zobrist_hash ^= ZOBRIST_KEYS[index][square as usize];
        if piece.piece_color==PieceColor::WHITE {
            self.psqt_white+=get_psqt(square as usize,piece);
        }
        else{
            self.psqt_black+=get_psqt(square as usize,piece);

        }
        self.game_phase+=GAMEPHASE_INC[piece.piece_type as usize];
        self.squares[square as usize] = Some(piece);
        self.get_color_bitboard_mut(piece.piece_color).set_square(square);
        self.get_piece_bitboard_mut(piece.piece_color, piece.piece_type).set_square(square);
        self.all_pieces_bitboard.set_square(square);
    }
    
    pub fn detect_pawns_only(&self,piece_color: PieceColor) -> bool {
        return self.get_color_bitboard(piece_color) ^ self.get_piece_bitboard(piece_color,PieceType::PAWN) ==0;
    }
    pub fn is_quiet_move(self:&Board,mv: &MoveData)->bool{
         !mv.is_capture() && !mv.is_promotion() && !self.is_check && !self.is_move_check(&mv)
    }
   pub fn is_move_check(&self, mv: &MoveData)->bool{
       let to=mv.to;
       let piece_moved=mv.piece_to_move;
       let opp_king=self.get_piece_bitboard(piece_moved.piece_color.opposite(),PieceType::KING);
       let blockers=self.all_pieces_bitboard & !Bitboard::create_from_square(to);
       match piece_moved.piece_type {
              PieceType::KING=>{
                false
              }
              PieceType::KNIGHT=>{
                  KNIGHT_MOVES[to as usize] & opp_king!=0
              }
           PieceType::PAWN=>{
               let pawn_attacks=opp_king.pawn_attack(piece_moved.piece_color.opposite(),Bitboard::create_from_square(to),true) | opp_king.pawn_attack(piece_moved.piece_color.opposite(),Bitboard::create_from_square(to),false);
               pawn_attacks & opp_king!=0
           }
              PieceType::BISHOP=>{
                let bishop_attacks=get_bishop_attacks(to as usize, blockers);
                bishop_attacks & opp_king!=0
              }
                PieceType::ROOK=>{
                    let rook_attacks=get_rook_attacks(to as usize, blockers);
                    rook_attacks & opp_king!=0
                }
                PieceType::QUEEN=>{
                    let bishop_attacks=get_bishop_attacks(to as usize, blockers);
                    let rook_attacks=get_rook_attacks(to as usize, blockers);
                    (bishop_attacks | rook_attacks) & opp_king!=0
                }
           


       }
   }

    
    pub fn from_fen(fen: &str) -> Self {
        let parts: Vec<&str> = fen.split_whitespace().collect();

        // Validate that the FEN has the minimum required parts
        if parts.len() < 6 {
            panic!("Invalid FEN string: insufficient parts");
        }

        // Parse piece placement string (first field of FEN)
        let piece_placement = parts[0];
        let active_color = parts[1]; // Second field (active color)
        let game_state_fen = parts[2..].join(" "); // Remaining fields (castling, en passant, clocks)

        let mut squares = [None; 64];
        let mut board = Board {
            squares,
            turn: if active_color == "w" { PieceColor::WHITE } else { PieceColor::BLACK },
            color_bitboards: [Bitboard::new(0), Bitboard::new(0)],
            piece_bitboards: [[Bitboard::new(0); 6]; 2],
            all_pieces_bitboard: Bitboard::new(0),
            game_state: GameState::from_fen(&game_state_fen),
            is_check: false,
            is_double_check: false,
            attacked_square: Bitboard::new(0),
            curr_king: 0,
            check_ray: Bitboard::new(u64::MAX),
            pinned_ray: Bitboard::new(0),
            psqt_white: W(0,0),
            psqt_black: W(0,0),
            game_phase: 0,
            history:Vec::new(),
            repetition_table:FxHashMap::default()
        };

        let mut rank = 7;
        let mut file = 0;

        // Parse piece placement into the board squares
        for c in piece_placement.chars() {
            match c {
                '/' => {
                    rank -= 1;
                    file = 0;
                }
                '1'..='8' => {
                    file += c.to_digit(10).unwrap() as usize;
                }
                _ => {
                    if let Some(piece) = Piece::from_fen(&c.to_string()) {
                        board.add_piece((rank * 8 + file) as u8, piece);
                    }
                    file += 1;
                }
            }
        }

        board
    }
    pub fn increment_repetition_table(&mut self) {
        let count = self.repetition_table.entry(self.game_state.zobrist_hash).or_insert(0);
        *count += 1;
        
    }
    pub fn is_three_draw_repetition(&self) -> bool {
        self.repetition_table.get(&self.game_state.zobrist_hash).is_some_and(|&count| count >= 3)
    }
    pub fn is_insufficient_material(&self) -> bool {
        let pieces_without_white_king = self.get_color_bitboard(PieceColor::WHITE)
            & !self.get_piece_bitboard(PieceColor::WHITE, PieceType::KING);
        let pieces_without_black_king = self.get_color_bitboard(PieceColor::BLACK)
            & !self.get_piece_bitboard(PieceColor::BLACK, PieceType::KING);

        let piece_count_white_without_king = pieces_without_white_king.pop_count();
        let piece_count_black_without_king = pieces_without_black_king.pop_count();

        // If both sides have more than one non-king piece, checkmate is possible
        if piece_count_white_without_king > 1 || piece_count_black_without_king > 1 {
            return false;
        }

        // If neither side has any piece besides the king, it's a draw
        if piece_count_white_without_king == 0 && piece_count_black_without_king == 0 {
            return true;
        }

        // Get the single piece for each side, if it exists
        let white_piece =  self.squares[pieces_without_white_king.get_single_set_bit() as usize];

        let black_piece =  self.squares[pieces_without_black_king.get_single_set_bit() as usize];

        // Check if each side has only a knight or bishop (this covers different-colored bishops case)
        let white_draw_bool = white_piece.is_none_or(|piece| {
            piece.piece_type == PieceType::BISHOP || piece.piece_type == PieceType::KNIGHT
        });

        let black_draw_bool = black_piece.is_none_or(|piece| {
            piece.piece_type == PieceType::BISHOP || piece.piece_type == PieceType::KNIGHT
        });

        white_draw_bool && black_draw_bool
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty_count = 0;
            for file in 0..8 {
                if let Some(piece) = self.squares[rank * 8 + file] {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    fen.push_str(&piece.to_fen());
                } else {
                    empty_count += 1;
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        let active_color = if self.turn == PieceColor::WHITE {
            "w"
        } else {
            "b"
        };

        fen.push_str(&format!(" {} {}", active_color, self.game_state.to_fen()));
        fen
    }
    pub fn make_move(&mut self, mv:&MoveData,is_search:bool)
    {
        let  old_game_state =self.game_state.clone();
        let moved_piece = mv.piece_to_move;
        if mv.is_capture() {
            self.remove_piece(mv.get_capture_square().unwrap(),mv.get_captured_piece().unwrap());
            self.disallow_castling_if_needed(mv.get_capture_square().unwrap(), mv.get_captured_piece().unwrap());

        } 
        if(mv.is_promotion()){
            self.remove_piece(mv.from,moved_piece);
            self.add_piece(mv.to,mv.get_promoted_piece().unwrap());
        }
        else{
            self.remove_piece(mv.from,moved_piece);
            self.add_piece(mv.to,moved_piece);
        }
        if mv.is_castling(){
            let rook_start = mv.get_rook_start().unwrap();
            let rook_end = mv.get_rook_end().unwrap();
            let rook = self.squares[rook_start as usize].unwrap();
            self.remove_piece(rook_start,rook);
            self.add_piece(rook_end,rook);
            self.game_state.disallow_castling_both(moved_piece.piece_color);
            
            
        }
        if(moved_piece.piece_type==PieceType::KING)
        {
            self.game_state.disallow_castling_both(moved_piece.piece_color);
         
        }
       self.disallow_castling_if_needed(mv.from, moved_piece);
        self.handle_en_passant(mv);

        
        
        self.turn = self.turn.opposite();
        self.game_state.zobrist_hash^=ZOBRIST_SIDE_TO_MOVE;
        self.game_state.fullmove_clock+=1;
        if moved_piece.piece_type==PieceType::PAWN || mv.is_capture(){
            self.game_state.halfmove_clock=0;
        }
        else{
            self.game_state.halfmove_clock+=1;
        }
        
        self.history.push(old_game_state);
        if !is_search {
            self.increment_repetition_table();
        }
        
        
        
        
    }
    pub fn is_board_draw(&self)->bool{
        self.is_three_draw_repetition() || self.is_insufficient_material() || self.game_state.halfmove_clock>=50
    }
    fn handle_en_passant(&mut self, mv: &MoveData) {
        if mv.piece_to_move.piece_type == PieceType::PAWN && mv.is_double_push() {
            let new_en_passant_square = if mv.piece_to_move.piece_color == PieceColor::WHITE {
                mv.to - 8
            } else {
                mv.to + 8
            };
            self.game_state.en_passant_file = Some(mv.to % 8);
            self.game_state.en_passant_square = Some(new_en_passant_square);

            // Update Zobrist hash for en passant
            self.game_state.zobrist_hash ^= ZOBRIST_EN_PASSANT[mv.to as usize % 8];
        } else {
            if let Some(file) = self.game_state.en_passant_file {
                // Remove old en passant from Zobrist hash
                self.game_state.zobrist_hash ^= ZOBRIST_EN_PASSANT[file as usize];
            }
            self.game_state.en_passant_file = None;
            self.game_state.en_passant_square = None;
        }
    }
    pub fn unmake_move(&mut self, mv: &MoveData) {
        let moved_piece = mv.piece_to_move;
        if mv.is_promotion() {
            self.remove_piece(mv.to, mv.get_promoted_piece().unwrap());
            self.add_piece(mv.from, moved_piece);
        }
        else {
            // Restore the piece to its original position
            self.remove_piece(mv.to, moved_piece);
            self.add_piece(mv.from, moved_piece);
        }

        // Restore captured piece if it was a capture move
        if mv.is_capture() {
            self.add_piece(mv.get_capture_square().unwrap(), mv.get_captured_piece().unwrap());
        }

        // Handle promotion


        // Handle castling
        if mv.is_castling() {
            let rook_start = mv.get_rook_start().unwrap();
            let rook_end = mv.get_rook_end().unwrap();
            let rook = self.squares[rook_end as usize].unwrap();
            self.remove_piece(rook_end, rook);
            self.add_piece(rook_start, rook);

        }

        // Restore game state
        self.game_state = self.history.pop().unwrap();
        self.turn = self.turn.opposite();
    }

    fn disallow_castling_if_needed(&mut self, square: u8, piece: Piece) {
        if piece.piece_type != PieceType::ROOK {
            return;
        }
        match (square, piece.piece_color) {
            (0, PieceColor::WHITE) if self.game_state.castle_white.is_allowed(&CastlingSide::Queenside) => {
                self.game_state.disallow_castling(AllowedCastling::from(CastlingSide::Queenside), piece.piece_color);
            }
            (7, PieceColor::WHITE) if self.game_state.castle_white.is_allowed(&CastlingSide::Kingside) => {
                self.game_state.disallow_castling(AllowedCastling::from(CastlingSide::Kingside), piece.piece_color);
            }
            (56, PieceColor::BLACK) if self.game_state.castle_black.is_allowed(&CastlingSide::Queenside) => {
                self.game_state.disallow_castling(AllowedCastling::from(CastlingSide::Queenside), piece.piece_color);
            }
            (63, PieceColor::BLACK) if self.game_state.castle_black.is_allowed(&CastlingSide::Kingside) => {
                self.game_state.disallow_castling(AllowedCastling::from(CastlingSide::Kingside), piece.piece_color);
            }
            _ => {}
        }
    }

    pub fn make_null_move(&mut self) {
        let old_game_state = self.game_state;
        self.history.push(old_game_state);

        // Clear en passant square before flipping the turn
        if let Some(file) = self.game_state.en_passant_file {
            self.game_state.zobrist_hash ^= ZOBRIST_EN_PASSANT[file as usize];
            self.game_state.en_passant_file = None;
            self.game_state.en_passant_square = None;
        }

        self.turn = self.turn.opposite();
        self.game_state.zobrist_hash ^= ZOBRIST_SIDE_TO_MOVE;
    }

    pub fn unmake_null_move(&mut self) {
        if let Some(previous_state) = self.history.pop() {
            self.game_state = previous_state;
            self.turn = self.turn.opposite();
        } else {
            panic!("No previous game state to unmake null move");
        }
    }


    pub fn to_stockfish_string(&self) -> String {
        let mut stockfish_str = String::new();

        for rank in (0..8).rev() {
            for file in 0..8 {
                if let Some(piece) = self.squares[rank * 8 + file] {
                    stockfish_str.push_str(&piece.to_fen());
                } else {
                    stockfish_str.push('.');
                }
            }
            stockfish_str.push('\n');
        }

        stockfish_str.push_str(&self.game_state.to_stockfish_string());
        stockfish_str.push_str(&format!("fen: {}\n", self.to_fen()));
        
        stockfish_str
    }
    pub  fn get_piece_bitboard(&self, color: PieceColor, piece: PieceType) -> Bitboard {
        self.piece_bitboards[color as usize][piece as usize]
    }
    pub fn get_color_bitboard(&self, color: PieceColor) -> Bitboard {
        self.color_bitboards[color as usize]
    }
    fn get_piece_bitboard_mut(&mut self, color: PieceColor, piece: PieceType) -> &mut Bitboard {
    &mut self.piece_bitboards[color as usize][piece as usize]
}

fn get_color_bitboard_mut(&mut self, color: PieceColor) -> &mut Bitboard {
    &mut self.color_bitboards[color as usize]
}
    pub fn get_all_pieces_bitboard(&self) -> Bitboard {
        self.all_pieces_bitboard
    }
}
