use std::ops::Neg;
use crate::board::board::Board;
use crate::movegen::movedata::MoveData;

#[derive(Clone, Copy)]
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
pub struct SearchOutput {
    pub(crate) nodes_evaluated: i32,
    pub(crate) principal_variation: Vec<MoveData>,
    pub(crate) eval: i32,
    pub (crate) depth:i32
}

impl SearchOutput {
    pub fn new(nodes_evaluated: i32, principal_variation: Vec<MoveData>,eval:i32,depth:i32) -> SearchOutput {
        SearchOutput {
            nodes_evaluated,
            principal_variation,
            eval,
            depth
            
        }
    }

    pub fn get_nodes_evaluated(&self) -> i32 {
        self.nodes_evaluated
    }

    pub fn get_principal_variation(&self) -> &Vec<MoveData> {
        &self.principal_variation
    }
}
pub struct SearchInput{
    pub(crate) depth:u8,

}


