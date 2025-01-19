use crate::board::piece::PieceType;
use super::{
    bitboard::Bitboard,
    gamestate::GameState,
    piece::{Piece, PieceColor},
};

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
}

impl Board {
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
                    squares[rank * 8 + file] = Piece::from_fen(&c.to_string());
                    file += 1;
                }
            }
        }

        // Parse the game state using the updated GameState::from_fen
        let game_state = GameState::from_fen(&game_state_fen);

        // Set the side to move
        let turn = if active_color == "w" {
            PieceColor::WHITE
        } else {
            PieceColor::BLACK
        };

        // Initialize bitboards (color and piece-specific)
        let mut color_bitboards = [Bitboard::new(0), Bitboard::new(0)];
        let mut piece_bitboards = [[Bitboard::new(0); 6]; 2];
        let mut all_pieces_bitboard = Bitboard::new(0);

        for (i, square) in squares.iter().enumerate() {
            if let Some(piece) = square {
                let color_index = piece.piece_color as usize;
                let piece_index = piece.piece_type as usize;

                color_bitboards[color_index].set_square(i as u8);
                piece_bitboards[color_index][piece_index].set_square(i as u8);
                all_pieces_bitboard.set_square(i as u8);
            }
        }

        // Set en passant based on the game state
        let en_passant = game_state.en_passant_square.map(|sq| sq as usize);

        // Return a fully initialized Board
        Board {
            squares,
            turn,
            color_bitboards,
            piece_bitboards,
            all_pieces_bitboard,
            game_state,
            is_check: false,
            is_double_check: false,
            attacked_square:Bitboard::new(0),
            curr_king:0,
            check_ray:Bitboard::new(u64::MAX),
            pinned_ray:Bitboard::new(0),
        }
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
        stockfish_str
    }
    pub  fn get_piece_bitboard(&self, color: PieceColor, piece: PieceType) -> Bitboard {
        self.piece_bitboards[color as usize][piece as usize]
    }
    pub fn get_color_bitboard(&self, color: PieceColor) -> Bitboard {
        self.color_bitboards[color as usize]
    }
    pub fn get_all_pieces_bitboard(&self) -> Bitboard {
        self.all_pieces_bitboard
    }
}
