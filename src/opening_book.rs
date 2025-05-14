use std::collections::HashMap;
use std::sync::LazyLock;
use num_traits::abs;
use rand::Rng;
use crate::engine::board::board::Board;
use crate::engine::board::castling::types::CastlingSide;
use crate::engine::board::piece::{Piece, PieceType};
use crate::engine::movegen::generate::get_pawn_dir;
use crate::engine::movegen::movedata::{CastlingMove, MoveData, MoveType, PromotionCaptureStruct};
use crate::engine::movegen::movedata::MoveType::PromotionCapture;

#[derive(Debug)]
pub struct PolyglotEntry {

    mv: u16,
    weight: u16,
}
impl  PolyglotEntry
{
    pub fn from_file_entry( file_entry: [u8; 16]) -> (u64,PolyglotEntry) {
        let key = u64::from_be_bytes(file_entry[0..8].try_into().unwrap());

        // Extract move (2 bytes)
        let raw_move = u16::from_be_bytes(file_entry[8..10].try_into().unwrap());

        let weight = u16::from_be_bytes(file_entry[10..12].try_into().unwrap());


        (key,PolyglotEntry {
           mv: raw_move,
            weight

        })
    }

}

pub fn decode_polyglot_move(board: &Board,raw_data: u16) -> MoveData {
    let from = (raw_data & 0x3F) as u8;      // Extract bits 0-5
    let to = ((raw_data >> 6) & 0x3F) as u8; // Extract bits 6-11
    let promotion_index = ((raw_data >> 12) & 0xF) as u8; // Extract bits 12-15
    let promote_move_type = match promotion_index {
        0 => None,
        1 => Some(PieceType::KNIGHT),
        2 => Some(PieceType::BISHOP),
        3 => Some(PieceType::ROOK),
        4 => Some(PieceType::QUEEN),
        _ => panic!("Invalid promotion index"),
    };
    let move_type=if promote_move_type.is_some() {
        if let Some(capture_piece) = board.squares[to as usize] {
            PromotionCapture(PromotionCaptureStruct { captured_piece: capture_piece, promoted_piece: Piece::new(board.turn, promote_move_type.unwrap()) })
        } else {
            MoveType::Promotion(Piece::new(board.turn, promote_move_type.unwrap()))
        }
    }

    else if let Some(capture_piece) = board.squares[to as usize] {

            if board.squares[from as usize].unwrap().piece_type==PieceType::PAWN && board.game_state.en_passant_square.is_some() && board.game_state.en_passant_square.unwrap()== to
            {
                let en_passant_target = (to as i8 - (8 * get_pawn_dir(board.turn))) as u8;
                MoveType::EnPassant(board.squares[en_passant_target as usize].unwrap(), en_passant_target)

            } else {
                MoveType::Capture(capture_piece)
            }


    }
        else if from==board.curr_king as u8 && abs(from as i8-to as i8)==2
        {

            MoveType::Castling(CastlingMove {side: CastlingSide::get_castling_from_squares(from, to, board.turn).unwrap(), color: board.turn})
        }
        else {
            MoveType::Normal
        };

    MoveData{from: from, to: to, move_type: move_type,piece_to_move: board.squares[from as usize].unwrap()}



}



pub const OPENING_BOOK_BYTES: &[u8; include_bytes!("polyglot.bin").len()] =
    include_bytes!("polyglot.bin");

pub static OPENING_BOOK_ENTRIES: LazyLock<HashMap<u64, Vec<PolyglotEntry>>> = LazyLock::new(|| {
    let mut entries: HashMap<u64, Vec<PolyglotEntry>> = HashMap::new();
    let bytes = &OPENING_BOOK_BYTES[..];

    for chunk in bytes.chunks_exact(16) {
        let file_entry: [u8; 16] = chunk.try_into().unwrap();
        let (key, entry) = PolyglotEntry::from_file_entry(file_entry);
        entries.entry(key).or_insert_with(Vec::new).push(entry);
    }

    entries
});

pub fn get_move_from_opening_book(board: &Board) -> Option<MoveData> {
    let key = board.game_state.zobrist_hash;

    let entries = OPENING_BOOK_ENTRIES.get(&key)?;


    // Compute total weight
    let total_weight: u16 = entries.iter().map(|entry| entry.weight).sum();

    if total_weight == 0 {
        return None;
    }

    // Choose a random move based on weight
    let mut rng = rand::thread_rng();
    let mut rand_choice = rng.gen_range(0..total_weight);

    for entry in entries {
        if rand_choice < entry.weight {
            return Some(decode_polyglot_move(board, entry.mv));
        }
        rand_choice -= entry.weight;
    }

    None
}
pub fn init_book() {
    let _ = OPENING_BOOK_ENTRIES.get(&0);
}