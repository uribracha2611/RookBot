use {derive_more::Add,derive_more::AddAssign,derive_more::Sub,derive_more::SubAssign};
#[derive(Copy, Clone, SubAssign, AddAssign, Add, Sub)]
pub struct W(i32,i32);
impl W{

    fn new(a:i32,b:i32)->W{
        W(a,b)
    }
    fn get_middle_game(&self)->i32{
        self.0
    }
    fn get_end_game(&self)->i32{
        self.1
    }

}

