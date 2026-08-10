use crate::engine::board::bitboard::Bitboard;
use crate::engine::board::castling::types::{AllowedCastling, CastlingSide};
use crate::engine::board::piece::{Piece, PieceColor};
use crate::engine::board::position::Position;
use crate::engine::search::nnue::NNUE_NETWORK;
use crate::engine::search::nnue::types::Accumulator;
use crate::engine::search::zobrist::constants::{ZOBRIST_CASTLING, ZOBRIST_EN_PASSANT};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GameState {
    pub castle_white: AllowedCastling,
    pub castle_black: AllowedCastling,
    pub halfmove_clock: u8,
    pub fullmove_clock: u32,
    pub en_passant_file: Option<u8>,
    pub en_passant_square: Option<u8>,
    pub zobrist_hash: u64,
    pub is_check: bool,
    pub is_double_check: bool,
    pub check_ray: Bitboard,
    pub acc_white: Accumulator,
    pub acc_black: Accumulator,
    pub pinned_ray: Bitboard,
    pub squares: [Option<Piece>; 64],
    pub color_bitboards: [Bitboard; 2],
    pub piece_bitboards: [[Bitboard; 6]; 2],
    pub all_pieces_bitboard: Bitboard,
    pub attacked_square: Bitboard,
    pub curr_king: u8,
}
impl GameState {
    pub fn new(
        castle_white: AllowedCastling,
        castle_black: AllowedCastling,
        halfmove_clock: u8,
        fullmove_clock: u32,
        en_passant_file: Option<u8>,
        en_passant_square: Option<u8>,
        zobrist_hash: u64,
    ) -> GameState {
        GameState {
            squares: [None; 64],
            castle_white,
            curr_king: 0,
            attacked_square: Bitboard::new(0),
            color_bitboards: [Bitboard::new(0), Bitboard::new(0)],
            piece_bitboards: [[Bitboard::new(0); 6]; 2],
            all_pieces_bitboard: Bitboard::new(0),
            castle_black,
            halfmove_clock,
            fullmove_clock,
            en_passant_file,
            en_passant_square,
            zobrist_hash,
            check_ray: Bitboard::new(u64::MAX),
            pinned_ray: Bitboard::new(0),
            acc_white: Accumulator::new(&NNUE_NETWORK),
            acc_black: Accumulator::new(&NNUE_NETWORK),
            is_double_check: false,
            is_check: false,
        }
    }

    pub fn disallow_castling_hash(self, hash: &mut u64, side: AllowedCastling, color: PieceColor) {
        let (mut old_white_castle, mut old_black_castle) = (self.castle_white, self.castle_black);
        let old_castling_index =
            GameState::zobrist_castling_index(old_white_castle, old_black_castle);

        // Remove the old castling right from the hash
        *hash ^= ZOBRIST_CASTLING[old_castling_index];
        if color == PieceColor::WHITE {
            old_white_castle = old_white_castle.disallow_castling(side);
        } else {
            old_black_castle = old_black_castle.disallow_castling(side);
        }
        let new_castling_index =
            GameState::zobrist_castling_index(old_white_castle, old_black_castle);

        // Remove the old castling right from the hash
        *hash ^= ZOBRIST_CASTLING[new_castling_index];
    }
    pub fn disallow_castling(&mut self, side: AllowedCastling, color: PieceColor) {
        let (old_white_castle, old_black_castle) = (self.castle_white, self.castle_black);
        let old_castling_index =
            GameState::zobrist_castling_index(old_white_castle, old_black_castle);

        // Remove the old castling right from the hash
        self.zobrist_hash ^= ZOBRIST_CASTLING[old_castling_index];

        if color == PieceColor::WHITE {
            self.castle_white = self.castle_white.disallow_castling(side);
        } else {
            self.castle_black = self.castle_black.disallow_castling(side);
        }
        let (new_white_castle, new_black_castle) = (self.castle_white, self.castle_black);
        let old_castling_index =
            GameState::zobrist_castling_index(new_white_castle, new_black_castle);

        // Add the new castling right to the hash
        self.zobrist_hash ^= ZOBRIST_CASTLING[old_castling_index];
    }

    pub fn disallow_castling_both(&mut self, color: PieceColor) {
        if color == PieceColor::WHITE {
            if self.castle_white.is_allowed(&CastlingSide::Kingside) {
                self.disallow_castling(AllowedCastling::Kingside, color);
            }
            if self.castle_white.is_allowed(&CastlingSide::Queenside) {
                self.disallow_castling(AllowedCastling::Queenside, color);
            }
        } else {
            if self.castle_black.is_allowed(&CastlingSide::Kingside) {
                self.disallow_castling(AllowedCastling::Kingside, color);
            }
            if self.castle_black.is_allowed(&CastlingSide::Queenside) {
                self.disallow_castling(AllowedCastling::Queenside, color);
            }
        }
    }
    pub fn zobrist_castling_index(
        castle_white: AllowedCastling,
        castle_black: AllowedCastling,
    ) -> usize {
        // Map each enum variant to an integer 0..3
        let to_index = |c: AllowedCastling| -> usize {
            match c {
                AllowedCastling::Both => 0,
                AllowedCastling::Queenside => 1,
                AllowedCastling::Kingside => 2,
                AllowedCastling::None => 3,
            }
        };

        let white_index = to_index(castle_white);
        let black_index = to_index(castle_black);

        white_index * 4 + black_index
    }

    pub fn init_zobrist_hash(&mut self) {
        self.zobrist_hash = 0;
        self.zobrist_hash ^= ZOBRIST_CASTLING
            [GameState::zobrist_castling_index(self.castle_white, self.castle_black)];

        // Add en passant square to the hash
        if let Some(file) = self.en_passant_file {
            self.zobrist_hash ^= ZOBRIST_EN_PASSANT[file as usize];
        }
    }

    pub fn from_fen(fen: &str) -> Self {
        let parts: Vec<&str> = fen.split_whitespace().collect();

        // Ensure correct length of FEN parts
        if parts.len() < 4 {
            panic!("Invalid FEN string: insufficient parts");
        }

        let castle_rights = parts[0];
        let en_passant = parts[1];

        // Parse en passant field safely
        let (en_passant_file, en_passant_square) = if en_passant == "-" {
            (None, None)
        } else if en_passant.len() == 2 {
            let file = en_passant.chars().next().unwrap() as u8 - b'a';
            let rank = en_passant.chars().nth(1).unwrap().to_digit(10).unwrap() as u8 - 1;
            (Some(file), Some(rank * 8 + file))
        } else {
            panic!("Invalid en passant field in FEN string");
        };
        let mut game_state = GameState {
            squares: [None; 64],
            castle_white: AllowedCastling::from_fen(castle_rights, PieceColor::WHITE),
            castle_black: AllowedCastling::from_fen(castle_rights, PieceColor::BLACK),
            halfmove_clock: parts[2]
                .parse()
                .unwrap_or_else(|_| panic!("Invalid halfmove clock in FEN string")),
            fullmove_clock: parts[3]
                .parse()
                .unwrap_or_else(|_| panic!("Invalid fullmove clock in FEN string")),
            en_passant_file,
            en_passant_square,
            zobrist_hash: 0,
            check_ray: Bitboard::new(u64::MAX),
            pinned_ray: Bitboard::new(0),
            attacked_square: Bitboard::new(0),
            curr_king: 0,
            acc_white: Accumulator::new(&NNUE_NETWORK),
            acc_black: Accumulator::new(&NNUE_NETWORK),
            is_double_check: false,
            is_check: false,
            color_bitboards: [Bitboard::new(0), Bitboard::new(0)],
            piece_bitboards: [[Bitboard::new(0); 6]; 2],
            all_pieces_bitboard: Bitboard::new(0),
        };

        game_state.init_zobrist_hash();
        game_state
    }
    pub fn to_fen(&self) -> String {
        // Convert GameState to FEN string
        let mut castling_rights = String::new();
        castling_rights.push_str(&self.castle_white.to_fen(PieceColor::WHITE));
        castling_rights.push_str(&self.castle_black.to_fen(PieceColor::BLACK));
        if castling_rights.is_empty() {
            castling_rights.push('-');
        }

        // Convert en_passant_square to algebraic notation
        let en_passant = if let Some(sqr) = self.en_passant_square {
            Position::from_index(sqr as i8)
                .and_then(|pos| pos.to_chess_notation())
                .unwrap_or_else(|| "-".to_string())
        } else {
            "-".to_string()
        };

        format!(
            "{} {} {} {}",
            castling_rights, en_passant, self.halfmove_clock, self.fullmove_clock,
        )
    }
    pub fn to_stockfish_string(&self) -> String {
        // Convert GameState to Stockfish formatted string
        format!(
            "Castle rights: {}{}\nHalfmove clock: {}\nFullmove clock: {}\nEn passant: {}{}",
            self.castle_white.to_fen(PieceColor::WHITE),
            self.castle_black.to_fen(PieceColor::BLACK),
            self.halfmove_clock,
            self.fullmove_clock,
            self.en_passant_file
                .map_or("-".to_string(), |f| ((f + b'a') as char).to_string()),
            self.en_passant_square
                .map_or("-".to_string(), |s| s.to_string())
        )
    }
}
