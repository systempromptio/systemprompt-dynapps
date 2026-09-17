//! Saturating and lossy numeric conversions used by the analytics rollups.
//!
//! Centralised so the truncation and saturation behaviour of every
//! count-to-float and float-to-count conversion is decided in one place rather
//! than at each call site. The `cast_*` lints are allowed at workspace scope on
//! the understanding that every `as` between an integer and a float lives
//! here: the values are DB counts, durations and display ratios, all far
//! inside the range where the cast is exact or the loss is invisible.

#[must_use]
#[inline]
pub const fn to_f64(v: i64) -> f64 {
    v as f64
}

// Why: `part` of `whole` as a display percentage; zero of nothing is 0.
#[must_use]
#[inline]
pub fn percentage(part: i64, whole: i64) -> f64 {
    if whole == 0 {
        return 0.0;
    }
    (to_f64(part) / to_f64(whole)) * 100.0
}

#[must_use]
#[inline]
pub const fn to_f32(v: f64) -> f32 {
    v as f32
}

#[must_use]
#[inline]
pub fn saturating_i32(v: i64) -> i32 {
    i32::try_from(v).unwrap_or(if v > 0 { i32::MAX } else { i32::MIN })
}

#[must_use]
#[inline]
pub const fn to_f32_from_i64(v: i64) -> f32 {
    v as f64 as f32
}

#[must_use]
#[inline]
pub const fn seconds_to_f64(v: i64) -> f64 {
    v as f64
}

#[must_use]
#[inline]
pub const fn to_usize(v: f64) -> usize {
    v.max(0.0) as usize
}

#[must_use]
#[inline]
pub const fn to_i64(v: f64) -> i64 {
    v as i64
}

#[must_use]
#[inline]
pub const fn round_to_i64(v: f64) -> i64 {
    v.round() as i64
}

#[must_use]
#[inline]
pub const fn i64_to_usize(v: i64) -> usize {
    v as usize
}

#[must_use]
#[inline]
pub const fn f64_rounded_to_i16(v: f64) -> i16 {
    v.round() as i16
}

#[must_use]
#[inline]
pub const fn usize_to_i64(v: usize) -> i64 {
    v as i64
}

#[must_use]
#[inline]
pub const fn usize_to_f64(v: usize) -> f64 {
    v as f64
}
