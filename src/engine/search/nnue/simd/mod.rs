use std::simd::num::SimdInt;
use std::simd::{cmp::SimdOrd, i16x64, i32x64};

pub unsafe fn add_1_sub_1(
    acc: *mut i16,
    weights_add: *const i16,
    weights_sub: *const i16,
    size: usize,
) {
    for i in 0..size {
        unsafe { *acc.add(i) += *weights_add.add(i) - *weights_sub.add(i)  };
    }
}
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
pub unsafe fn add_1_sub_2(
    acc: *mut i16,
    weights_add: *const i16,
    weights_sub_1: *const i16,
    weights_sub_2: *const i16,
    size: usize,
) {
    for i in 0..size {
        unsafe {
            *acc.add(i) += *weights_add.add(i) - *weights_sub_1.add(i) - *weights_sub_2.add(i) 
        };
    }
}
pub unsafe fn add_2_sub_2(
    acc: *mut i16,
    weights_add_1: *const i16,
    weights_add_2: *const i16,
    weights_sub_1: *const i16,
    weights_sub_2: *const i16,
    size: usize,
) {
    for i in 0..size {
        unsafe {
            *acc.add(i) += *weights_add_1.add(i) + *weights_add_2.add(i)
                - *weights_sub_1.add(i)
                - *weights_sub_2.add(i) 
        };
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
