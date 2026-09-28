//! The data model an exportable table declares.
//!
//! A [`DataSet`] is a table the console renders: a typed column list and a
//! loader that produces rows for a window, a filter and a viewer. Every
//! serialiser, the preview count and the export dialog are written once
//! against this model, so a page gains export by declaring its columns and
//! pointing at the repository function it already calls.
//!
//! Cells are typed rather than pre-formatted strings: a cost is a microdollar
//! ledger integer, a timestamp is a `DateTime`, and each format decides how
//! to write them. CSV writes cost as an exact six-decimal decimal because
//! these files feed finance spreadsheets, where a rounded `f64` would drift.
//!
//! A dataset reads the page's own query type back out of the request
//! ([`ExportContext::query`]), so the filters a page understands are exactly
//! the filters its export understands — there is no second parameter list.
//! A [`Table`]'s `total` counts every row the filter matched, so it exceeds
//! `rows.len()` when a cap cut the file and the preview can say so.
//!
//! [`Window`] names the contract a dataset reads under: `Live` is
//! `?preset=&from=&to=` (up to 90 days), `Retained` the snapshot pipeline's
//! UTC days (up to a year), `Days` a fixed preset only, `Month` is
//! `?month=YYYY-MM`.

use async_trait::async_trait;
use axum::http::Uri;
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::types::UserContext;

pub(crate) const HARD_CAP: i64 = 50_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CellKind {
    Text,
    Integer,
    Decimal,
    Money,
    Timestamp,
    Bool,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub(crate) struct Column {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: CellKind,
    pub default: bool,
    pub group: &'static str,
}

pub(crate) const UNGROUPED: &str = "Other";

impl Column {
    pub(crate) const fn new(key: &'static str, label: &'static str, kind: CellKind) -> Self {
        Self {
            key,
            label,
            kind,
            default: true,
            group: UNGROUPED,
        }
    }

    pub(crate) const fn group(self, name: &'static str) -> Self {
        Self {
            group: name,
            ..self
        }
    }

    pub(crate) const fn optional(self) -> Self {
        Self {
            default: false,
            ..self
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Cell {
    Empty,
    Text(String),
    Integer(i64),
    Decimal(f64),
    Money(i64),
    Timestamp(DateTime<Utc>),
    Bool(bool),
}

impl Cell {
    pub(crate) fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    pub(crate) fn opt_text(value: Option<impl Into<String>>) -> Self {
        value.map_or(Self::Empty, Self::text)
    }

    pub(crate) fn opt_int(value: Option<impl Into<i64>>) -> Self {
        value.map_or(Self::Empty, |v| Self::Integer(v.into()))
    }

    pub(crate) fn opt_decimal(value: Option<f64>) -> Self {
        value.map_or(Self::Empty, Self::Decimal)
    }

    pub(crate) fn opt_time(value: Option<DateTime<Utc>>) -> Self {
        value.map_or(Self::Empty, Self::Timestamp)
    }

    pub(crate) fn list(values: &[String]) -> Self {
        if values.is_empty() {
            Self::Empty
        } else {
            Self::Text(values.join("; "))
        }
    }
}

impl From<&str> for Cell {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<String> for Cell {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<i64> for Cell {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<i32> for Cell {
    fn from(value: i32) -> Self {
        Self::Integer(i64::from(value))
    }
}

impl From<bool> for Cell {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<DateTime<Utc>> for Cell {
    fn from(value: DateTime<Utc>) -> Self {
        Self::Timestamp(value)
    }
}

#[derive(Debug, Default)]
pub(crate) struct Table {
    pub rows: Vec<Vec<Cell>>,
    pub total: i64,
}

impl Table {
    pub(crate) fn complete(rows: Vec<Vec<Cell>>) -> Self {
        let total = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        Self { rows, total }
    }

    pub(crate) const fn capped_at(rows: Vec<Vec<Cell>>, total: i64) -> Self {
        Self { rows, total }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Window {
    Live,
    Retained,
    Days,
    Month,
    None,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ExportWindow {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub clamped: bool,
}

impl ExportWindow {
    pub(crate) fn whole_days_back_from_now(&self) -> i32 {
        i32::try_from((self.to - self.from).num_days().max(1)).unwrap_or(30)
    }
}

pub(crate) struct ExportContext<'a> {
    pub pool: &'a PgPool,
    pub user: &'a UserContext,
    pub window: Option<ExportWindow>,
    pub limit: i64,
    uri: Uri,
}

impl<'a> ExportContext<'a> {
    pub(crate) const fn new(pool: &'a PgPool, user: &'a UserContext, uri: Uri) -> Self {
        Self {
            pool,
            user,
            window: None,
            limit: HARD_CAP,
            uri,
        }
    }

    pub(crate) fn query<T: DeserializeOwned>(&self) -> AdminResult<T> {
        axum::extract::Query::<T>::try_from_uri(&self.uri)
            .map(|q| q.0)
            .map_err(|e| AdminError::BadRequest(e.body_text()))
    }

    pub(crate) fn window(&self) -> AdminResult<ExportWindow> {
        self.window
            .ok_or_else(|| AdminError::BadRequest("This export needs a window".to_owned()))
    }
}

// Why: Rust native async trait methods are not dyn-compatible, and the
// registry holds datasets as `&dyn DataSet`.
#[async_trait]
pub(crate) trait DataSet: Send + Sync {
    fn id(&self) -> &'static str;
    fn title(&self) -> &'static str;
    fn description(&self) -> &'static str {
        ""
    }
    fn columns(&self) -> &'static [Column];
    fn window(&self) -> Window;
    fn cap(&self) -> i64 {
        HARD_CAP
    }
    fn allows(&self, user: &UserContext) -> bool {
        user.is_console
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table>;
}
