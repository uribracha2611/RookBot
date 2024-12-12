use super::{castling::types::AllowedCastling, piece::PieceColor};

#[derive(Clone, Copy,PartialEq, Eq)]
pub struct GameState{
   pub  castle_white:AllowedCastling,
   pub  castle_black:AllowedCastling,
    pub halfmove_clock:u8,
   pub  fullmove_clock:u8,
pub     en_passant_file:Option<u8>,
   pub  en_passant_square:Option<u8>,    

}
impl GameState{ 
  pub  fn new(castle_white:AllowedCastling,castle_black:AllowedCastling,halfmove_clock:u8,fullmove_clock:u8,en_passant_file:Option<u8>,en_passant_square:Option<u8>)->GameState{
      GameState{castle_white,castle_black,halfmove_clock,fullmove_clock,en_passant_file,en_passant_square}

  }
  pub fn disallow_castling(&mut self,side: AllowedCastling  , color:PieceColor){
      if color == PieceColor::WHITE{
          self.castle_white = self.castle_white.disallow_castling(side);
      }else{
          self.castle_black = self.castle_black.disallow_castling(side);
      }
  }
  pub fn disallow_castling_both(&mut self,color:PieceColor){
      if color == PieceColor::WHITE{
          self.castle_white = AllowedCastling::None;
      }else{
          self.castle_black = AllowedCastling::None;
      }
  }
    
}