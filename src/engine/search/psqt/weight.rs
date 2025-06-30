use {derive_more::Add, derive_more::AddAssign, derive_more::Sub, derive_more::SubAssign};
#[derive(Copy, Clone, SubAssign, AddAssign, Add, Sub)]
pub struct W(pub i32, pub i32);
impl W {
    fn new(a: i32, b: i32) -> W {
        W(a, b)
    }
    pub fn get_middle_game(&self) -> i32 {
        self.0
    }
    pub fn get_end_game(&self) -> i32 {
        self.1
    }
}
