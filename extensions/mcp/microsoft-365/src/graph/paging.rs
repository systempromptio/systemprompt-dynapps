//! Offset paging that holds on every Graph collection.
//!
//! Outlook collections (mail, calendar, contacts, `OneNote`) honour `$skip`;
//! Teams, the directory and `SharePoint`/`OneDrive` refuse it and only page
//! through `@odata.nextLink`. The fast path hands `$skip`/`$top` to Graph;
//! on that refusal the cursor is walked here and the requested window
//! sliced out, so every tool keeps one `skip`/`top` contract. Headers a
//! query needs (Graph's `ConsistencyLevel: eventual` for advanced queries)
//! ride on every page, the cursor pages included.

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
        let first = self.send(&request(query.clone())).await;
        match first.and_then(|response| response.json()) {
            Ok(data) => {
                let (items, next) = split(data);
                return match cursor_behind_empty_page(&items, next) {
                    Ok(link) => {
                        let rest = Window {
                            skip: 0,
                            top: window.top,
                        };
                        let cursor = headed(GraphRequest::link(link), headers);
                        self.walk(cursor, headers, rest).await
                    },
                    Err(next) => Ok(Page {
                        items,
                        has_more: next.is_some(),
                    }),
                };
            },
            Err(error) if window.skip > 0 && error.rejects_option(&SKIP_UNSUPPORTED) => {},
            Err(error) => return Err(error),
        }
        query.remove("$skip");
        tracing::info!(
            skip = window.skip,
            "Graph endpoint rejects $skip; paging through nextLink"
        );
        match self.walk(request(query.clone()), headers, window).await {
            Err(error) if error.rejects_option(&TOP_UNSUPPORTED) => {
                query.remove("$top");
                self.walk(request(query), headers, window).await
            },
            other => other,
        }
    }

    async fn walk(
        &self,
        first: GraphRequest,
        headers: &[(&str, &str)],
        window: Window,
    ) -> Result<Page, GraphError> {
        let skip = window.skip as usize;
        let needed = window.top.map(|top| skip + top as usize);
        let max_pages = self.limits().max_pages.max(1);
        let mut items = Vec::new();
        let mut request = first;
        let mut pages = 0usize;
        let next = loop {
            let (page, next) = split(self.send(&request).await?.json()?);
            items.extend(page);
            pages += 1;
            let Some(link) = next else { break None };
            if pages >= max_pages || needed.is_some_and(|n| items.len() >= n) {
                break Some(link);
            }
            request = headed(GraphRequest::link(link), headers);
        };
        if next.is_some() && pages >= max_pages {
            tracing::warn!(pages, "Stopped following nextLink at the page cap");
        }
        let end = needed.map_or(items.len(), |n| n.min(items.len()));
        let beyond = items.len() > end;
        let window_items = items
            .into_iter()
            .skip(skip)
            .take(end.saturating_sub(skip))
            .collect();
        Ok(Page {
            items: window_items,
            has_more: next.is_some() || beyond,
        })
    }
}

// Why: MCP clients read an empty page carrying a cursor as "nothing, but
// more" and loop; Graph sends one for Teams member collections, so the
// cursor is followed instead. `Err` hands back the cursor state otherwise.
fn cursor_behind_empty_page(
    items: &[Value],
    next: Option<String>,
) -> Result<String, Option<String>> {
    match next {
        Some(link) if items.is_empty() => Ok(link),
        other => Err(other),
    }
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
