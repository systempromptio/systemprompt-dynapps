//! `/admin/brain/articles/{id}`: one article with the tree beside it, in a
//! language, with its tabs: Content, Chunks, Ingest, Files, Access, Usage and
//! History.

use axum::extract::{Path, Query};
use axum::response::Response;
use axum::Extension;
use chrono::{TimeDelta, Utc};
use dynbrain::ids::ArticleId;
use serde::{Deserialize, Serialize};
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use crate::access::{self, AccessView};
use crate::chunks::{self, ChunksView, IngestView};
use crate::read::{self, Document, FileItem, Reader, Space};
use crate::source::{Source, require_admin};
use crate::tree;
use crate::views::{AT_SOURCE, NOT_MOUNTED, crumbs, keep, when};

const TABS: [(&str, &str); 7] = [
    ("content", "Content"),
    ("chunks", "Chunks"),
    ("ingest", "Ingest"),
    ("files", "Files"),
    ("access", "Access"),
    ("usage", "Usage"),
    ("history", "History"),
];

#[derive(Debug, Default, Deserialize)]
pub struct ArticleQuery {
    #[serde(default)]
    pub lang: Option<String>,
    #[serde(default)]
    pub tab: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Choice {
    pub label: String,
    pub href: String,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct FileRow {
    pub name: String,
    pub kind: String,
    pub size: String,
    pub lang: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct UsageView {
    pub retrieved: i64,
    pub cited: i64,
}

#[derive(Debug, Serialize)]
pub struct RevisionRow {
    pub number: i32,
    pub title: String,
    pub author: String,
    pub when: String,
}

#[derive(Debug, Default, Serialize)]
pub struct ArticlePage {
    pub page: &'static str,
    pub title: String,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub unavailable: Option<&'static str>,
    pub tree: String,
    pub doc: Option<Document>,
    pub missing_lang: bool,
    pub languages: Vec<Choice>,
    pub tabs: Vec<Choice>,
    pub notice: Option<&'static str>,
    pub chunks: Option<ChunksView>,
    pub ingest: Option<IngestView>,
    pub show_content: bool,
    pub show_files: bool,
    pub files: Vec<FileRow>,
    pub access: Option<AccessView>,
    pub usage: Option<UsageView>,
    pub history: Option<Vec<RevisionRow>>,
}

pub async fn article_page(
    shell: Page,
    Extension(source): Extension<Source>,
    Path(id): Path<i64>,
    Query(query): Query<ArticleQuery>,
) -> AdminHtmlResult<Response> {
    require_admin(&shell)?;
    let mut faults = Faults::new();
    let mut page = ArticlePage {
        page: "brain-spaces",
        ..ArticlePage::default()
    };
    let Some(console) = source.console() else {
        page.unavailable = Some(NOT_MOUNTED);
        page.breadcrumbs = crumbs("Spaces");
        return Ok(render(&shell, &page, &faults));
    };
    let admin = &shell.user.user_id;
    let spaces = faults.take("Spaces", read::read_spaces(&console, admin).await);
    let wanted = query.lang.as_deref().filter(|lang| !lang.is_empty());
    let doc = match read::read_document(&console, admin, id, wanted).await {
        Ok(doc) if doc.found => doc,
        _ => {
            page.missing_lang = wanted.is_some();
            read::read_document(&console, admin, id, None)
                .await
                .ok()
                .filter(|doc| doc.found)
                .ok_or_else(|| AdminError::NotFound(format!("article {id}")))?
        },
    };
    let tab = TABS
        .iter()
        .find(|(key, _)| Some(*key) == query.tab.as_deref())
        .map_or("content", |(key, _)| *key);
    page.tree = tree::render(&tree::build(&spaces, Some(id)));
    page.title.clone_from(&doc.title);
    page.breadcrumbs = crumbs(&doc.title);
    page.languages = languages(&spaces, &doc, id, tab);
    page.tabs = TABS
        .iter()
        .map(|(key, label)| Choice {
            label: (*label).to_owned(),
            href: format!("/admin/brain/articles/{id}?lang={}&tab={key}", doc.lang),
            active: *key == tab,
        })
        .collect();
    let reader = Reader {
        console: &console,
        admin,
    };
    page.show_content = tab == "content";
    tab_body(&mut page, &reader, &doc, tab, &mut faults).await;
    page.doc = Some(doc);
    Ok(render(&shell, &page, &faults))
}

fn render(shell: &Page, page: &ArticlePage, faults: &Faults) -> Response {
    render_page_with_faults(
        &shell.engine,
        "brain-article",
        page,
        Viewer::new(&shell.user, &shell.marketplace),
        faults,
    )
}

fn languages(spaces: &[Space], doc: &Document, id: i64, tab: &str) -> Vec<Choice> {
    spaces
        .iter()
        .find(|space| space.item.space == doc.space)
        .map(|space| space.item.langs.as_slice())
        .unwrap_or_default()
        .iter()
        .map(|lang| Choice {
            label: lang.clone(),
            href: format!("/admin/brain/articles/{id}?lang={lang}&tab={tab}"),
            active: *lang == doc.lang,
        })
        .collect()
}

async fn tab_body(
    page: &mut ArticlePage,
    reader: &Reader<'_>,
    doc: &Document,
    tab: &str,
    faults: &mut Faults,
) {
    let (console, admin) = (reader.console, reader.admin);
    let id = doc.article_id;
    match (tab, &console.native) {
        ("files", _) => {
            let files = faults.take("Files", read::read_files(console, admin, id).await);
            page.show_files = true;
            page.files = files.iter().map(file_row).collect();
        },
        ("access", _) => page.access = Some(access::view(console, admin, id, &doc.lang).await),
        ("chunks", Some(native)) => page.chunks = chunks::chunks_view(native, id, doc, faults).await,
        ("ingest", Some(native)) => {
            page.ingest = chunks::ingest_view(native, id, &doc.lang, faults).await;
        },
        ("usage", Some(native)) => {
            let since = Utc::now() - TimeDelta::days(30);
            let usage = native.repo.count_article_usage(ArticleId(id), since).await;
            page.usage = keep(faults, "Usage", usage).map(|usage| UsageView {
                retrieved: usage.retrieved,
                cited: usage.cited,
            });
        },
        ("history", Some(native)) => {
            let revisions = native.repo.list_revisions(ArticleId(id), &doc.lang).await;
            page.history = keep(faults, "History", revisions).map(|revisions| {
                revisions
                    .into_iter()
                    .rev()
                    .map(|revision| RevisionRow {
                        number: revision.number,
                        title: revision.title,
                        author: revision
                            .author
                            .map(|author| author.as_str().to_owned())
                            .unwrap_or_default(),
                        when: when(revision.created_at),
                    })
                    .collect()
            });
        },
        ("chunks" | "ingest" | "usage" | "history", None) => page.notice = Some(AT_SOURCE),
        _ => {},
    }
}

fn file_row(file: &FileItem) -> FileRow {
    FileRow {
        name: file.name.clone(),
        kind: file.mimetype.clone(),
        size: format!("{} KB", file.size_bytes.div_ceil(1024)),
        lang: file.lang.clone().unwrap_or_else(|| "all".to_owned()),
    }
}
