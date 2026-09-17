//! Walking a page-only repository read up to the export cap.

use std::future::Future;

// Why: some reads offer pages of a fixed size and no "all" variant. Rather
// than a second query per table, the export walks the pages until it holds
// `limit` rows or the read runs dry; the total comes from the first page.
pub(crate) async fn collect_pages<T, F, Fut>(
    limit: i64,
    page_size: u32,
    mut read: F,
) -> Result<(Vec<T>, i64), sqlx::Error>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<(Vec<T>, i64), sqlx::Error>>,
{
    let mut out: Vec<T> = Vec::new();
    let mut total = 0;
    let mut page = 1;
    loop {
        let (rows, page_total) = read(page).await?;
        if page == 1 {
            total = page_total;
        }
        let got = rows.len();
        out.extend(rows);
        let want = usize::try_from(limit).unwrap_or(usize::MAX);
        if got < crate::numeric::i64_to_usize(i64::from(page_size)) || out.len() >= want {
            out.truncate(want);
            return Ok((out, total));
        }
        page += 1;
    }
}
