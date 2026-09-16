//! Paging and the hash filter for the Versions pages: one query type that
//! validates the page number and rebuilds every link from the whole query, so
//! a filter survives paging.

use serde::Deserialize;

use crate::error::AdminError;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RevisionPagination {
    page: Option<u32>,
    pub(super) hash: Option<String>,
}

impl RevisionPagination {
    // Why: paging must not silently drop the hash filter, so every page link is
    // built from the whole query rather than from the page number alone.
    pub(super) fn page_href(&self, page: i64) -> String {
        self.hash.as_deref().map_or_else(
            || format!("?page={page}"),
            |hash| format!("?page={page}&hash={}", urlencoding::encode(hash)),
        )
    }

    // Why: the filter form submits `?hash=` with an empty value when cleared by
    // hand, which must read as no filter rather than as a filter nothing matches.
    pub(super) fn normalize(&mut self) {
        self.hash = self
            .hash
            .take()
            .map(|hash| hash.trim().to_owned())
            .filter(|hash| !hash.is_empty());
    }

    pub(super) fn offset(&self) -> Result<i64, AdminError> {
        let page = self.page.unwrap_or(1);
        if !(1..=10_001).contains(&page) {
            return Err(AdminError::BadRequest("Invalid page".to_owned()));
        }
        Ok(i64::from(page - 1) * 50)
    }
}
