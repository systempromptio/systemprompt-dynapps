//! Offset paging that holds on every Graph collection.
//!
//! Outlook collections (mail, calendar, contacts, `OneNote`) honour `$skip`;
//! Teams, the directory and `SharePoint`/`OneDrive` refuse it and only page
//! through `@odata.nextLink`. The fast path hands `$skip`/`$top` to Graph;
//! on that refusal the cursor is walked here and the requested window
//! sliced out, so every tool keeps one `skip`/`top` contract. The walk asks
//! for pages as large as the collection takes, so a deep `skip` is reached
//! within the page cap, and keeps only the window's items as it goes. A
//! `skip` the cap still cannot reach is an error that tells the model to
//! narrow the query, never an empty page that claims more. Headers a query
//! needs (Graph's `ConsistencyLevel: eventual` for advanced queries) ride on
//! every page, the cursor pages included.

use reqwest::Method;
// JSON: Graph collection items are open-world resources passed through as
// returned.
use serde_json::Value;

use super::{GraphClient, GraphError, GraphQuery, GraphRequest};

const SKIP_UNSUPPORTED: [&str; 3] = [
    "query option 'skip' is not allowed",
    "'$skip' is not supported by the service",
    "$skip is not supported on this api",
];
const TOP_UNSUPPORTED: [&str; 3] = [
    "query option 'top' is not allowed",
    "'$top' is not supported by the service",
    "$top is not supported on this api",
];

// Why: Graph caps `$top` per collection — 999 on directory objects and drive
// items, 50 on Teams channel messages and chats, which answer 400 above it.
// The walk asks for the larger size first (never more than `skip + top`) and
// steps down to the next on a refusal, then to the caller's own `top`.
const WALK_PAGE_SIZES: [u32; 2] = [999, 50];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    pub skip: u32,
    pub top: Option<u32>,
}

// JSON: Graph collection items are open-world resources passed through as
// returned.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Page {
    pub items: Vec<Value>,
    pub has_more: bool,
}

impl GraphClient {
    pub async fn get_collection(
        &self,
        path: &str,
        query: &GraphQuery,
        window: Window,
    ) -> Result<Page, GraphError> {
        self.get_collection_with_headers(path, query, &[], window)
            .await
    }

    pub async fn get_collection_with_headers(
        &self,
        path: &str,
        query: &GraphQuery,
        headers: &[(&str, &str)],
        window: Window,
    ) -> Result<Page, GraphError> {
        let request = |query: GraphQuery| {
            headed(
                GraphRequest::new(Method::GET, path).with_query(query),
                headers,
            )
        };
        let mut query = query.clone();
        if let Some(top) = window.top {
            query.set("$top", top.to_string());
        }
        if window.skip > 0 {
            query.set("$skip", window.skip.to_string());
        }
        match self.first_page(&request(query.clone())).await {
            Ok(first) if cursor_behind_empty_page(&first) => {
                let rest = Window {
                    skip: 0,
                    top: window.top,
                };
                return self.walk(first, headers, rest).await;
            },
            Ok((items, next)) => {
                return Ok(Page {
                    has_more: next.is_some(),
                    items,
                });
            },
            Err(error) if window.skip > 0 && error.rejects_option(&SKIP_UNSUPPORTED) => {},
            Err(error) => return Err(error),
        }
        query.remove("$skip");
        tracing::info!(
            skip = window.skip,
            "Graph endpoint rejects $skip; paging through nextLink"
        );
        self.walk_from_start(&request, query, headers, window).await
    }

    async fn walk_from_start(
        &self,
        request: &(impl Fn(GraphQuery) -> GraphRequest + Sync),
        mut query: GraphQuery,
        headers: &[(&str, &str)],
        window: Window,
    ) -> Result<Page, GraphError> {
        let sizes = walk_page_sizes(window);
        for (rung, size) in sizes.iter().enumerate() {
            match size {
                Some(size) => query.set("$top", size.to_string()),
                None => query.remove("$top"),
            }
            let smaller_left = rung + 1 < sizes.len();
            match self.first_page(&request(query.clone())).await {
                Ok(first) => return self.walk(first, headers, window).await,
                Err(error) if error.rejects_option(&TOP_UNSUPPORTED) => break,
                Err(error) if smaller_left && page_size_refused(&error) => {
                    tracing::info!(
                        page_size = ?size,
                        kind = error.kind(),
                        "Graph refused the walk page size; asking for smaller pages"
                    );
                },
                Err(error) => return Err(error),
            }
        }
        query.remove("$top");
        let first = self.first_page(&request(query)).await?;
        self.walk(first, headers, window).await
    }

    async fn first_page(&self, request: &GraphRequest) -> Result<PageBody, GraphError> {
        Ok(split(self.send(request).await?.json()?))
    }

    async fn walk(
        &self,
        first: PageBody,
        headers: &[(&str, &str)],
        window: Window,
    ) -> Result<Page, GraphError> {
        let skip = window.skip as usize;
        let end = window
            .top
            .map_or(usize::MAX, |top| skip.saturating_add(top as usize));
        let max_pages = self.limits().max_pages.max(1);
        let mut kept = Vec::new();
        let (mut seen, mut pages) = (0usize, 0usize);
        let (mut page, mut next) = first;
        loop {
            pages += 1;
            let len = page.len();
            let from = skip.saturating_sub(seen).min(len);
            let to = end.saturating_sub(seen).min(len);
            kept.extend(page.into_iter().take(to).skip(from));
            seen += len;
            let Some(link) = next.as_deref() else { break };
            if seen >= end || pages >= max_pages {
                break;
            }
            let request = headed(GraphRequest::link(link), headers);
            (page, next) = split(self.send(&request).await?.json()?);
        }
        let capped = next.is_some() && seen < end;
        if capped && seen <= skip {
            tracing::warn!(
                pages,
                seen,
                skip,
                "Page cap reached before the requested skip"
            );
            return Err(out_of_reach(window.skip, pages, seen));
        }
        if capped {
            tracing::warn!(pages, "Stopped following nextLink at the page cap");
        }
        Ok(Page {
            items: kept,
            has_more: next.is_some() || seen > end,
        })
    }
}

// JSON: one Graph collection page — its items and its `@odata.nextLink`.
type PageBody = (Vec<Value>, Option<String>);

fn walk_page_sizes(window: Window) -> Vec<Option<u32>> {
    let Some(top) = window.top else {
        return vec![None];
    };
    let reach = window.skip.saturating_add(top);
    let mut sizes: Vec<u32> = WALK_PAGE_SIZES
        .iter()
        .map(|size| (*size).min(reach))
        .filter(|size| *size > top)
        .collect();
    sizes.dedup();
    sizes.push(top);
    sizes.into_iter().map(Some).collect()
}

const fn page_size_refused(error: &GraphError) -> bool {
    matches!(
        error,
        GraphError::Api { status: 400, .. } | GraphError::TooLarge { .. }
    )
}

fn out_of_reach(skip: u32, pages: usize, seen: usize) -> GraphError {
    GraphError::Invalid(format!(
        "skip={skip} is out of reach: this collection pages only through a cursor, \
         and {pages} pages ({seen} items) did not get there; narrow the request \
         with a filter or search instead of raising skip"
    ))
}

// Why: MCP clients read an empty page carrying a cursor as "nothing, but
// more" and loop; Graph sends one for Teams member collections, so the
// cursor is followed instead.
const fn cursor_behind_empty_page((items, next): &PageBody) -> bool {
    items.is_empty() && next.is_some()
}

fn headed(request: GraphRequest, headers: &[(&str, &str)]) -> GraphRequest {
    headers.iter().fold(request, |request, (name, value)| {
        request.with_header(name, *value)
    })
}

// JSON: a Graph collection body — `value` plus an optional `@odata.nextLink`.
fn split(data: Value) -> (Vec<Value>, Option<String>) {
    let Value::Object(mut body) = data else {
        return (Vec::new(), None);
    };
    let next = body
        .remove("@odata.nextLink")
        .and_then(|link| link.as_str().map(str::to_owned));
    let items = match body.remove("value") {
        Some(Value::Array(items)) => items,
        _ => Vec::new(),
    };
    (items, next)
}
