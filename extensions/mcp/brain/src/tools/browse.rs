//! `list_spaces`, `list`, `outline`, `export`, `metadata` and
//! `metadata_schema`: discovering and navigating.

use serde_json::json;

use super::fragments::{article_id, integer, lang, path, schema, text, where_filter};
use super::{Json, ToolKind, ToolSpec};

pub const SPECS: &[ToolSpec] = &[
    ToolSpec {
        name: "list_spaces",
        title: "List spaces",
        kind: ToolKind::Read,
        description: LIST_SPACES,
        schema: list_spaces_schema,
    },
    ToolSpec {
        name: "list",
        title: "List the articles of a space",
        kind: ToolKind::Read,
        description: LIST,
        schema: list_schema,
    },
    ToolSpec {
        name: "outline",
        title: "Outline a tree of articles",
        kind: ToolKind::Read,
        description: OUTLINE,
        schema: outline_schema,
    },
    ToolSpec {
        name: "export",
        title: "Export a space",
        kind: ToolKind::Read,
        description: EXPORT,
        schema: export_schema,
    },
    ToolSpec {
        name: "metadata",
        title: "Get article metadata",
        kind: ToolKind::Read,
        description: METADATA,
        schema: metadata_schema_of_metadata,
    },
    ToolSpec {
        name: "metadata_schema",
        title: "Describe metadata fields",
        kind: ToolKind::Read,
        description: METADATA_SCHEMA,
        schema: metadata_schema_schema,
    },
];

const LIST_SPACES: &str = "List the available documentation spaces. Use this to orient \
    yourself and to discover the valid `space` / `lang` values the other tools accept. \
    Filterable and paginated for large catalogs: pass `query` to match space names, `lang` to \
    filter by language; results are capped and the response reports `total` so you can \
    refine instead of enumerating everything. Each space reports your `level` on it. \
    Example: `{spaces: [{space, slug, name, lang, langs, article_count, level}], total, \
    shown, offset}`. For 'find documentation about X' prefer `search` - do not page through \
    the whole catalog. To explore the internal structure of one space, use `outline`.";

const LIST: &str = "List the articles inside ONE space. Requires `space` (this is not a \
    global dump). Paginated. Use it to browse the structure of a specific space once you \
    know which one; to search across everything use `search`. This is a FLAT list of paths - \
    for the hierarchy (parent/child tree) plus subtree sizes, use `outline`. Pass `lang` to \
    list it in one language: every article is titled by its own translation, and the ones \
    that do not carry it keep their own title. Example: `{documents: [{article_id, path, \
    title, lang, parent_id}], total, shown, offset}`.";

const OUTLINE: &str = "Return the STRUCTURE (skeleton) of the article tree - titles and \
    hierarchy only, never article bodies - starting from an anchor you provide (a `space`, \
    or any `node` article id). Built to stay cheap on gigantic, deeply-nested trees: you \
    never dump a whole tree, you traverse it deliberately, and every response is bounded. \
    Two modes: `descend` (default) expands DOWNWARD from the anchor up to `depth` levels, \
    capped at `max_nodes`; each node reports whether it has children and how many \
    (`has_children`, `child_count`) so you can drill in selectively, and if the subtree is \
    larger than the cap the response sets `truncated`/`has_more` with a `hint` to narrow to a \
    child (`node=<id>`) or page with `offset`. `ascend` walks UPWARD from a `node` up to \
    `max_levels`, returning each ancestor WITH the size of its subtree (a cheap count) and a \
    `recommended_ceiling` = the highest ancestor whose subtree still fits under `max_nodes`; \
    use it when you have one article and must find the right 'root' to work from WITHOUT \
    accidentally pulling an enormous aggregator - the level where the subtree size jumps is \
    the natural boundary. Pass `lang` to read the skeleton in one language: each node is \
    titled by its own translation, and a node without one keeps its title. Example \
    (descend): `{anchor: {id, name}, nodes: [{id, title, lang, depth, has_children, \
    child_count}], total, shown, truncated, has_more, offset, hint}`. This is THE tool to \
    understand or navigate the shape of a large corpus. To read an article's content use \
    `get`; to download a whole space as one file use `export`; to find articles by meaning \
    use `search`; to list spaces use `list_spaces`.";

const EXPORT: &str = "Export an ENTIRE space as a single Markdown document (index-only or \
    full text), so a large space is ingested in one call. Use `format='index'` for a table \
    of contents (llms.txt style) or `format='full'` for the whole concatenated Markdown. \
    Example: `{space, format, mimetype, markdown, size_bytes, article_count, asset_urls}`. \
    Only the articles you may read are included. For an interactive, bounded skeleton \
    (rather than the whole text), use `outline` instead.";

const METADATA: &str = "Return internal metadata for one or more articles - the fields you \
    cannot infer from search results: who authored and last edited it and when, where it \
    lives (space, path, parent, language variants), its revision, whether it inherits its \
    access, and - importantly before any write - whether YOU can edit it (`can_write`). \
    Accepts a single `article_id`/`path` or a batch of `article_ids` (capped by `limit`). \
    Example per item: `{id, title, path, lang, space, can_write, inherit_access, author, \
    created, last_editor, updated, parent_id, child_count, variant_langs, revision, \
    front_matter, metadata, description}`. Pass `lang` to read the metadata of THAT \
    variant: `title`, `description`, `front_matter` and `lang` are that variant's. \
    `metadata` (the TYPED fields) stays the ARTICLE's in every language: it is one record, \
    not one per translation.";

const METADATA_SCHEMA: &str = "Describe the TYPED metadata fields of a space - what exists, \
    of what type, and EXACTLY WHAT TO WRITE into each one. Call this BEFORE setting \
    metadata with `create`, `create_tree` or `update`: the fields are declared per space, so \
    they cannot be guessed from an article you have read. Answer per field: `{name, type, \
    label, options, expects}`. **`expects` is the instruction** - every field has one, \
    because a type name does not say how to write it:\n\
    - `selection`: exactly one key from `options` - the KEY, not the label an interface \
    shows.\n\
    - `tags`: a LIST of keys from `options`.\n\
    - `date` / `datetime`: `YYYY-MM-DD` / `YYYY-MM-DD HH:MM:SS`.\n\
    - `boolean`: true or false. `integer`/`float`: a number. `char`/`text`: text. `link`: an \
    absolute http(s) URL.\n\
    Anchor it on an article (`article_id`/`path`) or on a space (`space`), before there is \
    an article to ask about. A filter (`where` in `search` and `list`) matches the same \
    values you write.";

fn list_spaces_schema() -> Json {
    schema(
        &[
            ("query", text("Optional filter matching space names.")),
            ("lang", text("Optional ISO language code filter.")),
            ("limit", integer("Max rows (<=200, default 50).")),
            ("offset", integer("Pagination offset (default 0).")),
        ],
        &[],
        &[],
    )
}

fn list_schema() -> Json {
    schema(
        &[
            (
                "space",
                integer("Space id (see list_spaces)."),
            ),
            ("lang", lang()),
            ("limit", integer("Max rows (<=500, default 200).")),
            ("offset", integer("Pagination offset (default 0).")),
            ("where", where_filter()),
        ],
        &["space"],
        &[],
    )
}

fn outline_schema() -> Json {
    schema(
        &[
            (
                "node",
                integer(
                    "Anchor: an article id (any node in the tree). Required for mode 'ascend'.",
                ),
            ),
            (
                "space",
                integer("Anchor: a space id (see list_spaces); descend from its root."),
            ),
            (
                "mode",
                json!({
                    "type": "string",
                    "enum": ["descend", "ascend"],
                    "description": "'descend' (default) expands downward; 'ascend' walks up \
                        from a `node` to find the boundary.",
                }),
            ),
            (
                "depth",
                integer(
                    "descend only: levels to expand (default 2; -1 = the whole subtree, still \
                     capped by max_nodes).",
                ),
            ),
            (
                "max_nodes",
                integer(
                    "Hard cap on returned nodes (default 200); triggers truncated/has_more/hint.",
                ),
            ),
            (
                "max_levels",
                integer("ascend only: max ancestor levels to walk up (default 6)."),
            ),
            ("lang", lang()),
            (
                "offset",
                integer("descend only: paginate the anchor's direct children (default 0)."),
            ),
        ],
        &[],
        &[&["node"], &["space"]],
    )
}

fn export_schema() -> Json {
    schema(
        &[
            ("space", integer("Space id (see list_spaces).")),
            (
                "format",
                json!({
                    "type": "string",
                    "enum": ["full", "index"],
                    "description": "'full' concatenated Markdown or 'index' table of \
                        contents (default 'full').",
                }),
            ),
            ("lang", lang()),
        ],
        &["space"],
        &[],
    )
}

fn metadata_schema_of_metadata() -> Json {
    schema(
        &[
            ("article_id", article_id()),
            ("path", path()),
            (
                "article_ids",
                json!({
                    "type": "array",
                    "items": {"type": "integer"},
                    "description": "A batch of article ids (capped by limit).",
                }),
            ),
            (
                "limit",
                integer("Max rows for the batch (<=200, default 50)."),
            ),
            ("lang", lang()),
        ],
        &[],
        &[&["article_id"], &["path"], &["article_ids"]],
    )
}

fn metadata_schema_schema() -> Json {
    schema(
        &[
            (
                "article_id",
                integer("Article whose applicable fields you want."),
            ),
            ("path", path()),
            (
                "space",
                integer("Space id (see list_spaces), for everything it declares."),
            ),
        ],
        &[],
        &[&["article_id"], &["path"], &["space"]],
    )
}
