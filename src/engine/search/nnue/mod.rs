use crate::engine::search::nnue::types::Network;
pub mod simd;
pub mod types;
pub static NNUE_NETWORK: Network =
    unsafe { std::mem::transmute(*include_bytes!("./nets/nnue_256_v3.bin")) };
