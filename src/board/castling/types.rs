use crate::board::piece::PieceColor;

use super::constants::*;

pub enum CastlingSide {
    Kingside,
    Queenside,
}
    impl CastlingSide {
        pub const fn rook_start(&self, color: PieceColor) -> u8 {
            match (color, self) {
                (PieceColor::WHITE, CastlingSide::Kingside) => WHITE_KINGSIDE_ROOK_START,
                (PieceColor::WHITE, CastlingSide::Queenside) => WHITE_QUEENSIDE_ROOK_START,
                (PieceColor::BLACK, CastlingSide::Kingside) => BLACK_KINGSIDE_ROOK_START,
                (PieceColor::BLACK, CastlingSide::Queenside) => BLACK_QUEENSIDE_ROOK_START,
            }
        }
    
        pub const fn rook_end(&self, color: PieceColor) -> u8 {
            match (color, self) {
                (PieceColor::WHITE, CastlingSide::Kingside) => WHITE_KINGSIDE_ROOK_END,
                (PieceColor::WHITE, CastlingSide::Queenside) => WHITE_QUEENSIDE_ROOK_END,
                (PieceColor::BLACK, CastlingSide::Kingside) => BLACK_KINGSIDE_ROOK_END,
                (PieceColor::BLACK, CastlingSide::Queenside) => BLACK_QUEENSIDE_ROOK_END,
            }
        }
    
        pub const fn king_start(&self, color: PieceColor) -> u8 {
            match (color, self) {
                (PieceColor::WHITE, CastlingSide::Kingside) | (PieceColor::WHITE, CastlingSide::Queenside) => WHITE_KINGSIDE_KING_START,
                (PieceColor::BLACK, CastlingSide::Kingside) | (PieceColor::BLACK, CastlingSide::Queenside) => BLACK_KINGSIDE_KING_START,
            }
        }
    
        pub const fn king_end(&self, color: PieceColor) -> u8 {
            match (color, self) {
                (PieceColor::WHITE, CastlingSide::Kingside) => WHITE_KINGSIDE_KING_END,
                (PieceColor::WHITE, CastlingSide::Queenside) => WHITE_QUEENSIDE_KING_END,
                (PieceColor::BLACK, CastlingSide::Kingside) => BLACK_KINGSIDE_KING_END,
                (PieceColor::BLACK, CastlingSide::Queenside) => BLACK_QUEENSIDE_KING_END,
            }
        }
    
        pub const fn required_empty(&self, color: PieceColor) -> u64 {
            match (color, self) {
                (PieceColor::WHITE, CastlingSide::Kingside) => WHITE_KINGSIDE_REQUIRED_EMPTY,
                (PieceColor::WHITE, CastlingSide::Queenside) => WHITE_QUEENSIDE_REQUIRED_EMPTY,
                (PieceColor::BLACK, CastlingSide::Kingside) => BLACK_KINGSIDE_REQUIRED_EMPTY,
                (PieceColor::BLACK, CastlingSide::Queenside) => BLACK_QUEENSIDE_REQUIRED_EMPTY,
            }
        }
    }
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum AllowedCastling {
        Kingside,
        Queenside,
        Both,
        None,
    }
    
    impl AllowedCastling {
        /// Disallow kingside castling, returning the updated state.
        pub fn disallow_kingside(&self) -> AllowedCastling {
            match self {
                AllowedCastling::Kingside => AllowedCastling::None,
                AllowedCastling::Both => AllowedCastling::Queenside,
                _ => *self,
            }
        }
    
        /// Disallow queenside castling, returning the updated state.
        pub fn disallow_queenside(&self) -> AllowedCastling {
            match self {
                AllowedCastling::Queenside => AllowedCastling::None,
                AllowedCastling::Both => AllowedCastling::Kingside,
                _ => *self,
            }
        }
    
        /// Check if castling is allowed.
        pub fn is_allowed(&self, side: CastlingSide) -> bool {
            match (self, side) {
                (AllowedCastling::Kingside, CastlingSide::Kingside) => true,
                (AllowedCastling::Queenside, CastlingSide::Queenside) => true,
                (AllowedCastling::Both, _) => true,
                _ => false,
            }
        }
    
       
       pub fn disallow_castling(&self, side: AllowedCastling) -> AllowedCastling {
        
         match (self, side) {
             (AllowedCastling::Kingside, AllowedCastling::Kingside) => AllowedCastling::None,
             (AllowedCastling::Queenside, AllowedCastling::Queenside) => AllowedCastling::None,
             (AllowedCastling::Both, AllowedCastling::Both) => AllowedCastling::None,
             (AllowedCastling::Both, AllowedCastling::Kingside) => AllowedCastling::Queenside,
             (AllowedCastling::Both, AllowedCastling::Queenside) => AllowedCastling::Kingside,
             (AllowedCastling::None, _) => AllowedCastling::None,
             (_, AllowedCastling::None) => AllowedCastling::None,
             _ => *self
             
         }
       }
    }
    
    