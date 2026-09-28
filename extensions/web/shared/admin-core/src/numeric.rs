//! Saturating and lossy numeric conversions used by the analytics rollups.
//!
//! These are the conversions worth naming: a count to a float, a float back to
//! a count, and a ratio as a display percentage. Their behaviour on truncation
//! and saturation is decided here rather than at each call site.
//!
//! This module is **not** the only place an `as` between an integer and a float
//! appears under `extensions/web/admin` — there are around a hundred others, in
//! display formatting that divides a count by a unit. The `cast_*` lints are
//! allowed at workspace scope for that reason, not because this module has them
//! all: the values are DB counts, durations and display ratios, all far inside
//! the range where the cast is exact or the loss is invisible. Do not read the
//! allowance as a claim that every cast was routed through here.

#[must_use]
#[inline]
pub const fn to_f64(v: i64) -> f64 {
    v as f64
}

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
