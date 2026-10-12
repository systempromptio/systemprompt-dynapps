//! The run's timeline strip.
//!
//! Two lanes and a tick row over a fixed 1000-unit viewBox, so every x is a
//! permille of the run's own window and the SVG stretches to the card with
//! no script. Assistant turns sit on top, each lasting until the next event;
//! tool calls below, each a bar from its `tool_use` to its `tool_result` (or
//! to the end of the window while it is still running); hooks and stderr
//! lines are ticks beneath. Every mark links to its event on the rail.

use serde::Serialize;
use systemprompt_web_admin_core::ssr::format::format_duration_ms;

use super::RunEventView;

pub const VIEW_WIDTH: f64 = 1000.0;
const LANE_HEIGHT: f64 = 16.0;
const LANE_GAP: f64 = 6.0;
const TICK_HEIGHT: f64 = 10.0;
const TOOL_Y: f64 = LANE_HEIGHT + LANE_GAP;
const TICK_Y: f64 = TOOL_Y + LANE_HEIGHT + LANE_GAP;
pub(crate) const VIEW_HEIGHT: f64 = TICK_Y + TICK_HEIGHT;
pub const MIN_MARK_WIDTH: f64 = 3.0;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunTimelineView {
    pub view_width: f64,
    pub view_height: f64,
    pub marks: Vec<TimelineMark>,
    pub ticks: Vec<TimelineTick>,
    pub axis: Vec<AxisLabel>,
    pub has_marks: bool,
    pub window_label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TimelineMark {
    pub seq: i64,
    pub lane: &'static str,
    pub tone: &'static str,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TimelineTick {
    pub seq: i64,
    pub kind: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AxisLabel {
    pub x: f64,
    pub label: String,
}

pub fn timeline_view(events: &[RunEventView], total_ms: i64) -> RunTimelineView {
    let last = events.iter().map(|e| e.offset_ms).max().unwrap_or(0);
    let window = total_ms.max(last).max(1);
    let mut marks = Vec::new();
    let mut ticks = Vec::new();
    for (i, e) in events.iter().enumerate() {
        match e.kind.as_str() {
            "assistant_text" => {
                let end = events.get(i + 1).map_or(window, |n| n.offset_ms);
                marks.push(mark(
                    e,
                    Lane::new("assistant", "ok", 0.0),
                    (e.offset_ms, end),
                    window,
                ));
            },
            "tool_use" => {
                let result = e
                    .paired_seq
                    .and_then(|s| events.iter().find(|r| r.seq == s));
                let (end, tone) = match result {
                    Some(r) if r.is_error => (r.offset_ms, "err"),
                    Some(r) => (r.offset_ms, "ok"),
                    None => (window, "open"),
                };
                marks.push(mark(
                    e,
                    Lane::new("tool", tone, TOOL_Y),
                    (e.offset_ms, end),
                    window,
                ));
            },
            "hook" | "stderr" => ticks.push(TimelineTick {
                seq: e.seq,
                kind: e.kind.clone(),
                label: format!("{} · {}", e.title, e.offset_label),
                x: scale(e.offset_ms, window).min(VIEW_WIDTH - 2.0),
                y: TICK_Y,
                height: TICK_HEIGHT,
            }),
            _ => {},
        }
    }
    RunTimelineView {
        view_width: VIEW_WIDTH,
        view_height: VIEW_HEIGHT,
        has_marks: !marks.is_empty() || !ticks.is_empty(),
        marks,
        ticks,
        axis: axis(window),
        window_label: format_duration_ms(window),
    }
}

#[derive(Debug, Clone, Copy)]
struct Lane {
    name: &'static str,
    tone: &'static str,
    y: f64,
}

impl Lane {
    const fn new(name: &'static str, tone: &'static str, y: f64) -> Self {
        Self { name, tone, y }
    }
}

fn mark(e: &RunEventView, lane: Lane, (start_ms, end_ms): (i64, i64), window: i64) -> TimelineMark {
    let Lane {
        name: lane,
        tone,
        y,
    } = lane;
    let x = scale(start_ms, window).min(VIEW_WIDTH - MIN_MARK_WIDTH);
    let width = (scale(end_ms.max(start_ms), window) - x)
        .max(MIN_MARK_WIDTH)
        .min(VIEW_WIDTH - x);
    TimelineMark {
        seq: e.seq,
        lane,
        tone,
        label: format!(
            "{} · {} · {}",
            e.title,
            e.offset_label,
            format_duration_ms((end_ms - start_ms).max(0))
        ),
        x,
        y,
        width: round2(width),
        height: LANE_HEIGHT,
    }
}

fn axis(window: i64) -> Vec<AxisLabel> {
    (0..=4)
        .map(|i| {
            let fraction = f64::from(i) / 4.0;
            AxisLabel {
                x: round2(fraction * VIEW_WIDTH),
                label: format_duration_ms((window as f64 * fraction).round() as i64),
            }
        })
        .collect()
}

pub fn scale(ms: i64, window: i64) -> f64 {
    round2(ms.max(0) as f64 / window.max(1) as f64 * VIEW_WIDTH)
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
