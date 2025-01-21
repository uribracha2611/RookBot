use crate::board::position::Position;
use crate::board::{
    castling::types::CastlingSide,
    piece::{Piece, PieceColor},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MoveType {
    Normal,
    Capture(Piece),
    Castling(CastlingMove), // Using struct for castling
    Promotion(Piece),
    PromotionCapture(PromotionCapture),
    EnPassant(Piece, u8), // Piece and the square of the captured pawn
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CastlingMove {
    pub side: CastlingSide,
    pub color: PieceColor,
}
impl CastlingMove {
    pub fn new(side: CastlingSide, color: PieceColor) -> CastlingMove {
        CastlingMove { side, color }
    }
    pub fn get_rook_end(&self) -> u8 {
        self.side.rook_end(self.color)
    }
    pub fn get_rook_start(&self) -> u8 {
        self.side.rook_start(self.color)
    }
    
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PromotionCapture {
    pub captured_piece: Piece,
    pub promoted_piece: Piece,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MoveData {
    pub from: u8,
    pub to: u8,
    pub piece_to_move: Piece,
    move_type: MoveType,
}

impl MoveData {
    // Constructor to create a new MoveData instance from algebraic notation
    pub fn new(
        from: u8,
        to: u8,
        piece_to_move: Piece,
        move_type: MoveType,
    ) ->  MoveData{
        // Convert algebraic notation to board indices


        MoveData {
            from,
            to,
            piece_to_move,
            move_type,
        }
    }

    // Check if the move is a capture
    pub fn is_capture(&self) -> bool {
        matches!(
            self.move_type,
            MoveType::Capture(_) | MoveType::EnPassant(_, _)
        )
    }

    // Check if the move is a castling move
    pub fn is_castling(&self) -> bool {
        matches!(self.move_type, MoveType::Castling(_))
    }

    // Check if the move is a promotion
    pub fn is_promotion(&self) -> bool {
        matches!(
            self.move_type,
            MoveType::Promotion(_) | MoveType::PromotionCapture(_)
        )
    }
    pub fn is_double_push(&self) -> bool {
        let from_pos = Position::from_sqr(self.from as i8).unwrap();
        let to_pos = Position::from_sqr(self.to as i8).unwrap();
        let from_rank = from_pos.y;
        let to_rank = to_pos.y;
        let diff = (to_rank  - from_rank ).abs();
        diff == 2
    }

    // Check if the move is an en passant
    pub fn is_en_passant(&self) -> bool {
        matches!(self.move_type, MoveType::EnPassant(_, _))
    }
    pub fn get_capture_square(&self) -> Option<u8> {
        match &self.move_type {
            MoveType::Capture(_) => Some(self.to),
            MoveType::EnPassant(_, square) => Some(*square),
            _ => None,
        }
    }
    pub fn get_rook_start(&self) -> Option<u8> {
        match &self.move_type {
            MoveType::Castling(castling) => Some(castling.get_rook_start()),
            _ => None,
        }
    }
    pub fn get_castling_side(&self) -> Option<CastlingSide> {
        match &self.move_type {
            MoveType::Castling(castling) => Some(castling.side),
            _ => None,
        }
    }
    pub fn get_rook_end(&self) -> Option<u8> {
        match &self.move_type {
            MoveType::Castling(castling) => Some(castling.get_rook_end()),
            _ => None,
        }
    }
    // Convert the move to algebraic notation
    pub fn to_algebraic(&self) -> String {
        let from_pos = Position::from_sqr(self.from as i8).unwrap();
        let to_pos = Position::from_sqr(self.to as i8).unwrap();

        let from_notation = from_pos.to_chess_notation().unwrap();
        let to_notation = to_pos.to_chess_notation().unwrap();

        match &self.move_type {
            MoveType::Capture(_) => format!("{}x{}", from_notation, to_notation),
            MoveType::Promotion(piece) => {
                format!("{}{}={:?}", from_notation, to_notation, piece.piece_type)
            }
            MoveType::PromotionCapture(promo) => {
                format!(
                    "{}x{}={:?}",
                    from_notation, to_notation, promo.promoted_piece.piece_type
                )
            }
            MoveType::EnPassant(_, _) => format!("{}x{} e.p.", from_notation, to_notation),
            _ => format!("{}{}", from_notation, to_notation),
        }
    }

    // Get the captured piece if it's a capture move
    pub fn get_captured_piece(&self) -> Option<Piece> {
        match &self.move_type {
            MoveType::Capture(piece) => Some(piece.clone()),
            MoveType::EnPassant(piece, _) => Some(piece.clone()),
            MoveType::PromotionCapture(ref promo_capture) => {
                Some(promo_capture.captured_piece.clone())
            }
            _ => None,
        }
    }

    // Get the promoted piece if it's a promotion move
    pub fn get_promoted_piece(&self) -> Option<Piece> {
        match &self.move_type {
            MoveType::Promotion(piece) => Some(piece.clone()),
            MoveType::PromotionCapture(ref promo_capture) => {
                Some(promo_capture.promoted_piece.clone())
            }
            _ => None,
        }
    }
}
