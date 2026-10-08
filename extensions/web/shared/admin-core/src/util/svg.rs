//! Pure geometry for the server-rendered SVG charts.
//!
//! Every chart on the analytics pages is drawn in a fixed unit space —
//! `viewBox="0 0 100 40"` stretched with `preserveAspectRatio="none"` and
//! `vector-effect="non-scaling-stroke"` — so all coordinate math happens here
//! in Rust and the Handlebars partials only print attributes. Axis labels stay
//! in HTML gutters exactly like the CSS bar charts, which keeps this module
//! free of text layout entirely.
//!
//! The y scale is never per-series: every series on one chart shares one, or
//! the chart lies.

pub const PLOT_W: f64 = 100.0;
pub const PLOT_H: f64 = 40.0;

#[must_use]
pub fn scale_points(values: &[i64], y_max: i64) -> Vec<(f64, f64)> {
    if values.is_empty() || y_max <= 0 {
        return Vec::new();
    }
    let n = values.len() as f64;
    let max = y_max as f64;
    values
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = (i as f64 + 0.5) / n * PLOT_W;
            let y = (PLOT_H * (1.0 - v as f64 / max)).clamp(0.0, PLOT_H);
            (x, y)
        })
        .collect()
}

#[must_use]
pub fn line_path(points: &[(f64, f64)]) -> String {
    let mut d = String::new();
    for (i, (x, y)) in points.iter().enumerate() {
        let cmd = if i == 0 { 'M' } else { 'L' };
        d.push_str(&format!("{cmd}{x:.2},{y:.2} "));
    }
    d.trim_end().to_owned()
}

#[must_use]
pub fn area_path(points: &[(f64, f64)]) -> String {
    if points.is_empty() {
        return String::new();
    }
    let line = line_path(points);
    let (first_x, _) = points[0];
    let (last_x, _) = points[points.len() - 1];
    format!("{line} L{last_x:.2},{PLOT_H} L{first_x:.2},{PLOT_H} Z")
}

#[must_use]
pub fn ref_line_y(value: i64, y_max: i64) -> Option<f64> {
    if y_max <= 0 || value < 0 || value > y_max {
        return None;
    }
    Some(PLOT_H * (1.0 - value as f64 / y_max as f64))
}

#[must_use]
pub fn cumulative(values: &[i64]) -> Vec<i64> {
    let mut total = 0;
    values
        .iter()
        .map(|v| {
            total += v;
            total
        })
        .collect()
}

#[must_use]
pub fn stack_segments(segment_values: &[i64], y_max: i64) -> Vec<(f64, f64)> {
    if y_max <= 0 {
        return segment_values.iter().map(|_| (PLOT_H, 0.0)).collect();
    }
    let max = y_max as f64;
    let mut base = 0.0; // accumulated height from the baseline, in plot units
    segment_values
        .iter()
        .map(|&v| {
            let h = (v.max(0) as f64 / max * PLOT_H).min(PLOT_H - base);
            base += h;
            (PLOT_H - base, h)
        })
        .collect()
}

#[must_use]
pub fn bar_slots(n: usize, gap_ratio: f64) -> Vec<(f64, f64)> {
    if n == 0 {
        return Vec::new();
    }
    let slot = PLOT_W / n as f64;
    let gap = slot * gap_ratio.clamp(0.0, 0.9);
    let width = slot - gap;
    (0..n)
        .map(|i| ((i as f64).mul_add(slot, gap / 2.0), width))
        .collect()
}

const NICE_STEPS_IN_TENTHS: [i64; 4] = [10, 20, 25, 50];

#[must_use]
pub fn nice_max(raw_max: i64) -> i64 {
    if raw_max <= 0 {
        return 0;
    }
    let mut magnitude: i64 = 1;
    loop {
        for tenths in NICE_STEPS_IN_TENTHS {
            let scaled = tenths.saturating_mul(magnitude);
            let step_is_integral = scaled % 10 == 0;
            if step_is_integral && scaled / 10 >= raw_max {
                return scaled / 10;
            }
        }
        magnitude = magnitude.saturating_mul(10);
    }
}
