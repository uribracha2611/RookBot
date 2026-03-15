use crate::engine::board::board::Board;
use crate::engine::board::castling::types::CastlingSide;
use crate::engine::board::piece::PieceColor::WHITE;
use crate::engine::board::piece::{Piece, PieceColor, PieceType};
use crate::engine::board::position::Position;
use std::fmt;
use std::hint::unreachable_unchecked;
use std::num::{NonZeroI16, NonZeroU16};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MoveData {
    data: NonZeroU16,
}
impl MoveData {
    // Define your Wiki/Standard flags
    pub const QUIET: u8 = 0;
    pub const DOUBLE_PUSH: u8 = 1;
    pub const CASTLE_KING: u8 = 2;
    pub const CASTLE_QUEEN: u8 = 3;
    pub const CAPTURE: u8 = 4;
    pub const EN_PASSANT: u8 = 5;
    pub const PROMO_KNIGHT: u8 = 8;
    pub const PROMO_BISHOP: u8 = 9;
    pub const PROMO_ROOK: u8 = 10;
    pub const PROMO_QUEEN: u8 = 11;
    pub const PROMO_KNIGHT_CAP: u8 = 12;
    pub const PROMO_BISHOP_CAP: u8 = 13;
    pub const PROMO_ROOK_CAP: u8 = 14;
    pub const PROMO_QUEEN_CAP: u8 = 15;

    pub fn new(from: u8, to: u8, flags: u8) -> Self {
        let packed =
            ((flags as u16 & 0x0F) << 12) | ((to as u16 & 0x3F) << 6) | (from as u16 & 0x3F);

        unsafe {
            MoveData {
                data: NonZeroU16::new_unchecked(packed),
            }
        }
    }

    pub fn get_promotion_piece(self, color: PieceColor) -> Piece {
        let piece_type = match self.flags() {
            Self::PROMO_KNIGHT | Self::PROMO_KNIGHT_CAP => PieceType::KNIGHT,
            Self::PROMO_BISHOP | Self::PROMO_BISHOP_CAP => PieceType::BISHOP,
            Self::PROMO_ROOK | Self::PROMO_ROOK_CAP => PieceType::ROOK,
            _ => PieceType::QUEEN,
        };

        Piece::new(color, piece_type)
    }

    #[inline(always)]
    fn bits(self) -> u16 {
        self.data.get()
    }
    pub fn move_to_viri_format(&self, turn: PieceColor) -> NonZeroU16 {
        let from = self.from() as u16;
        let mut to = self.to() as u16;
        let mut flag_bits = 0u16;
        let mut promotion_piece = 0u16;

        if self.is_castle() {
            to = self.get_rook_start(turn) as u16;
            flag_bits = 2;
        } else if self.is_en_passant() {
            flag_bits = 1;
        } else if self.is_promotion() {
            flag_bits = 3;
            promotion_piece = match self.get_promotion_piece(turn).piece_type {
                PieceType::KNIGHT => 0,
                PieceType::BISHOP => 1,
                PieceType::ROOK => 2,
                PieceType::QUEEN => 3,
                _ => 0,
            };
        }

        //safaty from/=to from movegen correctness  so either from is A1=0 or To=A1=0 but not both
        unsafe {
            NonZeroU16::new_unchecked(
                from | (to << 6) | (promotion_piece << 12) | (flag_bits << 14),
            )
        }
    }
    pub fn is_castle(self) -> bool {
        (self.flags() & 0b1110) == 0b0010
    }
    pub fn from(self) -> u8 {
        (self.bits() & 0x3F) as u8
    }

    pub fn to(self) -> u8 {
        ((self.bits() >> 6) & 0x3F) as u8
    }

    pub fn flags(self) -> u8 {
        // Shift right by 12 to bring the flags back to the bottom 4 bits
        ((self.bits() >> 12) & 0x0F) as u8
    }
    pub fn get_rook_start(&self, color: PieceColor) -> u8 {
        let is_kside = self.flags() == Self::CASTLE_KING;

        match color {
            PieceColor::WHITE => {
                if is_kside {
                    7
                } else {
                    0
                }
            }
            PieceColor::BLACK => {
                if is_kside {
                    63
                } else {
                    56
                }
            }
        }
    }

    /// Returns the square where the rook lands after jumping over the king.
    pub fn get_rook_end(&self, color: PieceColor) -> u8 {
        let is_kside = self.flags() == Self::CASTLE_KING;

        match color {
            PieceColor::WHITE => {
                if is_kside {
                    5
                } else {
                    3
                }
            }
            PieceColor::BLACK => {
                if is_kside {
                    61
                } else {
                    59
                }
            }
        }
    }

    pub fn is_capture(self) -> bool {
        (self.flags() & 0b0100) != 0
    }

    pub fn is_promotion(self) -> bool {
        (self.flags() & 0b1000) != 0
    }

    pub fn from_algebraic(algebraic: &str, board: &Board) -> Self {
        let notation = algebraic.to_lowercase(); // UCI is usually lowercase

        // 1) Handle special UCI cases for Castling (e1g1, etc.)
        // We can just detect the squares. If the King is moving from e1 to g1, it's a castle.
        let from_sq = Position::from_chess_notation(&notation[0..2])
            .unwrap()
            .to_sqr()
            .unwrap() as u8;
        let to_sq = Position::from_chess_notation(&notation[2..4])
            .unwrap()
            .to_sqr()
            .unwrap() as u8;

        let moving_piece = board.game_state.squares[from_sq as usize].expect("No piece at from square");
        let mut flags = Self::QUIET;

        // 2) Detect Castling by King movement
        if moving_piece.piece_type == PieceType::KING {
            if (from_sq == 4 && to_sq == 6) || (from_sq == 60 && to_sq == 62) {
                flags = Self::CASTLE_KING;
            } else if (from_sq == 4 && to_sq == 2) || (from_sq == 60 && to_sq == 58) {
                flags = Self::CASTLE_QUEEN;
            }
        }

        // 3) Detect Captures (Normal and EP)
        if board.game_state.squares[to_sq as usize].is_some() {
            flags = Self::CAPTURE;
        } else if moving_piece.piece_type == PieceType::PAWN {
            // Diagonal pawn move to empty square is En Passant
            if (from_sq % 8) != (to_sq % 8) {
                flags = Self::EN_PASSANT;
            } else if (to_sq as i8 - from_sq as i8).abs() == 16 {
                flags = Self::DOUBLE_PUSH;
            }
        }

        // 4) Detect Promotion
        if notation.len() > 4 {
            let promo_char = notation.chars().nth(4).unwrap();
            let is_cap = flags == Self::CAPTURE || flags == Self::EN_PASSANT; // Handle Promo-Capture

            flags = match promo_char {
                'n' => Self::PROMO_KNIGHT,
                'b' => Self::PROMO_BISHOP,
                'r' => Self::PROMO_ROOK,
                _ => Self::PROMO_QUEEN,
            };
        }

        Self::new(from_sq, to_sq, flags)
    }
    pub fn is_double_push(&self) -> bool {
        self.flags() == Self::DOUBLE_PUSH
    }

    // Check if the move is an en passant
    pub fn is_en_passant(&self) -> bool {
        self.flags() == Self::EN_PASSANT
    }
    pub fn get_capture_square(&self) -> u8 {
        let to = self.to();

        if self.is_en_passant() {
            return if to >= 40 && to <= 47 { to - 8 } else { to + 8 };
        }

        to
    }
    pub fn get_castling_side(&self) -> Option<CastlingSide> {
        match self.flags() {
            Self::CASTLE_KING => Some(CastlingSide::Kingside),
            Self::CASTLE_QUEEN => Some(CastlingSide::Queenside),
            _ => None,
        }
    }
    pub fn to_algebraic(&self) -> String {
        let from_sq = self.from();
        let to_sq = self.to();

        let from_notation = Position::from_sqr(from_sq as i8)
            .unwrap()
            .to_chess_notation()
            .unwrap();
        let to_notation = Position::from_sqr(to_sq as i8)
            .unwrap()
            .to_chess_notation()
            .unwrap();

        let mut result = format!("{}{}", from_notation, to_notation);

        if self.is_promotion() {
            let promo_char = match self.flags() {
                Self::PROMO_KNIGHT => 'n',
                Self::PROMO_BISHOP => 'b',
                Self::PROMO_ROOK => 'r',
                _ => 'q', // Default to Queen for PROMO_QUEEN or Promo-Captures
            };
            result.push(promo_char);
        }

        result
    }
}
impl fmt::Debug for MoveData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags = self.flags();
        let mut info = String::new();

        // הוספת מידע על סוג המהלך לפי הדגלים
        if self.is_capture() {
            info.push_str(" [Capture]");
        }
        if self.is_castle() {
            info.push_str(" [Castle]");
        }

        if flags == Self::DOUBLE_PUSH {
            info.push_str(" [Double Push]");
        }
        if flags == Self::EN_PASSANT {
            info.push_str(" [En Passant]");
        }

        // הצגת המהלך בפורמט: Move(e2e4 [Capture])
        write!(f, "Move({}{})", self.to_algebraic(), info)
    }
}
impl fmt::Display for MoveData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // פשוט קורא לדיבאג שכבר כתבנו
        write!(f, "{:?}", self)
    }
}
