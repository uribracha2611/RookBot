use crate::engine::board::board::Board;
use std::io::Write;
use std::num::NonZeroU16;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct BoardPacked {
    piece_bitboard: u64LE,
    pieces_arr: Array32U4,
    stm_ep: u8,
    halfmove: u8,
    fullmove: u16,
    eval: I16LE,
    wdl: u8,
    extra: u8,
}
impl BoardPacked {
    pub fn pack(board: &Board, eval: i16, wdl: u8, extra: u8) -> BoardPacked {
        let pieces = board.pack_pieces_into_viri();
        let occ = board.get_all_pieces_bitboard();
        BoardPacked {
            piece_bitboard: u64LE::new(occ.to_unsinged()),
            pieces_arr: pieces,
            fullmove: board.game_state.fullmove_clock as u16,
            halfmove: board.game_state.halfmove_clock,
            stm_ep: ((board.turn as u8) << 7) | board.game_state.en_passant_square.unwrap_or(64),
            eval: I16LE::new(eval),
            extra,
            wdl,
        }
    }
    pub fn as_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];

        bytes[0..8].copy_from_slice(&self.piece_bitboard.0.to_le_bytes());

        bytes[8..24].copy_from_slice(&self.pieces_arr.0);

        bytes[24] = self.stm_ep;
        bytes[25] = self.halfmove;

        let fm_bytes = self.fullmove.to_le_bytes();
        bytes[26] = fm_bytes[0];
        bytes[27] = fm_bytes[1];

        let eval_bytes = self.eval.0.to_le_bytes();
        bytes[28] = eval_bytes[0];
        bytes[29] = eval_bytes[1];

        bytes[30] = self.wdl;
        bytes[31] = self.extra;

        bytes
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct u64LE(u64);
impl u64LE {
    pub fn new(v: u64) -> Self {
        Self(v.to_le())
    }
    pub fn get(&self) -> u64 {
        u64::from_le(self.0)
    }
}
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct u16LE(u16);

impl u16LE {
    pub fn new(v: u16) -> Self {
        Self(v.to_le())
    }
    pub fn get(&self) -> u16 {
        u16::from_le(self.0)
    }
}
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct I16LE(i16);
impl I16LE {
    pub fn new(v: i16) -> Self {
        Self(v.to_le())
    }
    pub fn get(&self) -> i16 {
        i16::from_le(self.0)
    }
}
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct Array32U4([u8; 16]);
impl Array32U4 {
    pub const fn get(&self, i: usize) -> u8 {
        (self.0[i / 2] >> ((i % 2) * 4)) & 0xF
    }

    pub fn set(&mut self, i: usize, v: u8) {
        self.0[i / 2] |= v << ((i % 2) * 4);
    }
}
pub struct PackedMove {
    move_data: NonZeroU16,
    score: I16LE,
}
impl PackedMove {
    pub fn new(move_data: NonZeroU16, score: i16) -> Self {
        Self {
            move_data,
            score: I16LE::new(score),
        }
    }
    pub fn get_movedata(&self) -> NonZeroU16 {
        self.move_data
    }
    pub fn get_score(&self) -> i16 {
        self.score.get()
    }
}
pub struct Game {
    initial_pos: BoardPacked,
    moves: Vec<PackedMove>,
}
impl Game {
    pub fn new(initial_pos: BoardPacked, moves: Vec<PackedMove>) -> Game {
        Self { initial_pos, moves }
    }
    pub fn get_moves(&self) -> &Vec<PackedMove> {
        &self.moves
    }
    pub fn get_initial_pos(&self) -> BoardPacked {
        self.initial_pos
    }
    pub fn write_to_bin<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.initial_pos.as_bytes())?;

        for packed_move in &self.moves {
            writer.write_all(&packed_move.get_movedata().get().to_le_bytes())?;
            writer.write_all(&packed_move.score.get().to_le_bytes())?;
        }

        writer.write_all(&[0u8; 4])?;

        Ok(())
    }
}
