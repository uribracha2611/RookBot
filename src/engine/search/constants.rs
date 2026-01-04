pub const INFINITY: i32 = 1000000;
pub const VAL_WINDOW: i32 = 50;
pub const FUTILITY_MARGIN_DEPTH: [i32; 2] = [100, 150];

pub const MATE_VALUE: i32 = 10000;
pub const RAZOR_MARGIN: i32 = 300;
pub const RAZOR_DEPTH: i32 = 3;
pub const MAX_EXTENSIONS: i32 = 5;


pub static mut LMR_TABLE: [[i32; 64]; 64] = [[0; 64]; 64];
pub fn init_lmr() {
    for depth in 0..64 {
        for move_count in 0..64 {
            if depth < 3 || move_count < 1 {
                unsafe { LMR_TABLE[depth][move_count] = 1 };
            } else {
                unsafe {
                    LMR_TABLE[depth][move_count] = (0.7844 + ((depth as f64).ln() * (move_count as f64).ln()) / 2.4696).ceil() as i32;
                }
            }
        }
    }
}