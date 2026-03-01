pub mod generic {
    pub unsafe fn add(acc: *mut i16, weights: *const i16, size: usize) {
        for i in 0..size {
            *acc.add(i) += *weights.add(i);
        }
    }

    pub unsafe fn sub(acc: *mut i16, weights: *const i16, size: usize) {
        for i in 0..size {
            *acc.add(i) -= *weights.add(i);
        }
    }

    pub unsafe fn evaluate_part(acc: *const i16, weights: *const i16, size: usize) -> i32 {
        let mut sum = 0;
        for i in 0..size {
            let x = *acc.add(i);
            let w = *weights.add(i);
            let clipped = (x as i32).clamp(0, 255);
            sum += clipped * clipped * (w as i32);
        }
        sum
    }
}

pub mod sse2 {
    use std::arch::x86_64::*;

    pub unsafe fn add(acc: *mut i16, weights: *const i16, size: usize) {
        for i in (0..size).step_by(8) {
            let a = _mm_loadu_si128(acc.add(i) as *const __m128i);
            let w = _mm_loadu_si128(weights.add(i) as *const __m128i);
            let res = _mm_add_epi16(a, w);
            _mm_storeu_si128(acc.add(i) as *mut __m128i, res);
        }
    }

    pub unsafe fn sub(acc: *mut i16, weights: *const i16, size: usize) {
        for i in (0..size).step_by(8) {
            let a = _mm_loadu_si128(acc.add(i) as *const __m128i);
            let w = _mm_loadu_si128(weights.add(i) as *const __m128i);
            let res = _mm_sub_epi16(a, w);
            _mm_storeu_si128(acc.add(i) as *mut __m128i, res);
        }
    }

    pub unsafe fn evaluate_part(acc: *const i16, weights: *const i16, size: usize) -> i32 {
        let mut sum_vec = _mm_setzero_si128();
        let zero = _mm_setzero_si128();
        let max_v = _mm_set1_epi16(255);

        for i in (0..size).step_by(8) {
            let mut v = _mm_loadu_si128(acc.add(i) as *const __m128i);
            v = _mm_min_epi16(_mm_max_epi16(v, zero), max_v);
            let w = _mm_loadu_si128(weights.add(i) as *const __m128i);

            let v_lo = _mm_unpacklo_epi16(v, zero);
            let v_hi = _mm_unpackhi_epi16(v, zero);
            let w_lo = _mm_srai_epi32(_mm_unpacklo_epi16(_mm_setzero_si128(), w), 16);
            let w_hi = _mm_srai_epi32(_mm_unpackhi_epi16(_mm_setzero_si128(), w), 16);

            let sq_lo = _mm_mullo_epi32(v_lo, v_lo);
            let sq_hi = _mm_mullo_epi32(v_hi, v_hi);

            sum_vec = _mm_add_epi32(sum_vec, _mm_mullo_epi32(sq_lo, w_lo));
            sum_vec = _mm_add_epi32(sum_vec, _mm_mullo_epi32(sq_hi, w_hi));
        }

        let mut res = [0i32; 4];
        _mm_storeu_si128(res.as_mut_ptr() as *mut __m128i, sum_vec);
        res.iter().sum()
    }
}

pub mod avx2 {
    use std::arch::x86_64::*;

    pub unsafe fn add(acc: *mut i16, weights: *const i16, size: usize) {
        for i in (0..size).step_by(16) {
            let a = _mm256_loadu_si256(acc.add(i) as *const __m256i);
            let w = _mm256_loadu_si256(weights.add(i) as *const __m256i);
            let res = _mm256_add_epi16(a, w);
            _mm256_storeu_si256(acc.add(i) as *mut __m256i, res);
        }
    }

    pub unsafe fn sub(acc: *mut i16, weights: *const i16, size: usize) {
        for i in (0..size).step_by(16) {
            let a = _mm256_loadu_si256(acc.add(i) as *const __m256i);
            let w = _mm256_loadu_si256(weights.add(i) as *const __m256i);
            let res = _mm256_sub_epi16(a, w);
            _mm256_storeu_si256(acc.add(i) as *mut __m256i, res);
        }
    }

    pub unsafe fn evaluate_part(acc: *const i16, weights: *const i16, size: usize) -> i32 {
        let mut sum_vec = _mm256_setzero_si256();
        let zero = _mm256_setzero_si256();
        let max_v = _mm256_set1_epi16(255);

        for i in (0..size).step_by(16) {
            let mut v = _mm256_loadu_si256(acc.add(i) as *const __m256i);
            v = _mm256_min_epi16(_mm256_max_epi16(v, zero), max_v);
            let w = _mm256_loadu_si256(weights.add(i) as *const __m256i);

            let v_lo = _mm256_cvtepi16_epi32(_mm256_extracti128_si256(v, 0));
            let v_hi = _mm256_cvtepi16_epi32(_mm256_extracti128_si256(v, 1));
            let w_lo = _mm256_cvtepi16_epi32(_mm256_extracti128_si256(w, 0));
            let w_hi = _mm256_cvtepi16_epi32(_mm256_extracti128_si256(w, 1));

            let sq_lo = _mm256_mullo_epi32(v_lo, v_lo);
            let sq_hi = _mm256_mullo_epi32(v_hi, v_hi);

            sum_vec = _mm256_add_epi32(sum_vec, _mm256_mullo_epi32(sq_lo, w_lo));
            sum_vec = _mm256_add_epi32(sum_vec, _mm256_mullo_epi32(sq_hi, w_hi));
        }

        let h128 = _mm_add_epi32(
            _mm256_castsi256_si128(sum_vec),
            _mm256_extracti128_si256(sum_vec, 1),
        );
        let h64 = _mm_add_epi32(h128, _mm_shuffle_epi32(h128, 0x4E));
        let h32 = _mm_add_epi32(h64, _mm_shuffle_epi32(h64, 0xB1));
        _mm_cvtsi128_si32(h32)
    }
}

pub mod avx512 {
    use std::arch::x86_64::*;

    pub unsafe fn add(acc: *mut i16, weights: *const i16, size: usize) {
        for i in (0..size).step_by(32) {
            let a = _mm512_loadu_si512(acc.add(i) as *const _);
            let w = _mm512_loadu_si512(weights.add(i) as *const _);
            let res = _mm512_add_epi16(a, w);
            _mm512_storeu_si512(acc.add(i) as *mut _, res);
        }
    }

    pub unsafe fn sub(acc: *mut i16, weights: *const i16, size: usize) {
        for i in (0..size).step_by(32) {
            let a = _mm512_loadu_si512(acc.add(i) as *const _);
            let w = _mm512_loadu_si512(weights.add(i) as *const _);
            let res = _mm512_sub_epi16(a, w);
            _mm512_storeu_si512(acc.add(i) as *mut _, res);
        }
    }

    pub unsafe fn evaluate_part(acc: *const i16, weights: *const i16, size: usize) -> i32 {
        let mut total_sum = 0;
        let zero = _mm512_setzero_si512();
        let max_v = _mm512_set1_epi16(255);

        for i in (0..size).step_by(32) {
            let mut v = _mm512_loadu_si512(acc.add(i) as *const _);
            v = _mm512_min_epi16(_mm512_max_epi16(v, zero), max_v);
            let w = _mm512_loadu_si512(weights.add(i) as *const _);

            let v_0 = _mm512_cvtepi16_epi32(_mm512_extracti64x4_epi64(v, 0));
            let v_1 = _mm512_cvtepi16_epi32(_mm512_extracti64x4_epi64(v, 1));
            let w_0 = _mm512_cvtepi16_epi32(_mm512_extracti64x4_epi64(w, 0));
            let w_1 = _mm512_cvtepi16_epi32(_mm512_extracti64x4_epi64(w, 1));

            let p_0 = _mm512_mullo_epi32(_mm512_mullo_epi32(v_0, v_0), w_0);
            let p_1 = _mm512_mullo_epi32(_mm512_mullo_epi32(v_1, v_1), w_1);

            total_sum += _mm512_reduce_add_epi32(p_0) + _mm512_reduce_add_epi32(p_1);
        }
        total_sum
    }
}

#[cfg(target_feature = "avx512f")]
pub use avx512::{add, evaluate_part, sub};

//#[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
//pub use avx2::{add, evaluate_part, sub};

/*#[cfg(all(
    not(target_feature = "sse2"),
    not(target_feature = "avx2"),
    not(target_feature = "avx512f")
))]
 */
pub use generic::{add, evaluate_part, sub};
#[cfg(all(
    target_feature = "sse2",
    not(target_feature = "avx2"),
    not(target_feature = "avx512f")
))]
pub use sse2::{add, evaluate_part, sub};
