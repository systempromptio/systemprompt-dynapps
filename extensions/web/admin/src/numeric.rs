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
pub const fn round_to_i64(v: f64) -> i64 {
    v.round() as i64
}
