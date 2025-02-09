use crate::board::castling::types::CastlingSide;
use super::{castling::types::AllowedCastling, piece::PieceColor};
use crate::board::position::Position;
use crate::search::zobrist::constants::{ZOBRIST_CASTLING, ZOBRIST_EN_PASSANT};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GameState {
    pub castle_white: AllowedCastling,
    pub castle_black: AllowedCastling,
    pub halfmove_clock: u8,
    pub fullmove_clock: u8,
    pub en_passant_file: Option<u8>,
    pub en_passant_square: Option<u8>,
    pub zobrist_hash: u64,
}
impl GameState {
    pub fn new(
        castle_white: AllowedCastling,
        castle_black: AllowedCastling,
        halfmove_clock: u8,
        fullmove_clock: u8,
        en_passant_file: Option<u8>,
        en_passant_square: Option<u8>,
        zobrist_hash: u64,
    ) -> GameState {
        GameState {
            castle_white,
            castle_black,
            halfmove_clock,
            fullmove_clock,
            en_passant_file,
            en_passant_square,
            zobrist_hash,
        }
    }

       pub fn disallow_castling(&mut self, side: AllowedCastling, color: PieceColor) {
           let old_hash = self.zobrist_hash;
           let castling_index = match (color, side) {
               (PieceColor::WHITE, AllowedCastling::Kingside) => 0,
               (PieceColor::WHITE, AllowedCastling::Queenside) => 1,
               (PieceColor::BLACK, AllowedCastling::Kingside) => 2,
               (PieceColor::BLACK, AllowedCastling::Queenside) => 3,
               _ => return,
           };

           // Remove the old castling right from the hash
           self.zobrist_hash ^= ZOBRIST_CASTLING[castling_index];

           if color == PieceColor::WHITE {
               self.castle_white = self.castle_white.disallow_castling(side);
           } else {
               self.castle_black = self.castle_black.disallow_castling(side);
           }

           // Add the new castling right to the hash
           self.zobrist_hash ^= ZOBRIST_CASTLING[castling_index];

       }

pub fn disallow_castling_both(&mut self, color: PieceColor) {
    let old_hash = self.zobrist_hash;

    if color == PieceColor::WHITE {
        if self.castle_white.is_allowed(&CastlingSide::Kingside) {
            self.zobrist_hash ^= ZOBRIST_CASTLING[0];
        }
        if self.castle_white.is_allowed(&CastlingSide::Queenside) {
            self.zobrist_hash ^= ZOBRIST_CASTLING[1];
        }
        self.castle_white = AllowedCastling::None;
    } else {
        if self.castle_black.is_allowed(&CastlingSide::Kingside) {
            self.zobrist_hash ^= ZOBRIST_CASTLING[2];
        }
        if self.castle_black.is_allowed(&CastlingSide::Queenside) {
            self.zobrist_hash ^= ZOBRIST_CASTLING[3];
        }
        self.castle_black = AllowedCastling::None;
    }


}

    pub fn init_zobrist_hash(&mut self) {
        self.zobrist_hash = 0;

        // Add castling rights to the hash
        if self.castle_white.is_allowed(&CastlingSide::Kingside) {
            self.zobrist_hash ^= ZOBRIST_CASTLING[0];
        }
        if self.castle_white.is_allowed(&CastlingSide::Queenside) {
            self.zobrist_hash ^= ZOBRIST_CASTLING[1];
        }
        if self.castle_black.is_allowed(&CastlingSide::Kingside) {
            self.zobrist_hash ^= ZOBRIST_CASTLING[2];
        }
        if self.castle_black.is_allowed(&CastlingSide::Queenside) {
            self.zobrist_hash ^= ZOBRIST_CASTLING[3];
        }

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
