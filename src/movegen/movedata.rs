use crate::board::position::Position;
use crate::board::{
    castling::types::CastlingSide,
    piece::{Piece, PieceColor},
};
use crate::board::board::Board;
use crate::board::castling::constants::{BLACK_KINGSIDE_KING_END, BLACK_KINGSIDE_KING_START, BLACK_QUEENSIDE_KING_END, BLACK_QUEENSIDE_KING_START, WHITE_KINGSIDE_KING_END, WHITE_KINGSIDE_KING_START, WHITE_QUEENSIDE_KING_END, WHITE_QUEENSIDE_KING_START};
use crate::board::piece::PieceType;

#[derive(Clone, Copy, PartialEq, Eq,Debug)]
pub enum MoveType {
    Normal,
    Capture(Piece),
    Castling(CastlingMove), // Using struct for castling
    Promotion(Piece),
    PromotionCapture(PromotionCaptureStruct),
    EnPassant(Piece, u8), // Piece and the square of the captured pawn
}

#[derive(Clone, Copy, PartialEq, Eq,Debug)]
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
#[derive(Clone, Copy, PartialEq, Eq,Debug)]
pub struct PromotionCaptureStruct {
    pub captured_piece: Piece,
    pub promoted_piece: Piece,
}

#[derive(Clone, Copy, PartialEq, Eq,Debug)]
pub struct MoveData {
    pub from: u8,
    pub to: u8,
    pub piece_to_move: Piece,
    pub(crate) move_type: MoveType,
}

impl MoveData {
    pub fn defualt() -> MoveData {
        MoveData {
            from: 0,
            to: 0,
            piece_to_move: Piece::new(PieceColor::WHITE, PieceType::PAWN),
            move_type: MoveType::Normal,
        }
    }
    pub fn new (from: u8, to: u8, piece_to_move: Piece, move_type: MoveType) -> MoveData {
        MoveData {
            from,
            to,
            piece_to_move,
            move_type,
        }
    }



    pub fn from_algebraic(algebraic: &str, board: &Board) -> Self {
        let notation = algebraic.to_uppercase();
        if notation == "E1G1" && (board.squares[4].unwrap().piece_type == PieceType::KING && board.squares[4].unwrap().piece_color == PieceColor::WHITE) && (board.squares[7].unwrap().piece_type == PieceType::ROOK && board.squares[7].unwrap().piece_color == PieceColor::WHITE) {
            MoveData {
                from: WHITE_KINGSIDE_KING_START,
                to: WHITE_KINGSIDE_KING_END,
                piece_to_move: board.squares[4].unwrap(),
                move_type: MoveType::Castling(CastlingMove::new(CastlingSide::Kingside, PieceColor::WHITE)),
            }
        } else if notation == "E1C1" && (board.squares[4].unwrap().piece_type == PieceType::KING && board.squares[4].unwrap().piece_color == PieceColor::WHITE) && (board.squares[0].unwrap().piece_type == PieceType::ROOK && board.squares[0].unwrap().piece_color == PieceColor::WHITE) {
            MoveData {
                from: WHITE_QUEENSIDE_KING_START,
                to: WHITE_QUEENSIDE_KING_END,
                piece_to_move: board.squares[4].unwrap(),
                move_type: MoveType::Castling(CastlingMove::new(CastlingSide::Queenside, PieceColor::WHITE)),
            }
        } else if notation == "E8G8" && (board.squares[60].unwrap().piece_type == PieceType::KING && board.squares[60].unwrap().piece_color == PieceColor::BLACK) && (board.squares[63].unwrap().piece_type == PieceType::ROOK && board.squares[63].unwrap().piece_color == PieceColor::BLACK) {
            MoveData {
                from: BLACK_KINGSIDE_KING_START,
                to: BLACK_KINGSIDE_KING_END,
                piece_to_move: board.squares[60].unwrap(),
                move_type: MoveType::Castling(CastlingMove::new(CastlingSide::Kingside, PieceColor::BLACK)),
            }
        } else if notation == "E8C8" && (board.squares[60].unwrap().piece_type == PieceType::KING && board.squares[60].unwrap().piece_color == PieceColor::BLACK) && (board.squares[56].unwrap().piece_type == PieceType::ROOK && board.squares[56].unwrap().piece_color == PieceColor::BLACK) {
            MoveData {
                from: BLACK_QUEENSIDE_KING_START,
                to: BLACK_QUEENSIDE_KING_END,
                piece_to_move: board.squares[60].unwrap(),
                move_type: MoveType::Castling(CastlingMove::new(CastlingSide::Queenside, PieceColor::BLACK)),
            }
        }
         else {

            // 2) Parse normal moves
            let from_pos = Position::from_chess_notation(&algebraic[0..2]).expect("\\Invalid from-square");
            let to_pos = Position::from_chess_notation(&algebraic[2..4]).expect("\\Invalid to-square");
            let from_sq = from_pos.to_sqr().unwrap() as usize;
            let to_sq = to_pos.to_sqr().unwrap() as usize;
            let moving_piece = board.squares[from_sq].unwrap();
            let promotion_part = if algebraic.len() > 4 { &algebraic[4..] } else { "" };

            // 3) Check promotion
            let mut move_type = if !promotion_part.is_empty(){
                let promo_char = promotion_part.chars().nth(0).unwrap_or('Q');
                let promo_type = match promo_char.to_ascii_uppercase() {
                    'N' => PieceType::KNIGHT,
                    'B' => PieceType::BISHOP,
                    'R' => PieceType::ROOK,
                    _ => PieceType::QUEEN,
                };
                if let Some(captured_piece) = board.squares[to_sq] {
                    MoveType::PromotionCapture(PromotionCaptureStruct {
                        captured_piece,
                        promoted_piece: Piece::new(moving_piece.piece_color, promo_type),
                    })
                } else {
                    MoveType::Promotion(Piece::new(moving_piece.piece_color, promo_type))
                }
            } else if board.squares[to_sq].is_some() {
                MoveType::Capture(board.squares[to_sq].unwrap())
            } else {
                MoveType::Normal
            };


            // 4) Check en passant (pawns capturing diagonally on empty square)
            if moving_piece.piece_type == PieceType::PAWN {
                let file_diff = (from_sq % 8) as i8 - (to_sq % 8) as i8;
                if file_diff.abs() == 1 && board.squares[to_sq].is_none() {
                    if let Some(ep_square) = board.game_state.en_passant_square {
                        
                        if ep_square as usize == to_sq {
                            let en_passant_target = (ep_square as i8 - (8 * crate::movegen::generate::get_pawn_dir(board.turn))) as u8;    
                            let captured_color = moving_piece.piece_color.opposite();
                            let captured_piece = Piece::new(captured_color, PieceType::PAWN);
                            move_type = MoveType::EnPassant(captured_piece, en_passant_target);
                   
                        }
                    }
                }
            }

            MoveData {
                from: from_sq as u8,
                to: to_sq as u8,
                piece_to_move: moving_piece,
                move_type,
            }
        }
    }
  

    // Check if the move is a capture
    pub fn is_capture(&self) -> bool {
        matches!(
            self.move_type,
            MoveType::Capture(_) | MoveType::EnPassant(_, _) | MoveType::PromotionCapture(_)
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
            MoveType::PromotionCapture(promo_capture) => Some(self.to),
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
            MoveType::Promotion(piece) => {
                format!("{}{}{}", from_notation, to_notation, piece.piece_type.to_char().to_lowercase())
            }
            MoveType::PromotionCapture(promo) => {
                format!(
                    "{}{}{}",
                    from_notation, to_notation, promo.promoted_piece.piece_type.to_char().to_lowercase()
                )
            }
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
