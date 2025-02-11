use crate::board::board::Board;
use crate::board::piece::PieceColor;
use crate::movegen::generate::generate_moves;
use crate::movegen::movedata::MoveData;
use crate::search::types::ChosenMove;

pub fn eval(board: &Board) ->i32{
    let mg_phase=board.game_phase.min(24);
    let eg_phase=24-mg_phase;
    let mg_score=board.psqt_white.get_middle_game()-board.psqt_black.get_middle_game();
    let eg_score=board.psqt_white.get_end_game()-board.psqt_black.get_end_game();
    let score=(mg_score*mg_phase+eg_score*eg_phase)/24;
     if board.turn==PieceColor::WHITE{
        score
    }
    else{
        -score
    }

}
pub fn search(mut board: &mut Board, depth:u8) ->ChosenMove{
    if depth==0 {
        let curr_eval=eval(board);
        return  ChosenMove::new(MoveData::defualt(),curr_eval);
    }
    let mut best_move=ChosenMove::new(MoveData::defualt(),-10000);
  let moves=generate_moves(&mut board);

    for mov in moves.iter(){
        board.make_move(mov);
        let curr_move=-search(&mut board,depth-1);
        board.unmake_move(mov);
        if curr_move.get_eval()>best_move.get_eval(){
            best_move=ChosenMove::new(*mov, curr_move.get_eval());
        }
    }
    best_move


}