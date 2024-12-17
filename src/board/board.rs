use super::{bitboard::Bitboard, gamestate::GameState, piece::{Piece, PieceColor}};

pub struct Board {
    squares: [Option<Piece>; 64],
    turn: PieceColor,
    pub color_bitboards: [Bitboard; 2],
    pub piece_bitboards: [[Bitboard; 6]; 2],
    pub all_pieces_bitboard: Bitboard,
    pub en_passant: Option<usize>,
    pub game_state: GameState,
    pub is_check: bool,
    pub is_double_check: bool,
   
}

impl Board {
    pub fn from_fen(fen: &str) -> Self {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        let piece_placement = parts[0];
        let active_color = parts[1];
        let game_state_fen = parts[2..].join(" ");

        let mut squares = [None; 64];
        let mut rank = 7;
        let mut file = 0;

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

        let game_state = GameState::from_fen(&game_state_fen);

        let turn = if active_color == "w" {
            PieceColor::WHITE
        } else {
            PieceColor::BLACK
        };

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

        Board {
            squares,
            turn,
            color_bitboards,
            piece_bitboards,
            all_pieces_bitboard,
            en_passant: None,
            game_state,
            is_check: false,
            is_double_check: false,
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
}