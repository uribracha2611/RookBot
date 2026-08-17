use super::{
    bitboard::Bitboard,
    gamestate::GameState,
    piece::{Piece, PieceColor},
};
use crate::engine::board::castling::constants::{
    BLACK_KINGSIDE_ROOK_START, BLACK_QUEENSIDE_ROOK_START, WHITE_KINGSIDE_ROOK_START,
    WHITE_QUEENSIDE_ROOK_START,
};
use crate::engine::board::piece::PieceType;
use crate::engine::board::piece::PieceType::{BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK};
use crate::engine::datagen::format::Array32U4;
use crate::engine::movegen::constants::KNIGHT_MOVES;
use crate::engine::movegen::magic::functions::{get_bishop_attacks, get_rook_attacks};
use crate::engine::movegen::movedata::MoveData;
use crate::engine::search::nnue::NNUE_NETWORK;
use crate::engine::search::nnue::types::{Accumulator, get_feature_indices};
use crate::engine::search::psqt::constants::GAMEPHASE_INC;
use crate::engine::search::psqt::function::get_psqt;
use crate::engine::search::zobrist::constants::{
    ZOBRIST_CASTLING, ZOBRIST_EN_PASSANT, ZOBRIST_KEYS, ZOBRIST_SIDE_TO_MOVE,
};
use crate::engine::{
    board::castling::types::CastlingSide::{Kingside, Queenside},
    movegen::constants::{ALIGN_MASK, KING_MOVES},
};
use crate::engine::{
    board::castling::types::{AllowedCastling, CastlingSide},
    movegen::generate::is_pinned,
};
use crate::engine::{
    board::piece::PieceColor::{BLACK, WHITE},
    movegen::generate::in_check_after_en_passant,
};

#[derive(Clone)]
pub struct Board {
    pub turn: PieceColor,
    pub game_state: GameState,
    history: Vec<GameState>,
    pub repetition_table: Vec<u64>,
}

impl Board {
    fn remove_piece(&mut self, square: u8, piece: Piece) {
        debug_assert!(self.game_state.squares[square as usize] == Some(piece));
        debug_assert!(
            self.get_piece_bitboard(piece.piece_color, piece.piece_type)
                .contains_square(square)
        );
        debug_assert!(
            self.get_color_bitboard(piece.piece_color)
                .contains_square(square)
        );
        let index = 6 * piece.piece_color.to_index() + piece.piece_type.to_index();
        // Update zobrist hash before removing the piece
        self.game_state.zobrist_hash ^= ZOBRIST_KEYS[index][square as usize];
        self.game_state.squares[square as usize] = None;
        self.get_color_bitboard_mut(piece.piece_color)
            .clear_square(square);
        self.get_piece_bitboard_mut(piece.piece_color, piece.piece_type)
            .clear_square(square);
        self.game_state.all_pieces_bitboard.clear_square(square);
        debug_assert!(self.game_state.squares[square as usize].is_none());
        debug_assert!(
            !self
                .get_piece_bitboard(piece.piece_color, piece.piece_type)
                .contains_square(square)
        );
        debug_assert!(
            !self
                .get_color_bitboard(piece.piece_color)
                .contains_square(square)
        );
    }

    fn add_piece(&mut self, square: u8, piece: Piece) {
        debug_assert!(self.game_state.squares[square as usize].is_none());
        debug_assert!(
            !self
                .get_piece_bitboard(piece.piece_color, piece.piece_type)
                .contains_square(square)
        );
        debug_assert!(
            !self
                .get_color_bitboard(piece.piece_color)
                .contains_square(square)
        );
        let index = 6 * piece.piece_color.to_index() + piece.piece_type.to_index();
        // Update zobrist hash before adding the piece
        self.game_state.zobrist_hash ^= ZOBRIST_KEYS[index][square as usize];

        self.game_state.squares[square as usize] = Some(piece);
        self.get_color_bitboard_mut(piece.piece_color)
            .set_square(square);
        self.get_piece_bitboard_mut(piece.piece_color, piece.piece_type)
            .set_square(square);
        self.game_state.all_pieces_bitboard.set_square(square);
        debug_assert!(self.game_state.squares[square as usize] == Some(piece));
        debug_assert!(
            self.get_piece_bitboard(piece.piece_color, piece.piece_type)
                .contains_square(square)
        );
        debug_assert!(
            self.get_color_bitboard(piece.piece_color)
                .contains_square(square)
        );
    }
    pub fn does_king_flip(&self, mv: MoveData) -> bool {
        let from = mv.from() as usize;
        let to = mv.to() as usize;

        if let Some(piece_from) = self.game_state.squares[from]
            && piece_from.piece_type == KING
        {
            return ((from % 8) > 3) != ((to % 8) > 3);
        }

        false
    }
    pub fn rebuild_acc(&self) -> (Accumulator, Accumulator) {
        let mut acc_white = Accumulator::new(&NNUE_NETWORK);
        let mut acc_black = Accumulator::new(&NNUE_NETWORK);
        let white_king = self
            .get_piece_bitboard(WHITE, PieceType::KING)
            .get_single_set_bit() as usize;

        let black_king = self
            .get_piece_bitboard(PieceColor::BLACK, PieceType::KING)
            .get_single_set_bit() as usize;
        for (sq, piece_opt) in self.game_state.squares.iter().enumerate() {
            if let Some(piece) = piece_opt {
                let (white_idx, black_idx) =
                    get_feature_indices(*piece, sq, white_king, black_king);

                acc_white.add_feature(white_idx, &NNUE_NETWORK);
                acc_black.add_feature(black_idx, &NNUE_NETWORK);
            }
        }

        (acc_white, acc_black)
    }

    pub fn detect_pawns_only(&self, piece_color: PieceColor) -> bool {
        self.get_color_bitboard(piece_color) ^ self.get_piece_bitboard(piece_color, PieceType::PAWN)
            == 0
    }
    pub fn is_quiet_move(self: &Board, mv: MoveData) -> bool {
        !mv.is_capture()
            && !mv.is_promotion()
            && !self.game_state.is_check
            && !self.is_move_check(mv)
    }
    pub fn is_move_check(&self, mv: MoveData) -> bool {
        let to = mv.to();
        let piece_moved = self.game_state.squares[mv.from() as usize].unwrap();
        let opp_king = self.get_piece_bitboard(piece_moved.piece_color.opposite(), PieceType::KING);
        let blockers = self.game_state.all_pieces_bitboard & !Bitboard::create_from_square(to);
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

    fn viri_is_castling_right_square(self: &Board, sqr: u8) -> bool {
        if sqr == WHITE_KINGSIDE_ROOK_START && self.game_state.castle_white.is_allowed(&Kingside) {
            return true;
        } else if sqr == WHITE_QUEENSIDE_ROOK_START
            && self.game_state.castle_white.is_allowed(&Queenside)
        {
            return true;
        }

        if sqr == BLACK_KINGSIDE_ROOK_START && self.game_state.castle_black.is_allowed(&Kingside) {
            return true;
        } else if sqr == BLACK_QUEENSIDE_ROOK_START
            && self.game_state.castle_black.is_allowed(&Queenside)
        {
            return true;
        }
        false
    }

    pub fn pack_pieces_into_viri(&self) -> Array32U4 {
        let mut pieces = Array32U4::default();

        let mut occ = self.game_state.all_pieces_bitboard;
        let mut index = 0;
        while occ != 0 {
            let sqr = occ.pop_lsb();
            let piece = self.game_state.squares[sqr as usize].unwrap();
            if self.viri_is_castling_right_square(sqr) {
                pieces.set(index, piece.piece_to_viri(true))
            } else {
                pieces.set(index, piece.piece_to_viri(false))
            }
            index += 1;
        }
        pieces
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

        let mut board = Board {
            turn: if active_color == "w" {
                PieceColor::WHITE
            } else {
                PieceColor::BLACK
            },

            game_state: GameState::from_fen(&game_state_fen),
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
        (board.game_state.acc_white, board.game_state.acc_black) = board.rebuild_acc();
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
            } else if knights == 0
                && white_bishop.pop_count().abs_diff(black_bishop.pop_count()) < 2
                || (white_bishop | white_knight).pop_count() == 1
                    && (black_bishop | black_knight).pop_count() == 1
            {
                return true;
            }
        } else if white_rook.pop_count() == 1 && black_rook == 0 {
            if (white_knight | white_bishop) == 0
                && ((black_knight | black_bishop).pop_count() == 1
                    || (black_knight | black_bishop).pop_count() == 2)
            {
                return true;
            }
        } else if white_rook == 0
            && black_rook.pop_count() == 1
            && (black_knight | black_bishop) == 0
            && ((white_knight | white_bishop).pop_count() == 1
                || (white_knight | white_bishop).pop_count() == 2)
        {
            return true;
        }
        false
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty_count = 0;
            for file in 0..8 {
                if let Some(piece) = self.game_state.squares[rank * 8 + file] {
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
    pub fn make_move(&mut self, mv: MoveData) {
        let old_game_state = self.game_state;
        let is_flip = self.does_king_flip(mv);
        if !is_flip {
            NNUE_NETWORK.handle_mv_nnue(self, mv);
        }
        let moved_piece = self.game_state.squares[mv.from() as usize].unwrap();
        if mv.is_capture() {
            let captured_piece = self.game_state.squares[mv.get_capture_square() as usize].unwrap();
            self.remove_piece(mv.get_capture_square(), captured_piece);
            self.disallow_castling_if_needed(mv.get_capture_square(), captured_piece);
        }
        if mv.is_promotion() {
            self.remove_piece(mv.from(), moved_piece);
            self.add_piece(mv.to(), mv.get_promotion_piece(self.turn));
        } else {
            self.remove_piece(mv.from(), moved_piece);
            self.add_piece(mv.to(), moved_piece);
        }
        if mv.is_castle() {
            let rook_start = mv.get_rook_start(self.turn);
            let rook_end = mv.get_rook_end(self.turn);
            let rook = self.game_state.squares[rook_start as usize].unwrap();
            self.remove_piece(rook_start, rook);
            self.add_piece(rook_end, rook);
            self.game_state
                .disallow_castling_both(moved_piece.piece_color);
        }
        if moved_piece.piece_type == PieceType::KING {
            self.game_state
                .disallow_castling_both(moved_piece.piece_color);
        }
        self.disallow_castling_if_needed(mv.from(), moved_piece);
        self.handle_en_passant(mv);

        self.turn = self.turn.opposite();
        self.game_state.zobrist_hash ^= ZOBRIST_SIDE_TO_MOVE;
        self.game_state.fullmove_clock += 1;
        if moved_piece.piece_type == PieceType::PAWN || mv.is_capture() {
            self.game_state.halfmove_clock = 0;
        } else {
            self.game_state.halfmove_clock += 1;
        }
        if is_flip {
            (self.game_state.acc_white, self.game_state.acc_black) = self.rebuild_acc();
        }
        self.history.push(old_game_state);
        self.repetition_table.push(self.game_state.zobrist_hash);
        debug_assert!(self.game_state.squares[mv.from() as usize].is_none());
        debug_assert!(
            (!mv.is_promotion()
                || self.game_state.squares[mv.to() as usize]
                    == Some(mv.get_promotion_piece(moved_piece.piece_color)))
        );
        debug_assert!(self.game_state.zobrist_hash == self.calc_zobrist());
    }
    pub fn is_board_draw(&self) -> bool {
        self.is_threefold_repetition() || self.game_state.halfmove_clock >= 100
    }
    fn handle_en_passant(&mut self, mv: MoveData) {
        if let Some(file) = self.game_state.en_passant_file {
            // Remove old en passant from zobrist hash
            self.game_state.zobrist_hash ^= ZOBRIST_EN_PASSANT[file as usize];
        }
        if mv.is_double_push() {
            let new_en_passant_square = if self.turn == PieceColor::WHITE {
                mv.to() - 8
            } else {
                mv.to() + 8
            };
            self.game_state.en_passant_file = Some(mv.to() % 8);
            self.game_state.en_passant_square = Some(new_en_passant_square);

            // Update zobrist hash for en passant
            self.game_state.zobrist_hash ^= ZOBRIST_EN_PASSANT[mv.to() as usize % 8];
        } else {
            self.game_state.en_passant_file = None;
            self.game_state.en_passant_square = None;
        }
    }
    #[inline(always)]
    pub fn is_move_legal(&self, mv: MoveData) -> bool {
        let from = mv.from() as usize;
        let to = mv.to() as usize;

        let from_piece = match self.game_state.squares[from] {
            Some(piece) if piece.piece_color == self.turn => piece,
            _ => return false,
        };
        if mv.is_promotion() && from_piece.piece_type != PAWN {
            return false;
        }

        if mv.is_capture() {
            let cap_square = mv.get_capture_square() as usize;
            match self.game_state.squares[cap_square] {
                Some(cap_piece) if cap_piece.piece_color != self.turn => {
                    if mv.is_en_passant()
                        && in_check_after_en_passant(self, from as u8, to as u8, cap_square as u8)
                    {
                        return false;
                    }
                }

                _ => return false,
            }
        }

        let diff_to_from = (mv.to() as i8) - (mv.from() as i8);
        let abs_diff_to_from = diff_to_from.abs();
        match from_piece.piece_type {
            PAWN => {
                if mv.is_capture() {
                    if abs_diff_to_from != 7 && abs_diff_to_from != 9 {
                        return false;
                    }
                } else if mv.is_double_push() {
                    if abs_diff_to_from != 16 {
                        return false;
                    }
                    let intermediate_square = if self.turn == PieceColor::WHITE {
                        from + 8
                    } else {
                        from - 8
                    };
                    if self.game_state.squares[intermediate_square].is_some() {
                        return false;
                    }
                } else {
                    if abs_diff_to_from != 8 {
                        return false;
                    }
                }

                if self.turn == PieceColor::WHITE && diff_to_from < 0 {
                    return false;
                }
                if self.turn == PieceColor::BLACK && diff_to_from > 0 {
                    return false;
                }
            }
            KNIGHT => {
                if !KNIGHT_MOVES[from].contains_square(to as u8) {
                    return false;
                }
            }
            BISHOP => {
                if !get_bishop_attacks(from, self.get_all_pieces_bitboard())
                    .contains_square(to as u8)
                {
                    return false;
                }
            }
            ROOK => {
                if !get_rook_attacks(from, self.get_all_pieces_bitboard()).contains_square(to as u8)
                {
                    return false;
                }
            }
            QUEEN => {
                let blockers = self.get_all_pieces_bitboard();
                let attacks = get_bishop_attacks(from, blockers) | get_rook_attacks(from, blockers);
                if !attacks.contains_square(to as u8) {
                    return false;
                }
            }
            KING => {
                if !mv.is_castle() && !KING_MOVES[from].contains_square(to as u8) {
                    return false;
                }
            }
        }
        if from_piece.piece_type == KING {
            if self.game_state.attacked_square.contains_square(to as u8) {
                return false;
            }
        } else {
            if self.game_state.is_check
                && ((self.game_state.is_double_check
                    || !self.game_state.check_ray.contains_square(to as u8))
                    && !mv.is_en_passant())
            {
                return false;
            }
            if is_pinned(&self, from as u8)
                && ALIGN_MASK[from][self.game_state.curr_king as usize]
                    != ALIGN_MASK[to][self.game_state.curr_king as usize]
            {
                return false;
            }
        }
        if !mv.is_capture() && self.game_state.squares[to].is_some() {
            return false;
        }
        if mv.is_castle() {
            let side = unsafe { mv.get_castling_side().unwrap_unchecked() };
            let rights = if self.turn == PieceColor::WHITE {
                self.game_state.castle_white
            } else {
                self.game_state.castle_black
            };

            if !rights.is_allowed(&side)
                || self.game_state.is_check
                || (self.get_all_pieces_bitboard() & side.required_empty(self.turn) != 0)
                || (self.game_state.attacked_square & side.king_moves_trough(self.turn) != 0)
            {
                return false;
            }
        }

        true
    }
    pub fn unmake_move(&mut self, _mv: MoveData) {
        let old_state = self.history.pop().unwrap();

        self.game_state = old_state;
        self.repetition_table.pop();
        self.turn = self.turn.opposite();
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
    fn disallow_castling_if_needed_hash(&self, hash: &mut u64, square: u8, piece: Piece) {
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
                self.game_state.disallow_castling_hash(
                    hash,
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
                self.game_state.disallow_castling_hash(
                    hash,
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
                self.game_state.disallow_castling_hash(
                    hash,
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
                self.game_state.disallow_castling_hash(
                    hash,
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
        self.get_color_bitboard(self.turn)
            ^ (self.get_piece_bitboard(self.turn, KING) | self.get_piece_bitboard(self.turn, PAWN))
            != 0
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
                if let Some(piece) = self.game_state.squares[rank * 8 + file] {
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
        self.game_state.piece_bitboards[color as usize][piece as usize]
    }
    pub fn get_color_bitboard(&self, color: PieceColor) -> Bitboard {
        self.game_state.color_bitboards[color as usize]
    }
    fn get_piece_bitboard_mut(&mut self, color: PieceColor, piece: PieceType) -> &mut Bitboard {
        &mut self.game_state.piece_bitboards[color as usize][piece as usize]
    }

    fn get_color_bitboard_mut(&mut self, color: PieceColor) -> &mut Bitboard {
        &mut self.game_state.color_bitboards[color as usize]
    }
    pub fn get_all_pieces_bitboard(&self) -> Bitboard {
        self.game_state.all_pieces_bitboard
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
            if let Some(piece) = self.game_state.squares[sqr] {
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
        for sqr in 0..64 {
            if let Some(piece) = self.game_state.squares[sqr] {
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
        for sqr in 0..64 {
            if let Some(piece) = self.game_state.squares[sqr] {
                gamephase += GAMEPHASE_INC[piece.piece_type as usize];
            }
        }
        gamephase
    }
    fn toggle_only_hash_piece(hash: &mut u64, square: u8, piece: Piece) {
        let index = 6 * piece.piece_color.to_index() + piece.piece_type.to_index();
        *hash ^= ZOBRIST_KEYS[index][square as usize];
    }
    pub fn calc_hash_after_move(&self, mv: &MoveData) -> u64 {
        let mut curr_hash = self.game_state.zobrist_hash;
        let moved_piece = self.game_state.squares[mv.from() as usize].unwrap();
        if mv.is_capture() {
            let captured_piece = self.game_state.squares[mv.get_capture_square() as usize].unwrap();
            Self::toggle_only_hash_piece(&mut curr_hash, mv.get_capture_square(), captured_piece);

            self.disallow_castling_if_needed_hash(
                &mut curr_hash,
                mv.get_capture_square(),
                captured_piece,
            );
        }
        if mv.is_promotion() {
            Self::toggle_only_hash_piece(&mut curr_hash, mv.from(), moved_piece);
            Self::toggle_only_hash_piece(
                &mut curr_hash,
                mv.to(),
                mv.get_promotion_piece(self.turn),
            );
        } else {
            Self::toggle_only_hash_piece(&mut curr_hash, mv.from(), moved_piece);
            Self::toggle_only_hash_piece(&mut curr_hash, mv.to(), moved_piece);
        }
        if mv.is_castle() {
            let rook_start = mv.get_rook_start(self.turn);
            let rook_end = mv.get_rook_end(self.turn);
            Self::toggle_only_hash_piece(&mut curr_hash, rook_start, moved_piece);
            Self::toggle_only_hash_piece(&mut curr_hash, rook_end, moved_piece);
            self.game_state.disallow_castling_hash(
                &mut curr_hash,
                AllowedCastling::Kingside,
                moved_piece.piece_color,
            );
            self.game_state.disallow_castling_hash(
                &mut curr_hash,
                AllowedCastling::Queenside,
                moved_piece.piece_color,
            );
        }
        if moved_piece.piece_type == PieceType::KING {
            self.game_state.disallow_castling_hash(
                &mut curr_hash,
                AllowedCastling::Kingside,
                moved_piece.piece_color,
            );
            self.game_state.disallow_castling_hash(
                &mut curr_hash,
                AllowedCastling::Queenside,
                moved_piece.piece_color,
            );
        }
        self.disallow_castling_if_needed_hash(&mut curr_hash, mv.from(), moved_piece);
        if let Some(file) = self.game_state.en_passant_file {
            // Remove old en passant from zobrist hash
            curr_hash ^= ZOBRIST_EN_PASSANT[file as usize];
        }
        if mv.is_double_push() {
            curr_hash ^= ZOBRIST_EN_PASSANT[(mv.to() % 8) as usize];
        }
        curr_hash ^= ZOBRIST_SIDE_TO_MOVE;

        curr_hash
    }
    pub fn calc_hash_after_null_move(&self) -> u64 {
        let mut curr_hash = self.game_state.zobrist_hash;
        curr_hash ^= ZOBRIST_SIDE_TO_MOVE;
        if let Some(file) = self.game_state.en_passant_file {
            curr_hash ^= ZOBRIST_EN_PASSANT[file as usize];
        }
        curr_hash
    }
}
