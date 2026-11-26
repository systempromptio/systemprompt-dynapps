//! `request_upload`, `asset_upload`, `asset_list` and `asset_link`: the files
//! of articles. The bytes never travel through a tool call.

use super::fragments::{article_id, integer, lang, path, schema, text};
use super::{Json, ToolKind, ToolSpec};

pub(super) const SPECS: &[ToolSpec] = &[
    ToolSpec {
        name: "request_upload",
        title: "Request a file upload URL",
        kind: ToolKind::Write,
        description: REQUEST_UPLOAD,
        schema: request_upload_schema,
    },
    ToolSpec {
        name: "asset_list",
        title: "List the files of an article",
        kind: ToolKind::Read,
        description: ASSET_LIST,
        schema: asset_list_schema,
    },
    ToolSpec {
        name: "asset_link",
        title: "Get a file download URL",
        kind: ToolKind::Read,
        description: ASSET_LINK,
        schema: asset_link_schema,
    },
    ToolSpec {
        name: "asset_upload",
        title: "Add a file to an article",
        kind: ToolKind::Write,
        description: ASSET_UPLOAD,
        schema: asset_upload_schema,
    },
];

const REQUEST_UPLOAD: &str = "Issue a one-time, short-lived upload URL so a FILE is sent \
    straight to the server out of band, without its bytes passing through this channel \
    (saving your context and tokens). Name the article the file will belong to with \
    `article_id` or `path`, and the file with `filename`. Flow: call this, then send the \
    file to the returned `upload_url` as the RAW request body - for example `curl \
    --data-binary @file \"<upload_url>\"` - never as a multipart form (`-F` is refused): \
    the response carries the `upload_id`. Then call `asset_upload` with the same \
    article, the filename and that `upload_id` to attach the file. The URL is valid for `expires_minutes` \
    and takes one upload of at most `max_bytes`. Markdown goes inline in `markdown`; this \
    is for files. Example: `{upload_id, upload_url, method, max_bytes, expires_minutes, \
    article_id, filename, instructions}`.";

const ASSET_LIST: &str = "List the FILES of one article - images, diagrams, PDFs, \
    spreadsheets - by `article_id` or `path` (exactly one). An article is prose plus the \
    files the prose is about, and this is how you see them: every file comes back with its \
    `attachment_id` (pass it to `asset_link` to download it), `name`, `mimetype`, \
    `size_bytes`, its `ref` (the `asset://<id>` the article's Markdown names it by) and the \
    `markdown` that embeds it. Example: `{article_id, files: [{attachment_id, name, \
    mimetype, size_bytes, lang, ref, markdown}], total, shown, offset}`.";

const ASSET_LINK: &str = "Get a short-lived download URL for ONE file of an article (find \
    its `attachment_id` with `asset_list`). The bytes are served out of band with your own \
    permissions - fetch the URL directly - so a 20 MB PDF never passes through this \
    channel. Only files of articles you may read are served. Example: `{found, \
    attachment_id, article_id, name, mimetype, size_bytes, download_url, expires_at, \
    expires_minutes}`.";

const ASSET_UPLOAD: &str = "Add a FILE to an article - an image, a diagram, a PDF, a \
    spreadsheet - in two calls. First call it with `article_id` (or `path`) and `filename`: \
    you get back an `upload_url` and an `upload_id`, as from `request_upload`; send the \
    file to the URL as the RAW request body (`curl --data-binary @file \"<upload_url>\"`, \
    never as a multipart form). Then call this again with the same article, the same \
    `filename` and that `upload_id`, and the file is attached to the article. Executables are refused; the file \
    is stored exactly as uploaded (never re-encoded or resized). For it to be VISIBLE in \
    the article, reference it from the Markdown with the `ref` the answer gives (`![](asset://<id>)` \
    for an image) via `update`. Example (first call): `{upload_id, upload_url, method, \
    max_bytes, expires_minutes, article_id, filename, instructions}`; (second call): \
    `{attached: true, article_id, attachment_id, name, mimetype, size_bytes, ref, \
    markdown}`.";

fn request_upload_schema() -> Json {
    schema(
        &[
            ("article_id", article_id()),
            ("path", path()),
            (
                "filename",
                text("Real name of the file, with its extension (e.g. 'network-diagram.svg')."),
            ),
        ],
        &[],
        &[&["article_id"], &["path"]],
    )
}

fn asset_list_schema() -> Json {
    schema(
        &[
            ("article_id", article_id()),
            ("path", path()),
            ("lang", lang()),
            ("limit", integer("Max files (<=500, default 100).")),
            ("offset", integer("Files to skip (paging).")),
        ],
        &[],
        &[&["article_id"], &["path"]],
    )
}

fn asset_link_schema() -> Json {
    schema(
        &[
            (
                "attachment_id",
                integer("Id of the file, from asset_list."),
            ),
            (
                "article_id",
                integer("Optional: the article the file belongs to."),
            ),
        ],
        &["attachment_id"],
        &[],
    )
}

fn asset_upload_schema() -> Json {
    schema(
        &[
            ("article_id", article_id()),
            ("path", path()),
            (
                "filename",
                text(
                    "Real name of the file, with its extension (e.g. 'network-diagram.svg'). \
                     Send it on both calls.",
                ),
            ),
            (
                "upload_id",
                text("SECOND call only: the `upload_id` the first call returned."),
            ),
            ("lang", lang()),
        ],
        &[],
        &[&["article_id"], &["path"]],
    )
}
