//! View-model builders for the ratings panel on the session detail page:
//! the human ratings people left.

use crate::handlers::ssr::analysis::tone::score_tone;
use crate::handlers::ssr::format::{local_time, short_id};
use crate::repositories::analytics::session_quality::SessionRatingRow;

use super::context::RatingView;

pub(super) fn rating_view(r: &SessionRatingRow) -> RatingView {
    let user_label = r
        .display_name
        .clone()
        .unwrap_or_else(|| short_id(r.user_id.as_str()));
    RatingView {
        user_url: format!("/admin/users/{}", urlencoding::encode(r.user_id.as_str())),
        user_id: r.user_id.clone(),
        user_label,
        rating: r.rating,
        stars: stars(r.rating),
        tone: score_tone(r.rating * 20),
        outcome: if r.outcome.is_empty() {
            "—".to_owned()
        } else {
            r.outcome.clone()
        },
        notes: if r.notes.is_empty() {
            "—".to_owned()
        } else {
            r.notes.clone()
        },
        created_at_local: local_time(r.created_at),
    }
}

// Why: Mean of the ratings, to one decimal. `None` when nobody has rated yet,
// so the panel shows the count only rather than an average of nothing.
pub(super) fn rating_average(ratings: &[SessionRatingRow]) -> Option<String> {
    if ratings.is_empty() {
        return None;
    }
    let sum: i64 = ratings.iter().map(|r| i64::from(r.rating)).sum();
    let mean = sum as f64 / ratings.len() as f64;
    Some(format!("{mean:.1}"))
}

fn stars(rating: i16) -> String {
    let filled = rating.clamp(0, 5);
    let empty = 5 - filled;
    "★".repeat(filled as usize) + &"☆".repeat(empty as usize)
}
