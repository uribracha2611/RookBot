use super::{
    bitboard::Bitboard,
    gamestate::GameState,
    piece::{Piece, PieceColor},
};
use crate::engine::board::castling::types::{AllowedCastling, CastlingSide};
use crate::engine::board::piece::PieceColor::{BLACK, WHITE};
use crate::engine::board::piece::PieceType;
use crate::engine::board::piece::PieceType::{BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK};
use crate::engine::movegen::constants::KNIGHT_MOVES;
use crate::engine::movegen::magic::functions::{get_bishop_attacks, get_rook_attacks};
use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::psqt::constants::GAMEPHASE_INC;
use crate::engine::search::psqt::function::get_psqt;
use crate::engine::search::psqt::weight::W;
use crate::engine::search::zobrist::constants::{
    ZOBRIST_CASTLING, ZOBRIST_EN_PASSANT, ZOBRIST_KEYS, ZOBRIST_SIDE_TO_MOVE,
};

#[derive(Clone)]
pub struct Board {
    pub squares: [Option<Piece>; 64],
    pub turn: PieceColor,
    color_bitboards: [Bitboard; 2],
    piece_bitboards: [[Bitboard; 6]; 2],
    all_pieces_bitboard: Bitboard,
    pub game_state: GameState,
    pub attacked_square: Bitboard,
    pub curr_king: u8,
    pub psqt_white: W,
    pub psqt_black: W,
    pub game_phase: i32,
    history: Vec<GameState>,
    pub repetition_table: Vec<u64>,
}

impl Board {
    fn remove_piece(&mut self, square: u8, piece: Piece) {
        debug_assert!(self.squares[square as usize] == Some(piece));
        debug_assert!(self.get_piece_bitboard(piece.piece_color, piece.piece_type).contains_square(square));
        debug_assert!(self.get_color_bitboard(piece.piece_color).contains_square(square));
        let index = 6 * piece.piece_color.to_index() + piece.piece_type.to_index();
        // Update zobrist hash before removing the piece
        self.game_state.zobrist_hash ^= ZOBRIST_KEYS[index][square as usize];
        if piece.piece_color == PieceColor::WHITE {
            self.psqt_white -= get_psqt(square as usize, piece);
            self.game_phase -= GAMEPHASE_INC[piece.piece_type as usize];
        } else {
            self.psqt_black -= get_psqt(square as usize, piece);
            self.game_phase -= GAMEPHASE_INC[piece.piece_type as usize];
        }
        self.squares[square as usize] = None;
        self.get_color_bitboard_mut(piece.piece_color)
            .clear_square(square);
        self.get_piece_bitboard_mut(piece.piece_color, piece.piece_type)
            .clear_square(square);
        self.all_pieces_bitboard.clear_square(square);
        debug_assert!(self.squares[square as usize].is_none());
        debug_assert!(!self.get_piece_bitboard(piece.piece_color, piece.piece_type).contains_square(square));
        debug_assert!(!self.get_color_bitboard(piece.piece_color).contains_square(square));
    }

    fn add_piece(&mut self, square: u8, piece: Piece) {
        debug_assert!(self.squares[square as usize].is_none());
        debug_assert!(!self.get_piece_bitboard(piece.piece_color, piece.piece_type).contains_square(square));
        debug_assert!(!self.get_color_bitboard(piece.piece_color).contains_square(square));
        let index = 6 * piece.piece_color.to_index() + piece.piece_type.to_index();
        // Update zobrist hash before adding the piece
        self.game_state.zobrist_hash ^= ZOBRIST_KEYS[index][square as usize];
        if piece.piece_color == PieceColor::WHITE {
            self.psqt_white += get_psqt(square as usize, piece);
        } else {
            self.psqt_black += get_psqt(square as usize, piece);
        }
        self.game_phase += GAMEPHASE_INC[piece.piece_type as usize];
        self.squares[square as usize] = Some(piece);
        self.get_color_bitboard_mut(piece.piece_color)
            .set_square(square);
        self.get_piece_bitboard_mut(piece.piece_color, piece.piece_type)
            .set_square(square);
        self.all_pieces_bitboard.set_square(square);
        debug_assert!(self.squares[square as usize] == Some(piece));
        debug_assert!(self.get_piece_bitboard(piece.piece_color, piece.piece_type).contains_square(square));
        debug_assert!(self.get_color_bitboard(piece.piece_color).contains_square(square));
    }

    pub fn detect_pawns_only(&self, piece_color: PieceColor) -> bool {
        self.get_color_bitboard(piece_color)
            ^ self.get_piece_bitboard(piece_color, PieceType::PAWN)
            == 0
    }
    pub fn is_quiet_move(self: &Board, mv: &MoveData) -> bool {
        !mv.is_capture() && !mv.is_promotion() && !self.game_state.is_check && !self.is_move_check(mv)
    }
    pub fn is_move_check(&self, mv: &MoveData) -> bool {
        let to = mv.to;
        let piece_moved = mv.piece_to_move;
        let opp_king = self.get_piece_bitboard(piece_moved.piece_color.opposite(), PieceType::KING);
        let blockers = self.all_pieces_bitboard & !Bitboard::create_from_square(to);
        match piece_moved.piece_type {
            PieceType::KING => false,
            PieceType::KNIGHT => KNIGHT_MOVES[to as usize] & opp_king != 0,
            PieceType::PAWN => {
                let pawn_attacks = opp_king.pawn_attack(
                    piece_moved.piece_color.opposite(),
                    Bitboard::create_from_square(to),
                    true,
                ) | opp_king.pawn_attack(
                    piece_moved.piece_color.opposite(),
                    Bitboard::create_from_square(to),
                    false,
                );
                pawn_attacks & opp_king != 0
            }
            PieceType::BISHOP => {
                let bishop_attacks = get_bishop_attacks(to as usize, blockers);
                bishop_attacks & opp_king != 0
            }
            PieceType::ROOK => {
                let rook_attacks = get_rook_attacks(to as usize, blockers);
                rook_attacks & opp_king != 0
            }
            PieceType::QUEEN => {
                let bishop_attacks = get_bishop_attacks(to as usize, blockers);
                let rook_attacks = get_rook_attacks(to as usize, blockers);
                (bishop_attacks | rook_attacks) & opp_king != 0
            }
        }
    }

    pub fn is_threefold_repetition(&self) -> bool {
        // let len=self.repetition_table.len();
        // let start = len.saturating_sub(self.game_state.halfmove_clock as usize);
        let count = self
            .repetition_table
            .iter()
            .filter(|&&hash| hash == self.game_state.zobrist_hash)
            .count();
        count >= 3
    }

    pub fn from_fen(fen: &str) -> Self {
        let parts: Vec<&str> = fen.split_whitespace().collect();

        // Validate that the FEN has the minimum required parts
        if parts.len() < 6 {
            println!("Invalid fen: {}", fen);
            panic!("Invalid FEN string: insufficient parts");
        }

        // Parse piece placement string (first field of FEN)
        let piece_placement = parts[0];
        let active_color = parts[1]; // Second field (active color)
        let game_state_fen = parts[2..].join(" "); // Remaining fields (castling, en passant, clocks)

        let squares = [None; 64];
        let mut board = Board {
            squares,
            turn: if active_color == "w" {
                PieceColor::WHITE
            } else {
                PieceColor::BLACK
            },
            color_bitboards: [Bitboard::new(0), Bitboard::new(0)],
            piece_bitboards: [[Bitboard::new(0); 6]; 2],
            all_pieces_bitboard: Bitboard::new(0),
            game_state: GameState::from_fen(&game_state_fen),
            curr_king: 0,
            attacked_square: Bitboard::new(0),
            psqt_white: W(0, 0),
            psqt_black: W(0, 0),
            game_phase: 0,
            history: Vec::new(),
            repetition_table: Vec::new(),
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
        if board.turn == PieceColor::BLACK {
            board.game_state.zobrist_hash ^= ZOBRIST_SIDE_TO_MOVE;
        }
        board.repetition_table.push(board.game_state.zobrist_hash);
        board
    }

    pub fn is_insufficient_material(&self) -> bool {
        if self.get_piece_bitboard(WHITE, PAWN) | self.get_piece_bitboard(BLACK, PAWN) != 0 {
            return false;
        }
        let white_queen = self.get_piece_bitboard(WHITE, QUEEN);
        let black_queen = self.get_piece_bitboard(BLACK, QUEEN);
        if white_queen | black_queen != 0 {
            return false;
        }

        let white_bishop = self.get_piece_bitboard(WHITE, BISHOP);
        let black_bishop = self.get_piece_bitboard(BLACK, BISHOP);
        let white_knight = self.get_piece_bitboard(WHITE, KNIGHT);
        let white_rook = self.get_piece_bitboard(WHITE, ROOK);
        let black_knight = self.get_piece_bitboard(BLACK, KNIGHT);
        let black_rook = self.get_piece_bitboard(BLACK, ROOK);
        let bishops = white_bishop | black_bishop;
        let knights = white_knight | black_knight;
        let rooks = white_rook | black_rook;
        if rooks == 0 {
            if bishops == 0 {
                if white_knight.pop_count() < 3 && black_knight.pop_count() < 3 {
                    return true;
                }
            } else if knights == 0 && white_bishop.pop_count().abs_diff(black_bishop.pop_count()) < 2 || (white_bishop | white_knight).pop_count() == 1 && (black_bishop | black_knight).pop_count() == 1 {
                return true;
            }
        } else if white_rook.pop_count() == 1 && black_rook == 0 {
            if (white_knight | white_bishop) == 0 && ((black_knight | black_bishop).pop_count() == 1 || (black_knight | black_bishop).pop_count() == 2) {
                return true;
            }
        } else if white_rook == 0 && black_rook.pop_count() == 1 && (black_knight | black_bishop) == 0 && ((white_knight | white_bishop).pop_count() == 1 || (white_knight | white_bishop).pop_count() == 2) {
            return true
        }
        false
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
    pub fn make_move(&mut self, mv: &MoveData) {
        let old_game_state = self.game_state;
        let moved_piece = mv.piece_to_move;
        if mv.is_capture() {
            self.remove_piece(
                mv.get_capture_square().unwrap(),
                mv.get_captured_piece().unwrap(),
            );
            self.disallow_castling_if_needed(
                mv.get_capture_square().unwrap(),
                mv.get_captured_piece().unwrap(),
            );
        }
        if mv.is_promotion() {
            self.remove_piece(mv.from, moved_piece);
            self.add_piece(mv.to, mv.get_promoted_piece().unwrap());
        } else {
            self.remove_piece(mv.from, moved_piece);
            self.add_piece(mv.to, moved_piece);
        }
        if mv.is_castling() {
            let rook_start = mv.get_rook_start().unwrap();
            let rook_end = mv.get_rook_end().unwrap();
            let rook = self.squares[rook_start as usize].unwrap();
            self.remove_piece(rook_start, rook);
            self.add_piece(rook_end, rook);
            self.game_state
                .disallow_castling_both(moved_piece.piece_color);
        }
        if moved_piece.piece_type == PieceType::KING {
            self.game_state
                .disallow_castling_both(moved_piece.piece_color);
        }
        self.disallow_castling_if_needed(mv.from, moved_piece);
        self.handle_en_passant(mv);

        self.turn = self.turn.opposite();
        self.game_state.zobrist_hash ^= ZOBRIST_SIDE_TO_MOVE;
        self.game_state.fullmove_clock += 1;
        if moved_piece.piece_type == PieceType::PAWN || mv.is_capture() {
            self.game_state.halfmove_clock = 0;
        } else {
            self.game_state.halfmove_clock += 1;
        }

        self.history.push(old_game_state);
        self.repetition_table.push(self.game_state.zobrist_hash);
        debug_assert!(self.squares[mv.from as usize].is_none());
        debug_assert!((mv.is_promotion() && self.squares[mv.to as usize] == Some(mv.get_promoted_piece().unwrap())) || (!mv.is_promotion() && self.squares[mv.to as usize] == Some(mv.piece_to_move)));
        debug_assert!(self.game_state.zobrist_hash == self.calc_zobrist());
    }
    pub fn is_board_draw(&self) -> bool {
        self.is_threefold_repetition()
            || self.game_state.halfmove_clock >= 100
    }
    fn handle_en_passant(&mut self, mv: &MoveData) {
        if let Some(file) = self.game_state.en_passant_file {
            // Remove old en passant from zobrist hash
            self.game_state.zobrist_hash ^= ZOBRIST_EN_PASSANT[file as usize];
        }
        if mv.piece_to_move.piece_type == PieceType::PAWN && mv.is_double_push() {
            let new_en_passant_square = if mv.piece_to_move.piece_color == PieceColor::WHITE {
                mv.to - 8
            } else {
                mv.to + 8
            };
            self.game_state.en_passant_file = Some(mv.to % 8);
            self.game_state.en_passant_square = Some(new_en_passant_square);

            // Update zobrist hash for en passant
            self.game_state.zobrist_hash ^= ZOBRIST_EN_PASSANT[mv.to as usize % 8];
        } else {
            self.game_state.en_passant_file = None;
            self.game_state.en_passant_square = None;
        }
    }
    pub fn unmake_move(&mut self, mv: &MoveData) {
        let moved_piece = mv.piece_to_move;
        if mv.is_promotion() {
            self.remove_piece(mv.to, mv.get_promoted_piece().unwrap());
            self.add_piece(mv.from, moved_piece);
        } else {
            // Restore the piece to its original position
            self.remove_piece(mv.to, moved_piece);
            self.add_piece(mv.from, moved_piece);
        }

        // Restore captured piece if it was a capture move
        if mv.is_capture() {
            self.add_piece(
                mv.get_capture_square().unwrap(),
                mv.get_captured_piece().unwrap(),
            );
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
        self.repetition_table.pop();
        self.turn = self.turn.opposite();
        debug_assert!(self.game_state.zobrist_hash == self.calc_zobrist());
        debug_assert!(self.squares[mv.from as usize] == Some(mv.piece_to_move));
        debug_assert!((!mv.is_capture() && self.squares[mv.to as usize].is_none()) || (mv.is_capture() && self.squares[mv.get_capture_square().unwrap() as usize] == Some(mv.get_captured_piece().unwrap())));
    }

    fn disallow_castling_if_needed(&mut self, square: u8, piece: Piece) {
        if piece.piece_type != PieceType::ROOK {
            return;
        }
        match (square, piece.piece_color) {
            (0, PieceColor::WHITE)
            if self
                .game_state
                .castle_white
                .is_allowed(&CastlingSide::Queenside) =>
                {
                    self.game_state.disallow_castling(
                        AllowedCastling::from(CastlingSide::Queenside),
                        piece.piece_color,
                    );
                }
            (7, PieceColor::WHITE)
            if self
                .game_state
                .castle_white
                .is_allowed(&CastlingSide::Kingside) =>
                {
                    self.game_state.disallow_castling(
                        AllowedCastling::from(CastlingSide::Kingside),
                        piece.piece_color,
                    );
                }
            (56, PieceColor::BLACK)
            if self
                .game_state
                .castle_black
                .is_allowed(&CastlingSide::Queenside) =>
                {
                    self.game_state.disallow_castling(
                        AllowedCastling::from(CastlingSide::Queenside),
                        piece.piece_color,
                    );
                }
            (63, PieceColor::BLACK)
            if self
                .game_state
                .castle_black
                .is_allowed(&CastlingSide::Kingside) =>
                {
                    self.game_state.disallow_castling(
                        AllowedCastling::from(CastlingSide::Kingside),
                        piece.piece_color,
                    );
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
        self.game_state.halfmove_clock = 0;
        self.game_state.fullmove_clock += 1;
        debug_assert!(self.game_state.zobrist_hash == self.calc_zobrist());
    }

    pub fn has_major_or_minor_material(&self) -> bool {
        self.get_color_bitboard(self.turn) ^ (self.get_piece_bitboard(self.turn, KING) | self.get_piece_bitboard(self.turn, PAWN)) != 0
    }
    pub fn unmake_null_move(&mut self) {
        if let Some(previous_state) = self.history.pop() {
            self.game_state = previous_state;
            self.turn = self.turn.opposite();
            debug_assert!(self.game_state.zobrist_hash == self.calc_zobrist());
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
    pub fn get_piece_bitboard(&self, color: PieceColor, piece: PieceType) -> Bitboard {
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
    pub fn calc_zobrist(&self) -> u64 {
        let mut zobrist = 0;
        if self.turn == PieceColor::BLACK {
            zobrist ^= ZOBRIST_SIDE_TO_MOVE;
        }
        if let Some(en_passant_file) = self.game_state.en_passant_file {
            zobrist ^= ZOBRIST_EN_PASSANT[en_passant_file as usize];
        }
        zobrist ^= ZOBRIST_CASTLING[GameState::zobrist_castling_index(
            self.game_state.castle_white,
            self.game_state.castle_black,
        )];

        for sqr in 0..64 {
            if let Some(piece) = self.squares[sqr] {
                let piece_index = piece.piece_color.to_index() * 6 + piece.piece_type.to_index();
                zobrist ^= ZOBRIST_KEYS[piece_index][sqr]
            }
        }

        zobrist
    }
    pub fn calc_eval(&self) -> (i32, i32, i32, i32) {
        let mut eval_white_mg = 0;
        let mut eval_black_mg = 0;
        let mut eval_white_eg = 0;
        let mut eval_black_eg = 0;
        for sqr in 0..64  {
            if let Some(piece) = self.squares[sqr] {
                let psqt = get_psqt(sqr, piece);
                if piece.piece_color == PieceColor::WHITE {
                    eval_white_mg += psqt.get_middle_game();
                    eval_white_eg += psqt.get_end_game();
                } else {
                    eval_black_mg += psqt.get_middle_game();
                    eval_black_eg += psqt.get_end_game();
                }
            }
        }
        (eval_white_mg, eval_white_eg, eval_black_mg, eval_black_eg)
    }
    pub fn calc_gamephase(&self) -> i32 {
        let mut gamephase = 0;
        for sqr in 0..64  {
            if let Some(piece) = self.squares[sqr] {
                gamephase += GAMEPHASE_INC[piece.piece_type as usize];
            }
        }
        gamephase
    }
}
