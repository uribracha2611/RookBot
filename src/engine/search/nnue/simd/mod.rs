use std::simd::num::SimdInt;
use std::simd::{cmp::SimdOrd, i16x64, i32x64};

pub unsafe fn add(acc: *mut i16, weights: *const i16, size: usize) {
    for i in 0..size {
        unsafe { *acc.add(i) += *weights.add(i) };
    }
}

pub unsafe fn sub(acc: *mut i16, weights: *const i16, size: usize) {
    for i in 0..size {
        unsafe { *acc.add(i) -= *weights.add(i) };
    }
}
pub unsafe fn evaluate_part(acc: &[i16], weights: &[i16], size: usize) -> i32 {
    let mut output = i32x64::splat(0);
    let min = i16x64::splat(0);
    let max = i16x64::splat(255);
    let (acc_chunks, []) = acc[..size].as_chunks::<64>() else {
        unreachable!()
    };
    let (weight_chucks, []) = weights[..size].as_chunks::<64>() else {
        unreachable!();
    };
    for (input, weight) in acc_chunks.iter().zip(weight_chucks.iter()) {
        let input = i16x64::from_array(*input).simd_clamp(min, max);
        let weights = input * i16x64::from_array(*weight);
        output += input.cast::<i32>() * weights.cast::<i32>();
    }
    output.reduce_sum()
}
