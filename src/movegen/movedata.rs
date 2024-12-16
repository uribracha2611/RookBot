use crate::board::{castling::types::CastlingSide, piece::{Piece, PieceColor, PieceType}};
use crate::board::position::Position;

pub enum MoveType {
    Normal,
    Capture(Piece),
    Castling(CastlingMove),  // Using struct for castling
    Promotion(Piece),
    PromotionCapture(PromotionCapture),
    EnPassant(Piece, u8),  // Piece and the file of the captured pawn
}

pub struct CastlingMove {
    pub side: CastlingSide,
    pub color: PieceColor,
}

pub struct PromotionCapture {
    pub captured_piece: Piece,
    pub promoted_piece: Piece,
}

pub struct MoveData {
    pub from: u8,
    pub to: u8,
    pub piece_to_move: Piece,
    move_type: MoveType,
}

impl MoveData {
    // Constructor to create a new MoveData instance from algebraic notation
    pub fn new(from: &str, to: &str, piece_to_move: Piece, move_type: MoveType) -> Result<MoveData, &'static str> {
        // Convert algebraic notation to board indices
        let from_pos = Position::from_chess_notation(from)?;
        let to_pos = Position::from_chess_notation(to)?;

        let from_sqr = from_pos.to_sqr();
        let to_sqr = to_pos.to_sqr();

        Ok(MoveData {
            from: from_sqr,
            to: to_sqr,
            piece_to_move,
            move_type,
        })
    }

    // Check if the move is a capture
    pub fn is_capture(&self) -> bool {
        matches!(self.move_type, MoveType::Capture(_) | MoveType::EnPassant(_, _))
    }

    // Check if the move is a castling move
    pub fn is_castling(&self) -> bool {
        matches!(self.move_type, MoveType::Castling(_))
    }

    // Check if the move is a promotion
    pub fn is_promotion(&self) -> bool {
        matches!(self.move_type, MoveType::Promotion(_) | MoveType::PromotionCapture(_))
    }

    // Check if the move is an en passant
    pub fn is_en_passant(&self) -> bool {
        matches!(self.move_type, MoveType::EnPassant(_, _))
    }

    // Convert the move to algebraic notation
    pub fn to_algebraic(&self) -> String {
        let from_pos = Position::from_sqr(self.from).unwrap();
        let to_pos = Position::from_sqr(self.to).unwrap();

        let from_notation = from_pos.to_chess_notation().unwrap();
        let to_notation = to_pos.to_chess_notation().unwrap();

        match &self.move_type {
            MoveType::Capture(_) => format!("{}x{}", from_notation, to_notation),
            MoveType::Promotion(piece) => format!("{}{}={:?}", from_notation, to_notation, piece.piece_type),
            MoveType::PromotionCapture(promo) => {
                format!("{}x{}={:?}", from_notation, to_notation, promo.promoted_piece.piece_type)
            },
            MoveType::EnPassant(_, _) => format!("{}x{} e.p.", from_notation, to_notation),
            _ => format!("{}{}", from_notation, to_notation),
        }
    }

    // Get the captured piece if it's a capture move
    pub fn get_captured_piece(&self) -> Option<Piece> {
        match &self.move_type {
            MoveType::Capture(piece) => Some(piece.clone()),
            MoveType::EnPassant(piece, _) => Some(piece.clone()),
            MoveType::PromotionCapture(ref promo_capture) => Some(promo_capture.captured_piece.clone()),
            _ => None,
        }
    }

    // Get the promoted piece if it's a promotion move
    pub fn get_promoted_piece(&self) -> Option<Piece> {
        match &self.move_type {
            MoveType::Promotion(piece) => Some(piece.clone()),
            MoveType::PromotionCapture(ref promo_capture) => Some(promo_capture.promoted_piece.clone()),
            _ => None,
        }
    }
}