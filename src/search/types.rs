use std::ops::Neg;
use crate::movegen::movedata::MoveData;

pub struct ChosenMove{
     mv: MoveData,
    eval: i32
}
impl ChosenMove {
    pub(crate) fn new(mv: MoveData, eval: i32) -> ChosenMove {
        ChosenMove {
            mv,
            eval
        }
    }
    pub fn get_move(&self) -> MoveData {
        self.mv
    }
    pub fn get_eval(&self) -> i32 {
        self.eval
    }

}
impl Neg for ChosenMove {
    type Output = ChosenMove;

    fn neg(self) -> Self::Output {
        ChosenMove {
            mv: self.mv,
            eval: -self.eval
        }
    }
}