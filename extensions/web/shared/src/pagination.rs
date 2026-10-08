//! Page-window arithmetic for paginated list views.

/// `index` is zero-based; `shown_rows` counts what this page renders, so the
/// last page is short.
#[derive(Debug, Clone, Copy)]
pub struct PageWindow {
    pub index: i64,
    pub size: i64,
    pub total_pages: i64,
    pub total_rows: i64,
    pub shown_rows: i64,
    pub noun: &'static str,
}

impl PageWindow {
    pub const fn new(
        index: i64,
        size: i64,
        total_rows: i64,
        shown_rows: i64,
        noun: &'static str,
    ) -> Self {
        let total_pages = if total_rows == 0 {
            1
        } else {
            (total_rows + size - 1) / size
        };
        Self {
            index,
            size,
            total_pages,
            total_rows,
            shown_rows,
            noun,
        }
    }

    pub const fn bounds(self) -> (i64, i64) {
        if self.shown_rows == 0 {
            return (0, 0);
        }
        let first = self.index * self.size + 1;
        (first, self.index * self.size + self.shown_rows)
    }
}
