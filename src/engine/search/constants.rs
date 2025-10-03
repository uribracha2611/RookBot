pub const INFINITY: i32 = 1000000;
pub const VAL_WINDOW: i32 = 50;
pub const FUTILITY_MARGIN_DEPTH: [i32; 2] = [100, 150];

pub const MATE_VALUE: i32 = 10000;
pub const RAZOR_MARGIN: i32 = 300;
pub const RAZOR_DEPTH: i32 = 3;
pub const MAX_EXTENSIONS: i32 = 5;
pub const LMR_TABLE: [[u8; 64]; 64] =
    {
        let mut row_index = 0;
        let mut col_index = 0;
        let mut table = [[0; 64]; 64];
        let raw_info = include_bytes!("../../lmr_table.bin");
        while row_index < 64 {
            col_index = 0;
            while col_index < 64 {
                table[row_index][col_index] = raw_info[row_index * 64 + col_index];
                col_index += 1;
            }
            row_index += 1;
        }
        table
    };
