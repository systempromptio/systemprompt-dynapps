//! `create`, `create_tree`, `update`, `move` and `trash`: authoring.

use serde_json::json;

use super::fragments::{
    FRONT_MATTER_NEW, MAX_NODES, METADATA_NEW, article_id, integer, lang, object, position,
    schema, text,
};
use super::{Json, ToolKind, ToolSpec};

pub(super) const SPECS: &[ToolSpec] = &[
    ToolSpec {
        name: "create",
        title: "Create an article",
        kind: ToolKind::Write,
        description: CREATE,
        schema: create_schema,
    },
    ToolSpec {
        name: "create_tree",
        title: "Create a tree of articles",
        kind: ToolKind::Write,
        description: CREATE_TREE,
        schema: create_tree_schema,
    },
    ToolSpec {
        name: "update",
        title: "Update an article",
        kind: ToolKind::IdempotentWrite,
        description: UPDATE,
        schema: update_schema,
    },
    ToolSpec {
        name: "move",
        title: "Move an article",
        kind: ToolKind::IdempotentWrite,
        description: MOVE,
        schema: move_schema,
    },
    ToolSpec {
        name: "trash",
        title: "Trash an article",
        kind: ToolKind::Destructive,
        description: TRASH,
        schema: trash_schema,
    },
];

const CREATE: &str = "Create a NEW article under an existing parent. Precondition: you must \
    be able to write the parent (or, for a top-level article, the space - pass `space` \
    instead of `parent_id`); check with `metadata` (`can_write`) if unsure. Use this when \
    the user asks you to author or draft documentation. Pass the body as Markdown in \
    `markdown`; it is stored verbatim as the canonical source. `position` places it first \
    or last among siblings (default last). Pass `lang` to author the article in an explicit \
    language: its variant is created, so the article is readable and editable in that \
    language afterwards; omit it and the article gets a single, default variant. Content \
    may be pasted as a whole `.md` file: a YAML front matter header is split off and kept \
    as the article's header (reported in `front_matter`), never stored as part of the body. \
    `front_matter` and `metadata` set the header and typed fields directly, without pasting \
    a YAML block: an explicit key wins over the same key in a pasted header. A second \
    LANGUAGE is NOT a second article: create the article once, then add the other language \
    with `update {article_id, lang}` on the SAME `article_id`. A new article is indexed for \
    search shortly after it is written. Example: `{article_id, space, parent_id, title, \
    path, variant: {lang, is_default}}`. To build a whole section at once use \
    `create_tree`.";

const CREATE_TREE: &str = "Create a whole subtree (a section with nested articles) in ONE \
    call and get back the created ids. Precondition: you must be able to write the parent \
    (or the space, for top-level articles). Use this when the user wants a structured set \
    of articles authored from Markdown at once (e.g. a guide with chapters). `nodes` is a \
    nested list of `{title, markdown?, front_matter?, metadata?, children?}`; each node \
    becomes an article and its `children` become nested articles. `front_matter`/`metadata` \
    apply PER NODE, exactly as in a single `create`. Every node is checked before the first \
    one is created, so a bad node leaves nothing behind. Capped at 200 nodes per call - \
    split larger trees. `lang` applies to the WHOLE subtree: one call authors one language. \
    Returns the mirror tree with ids so you can reference the created articles afterwards. \
    Example: `{parent_id, nodes: [{title, article_id, path, children: [...]}], created}`.";

const UPDATE: &str = "Update an article's `title` and/or content. Precondition: you must be \
    able to write the article; verify with `metadata` if unsure. Pass new content as \
    Markdown in `markdown`; it replaces the stored source and appends a revision. Provide \
    at least one of `title`, `markdown`, `front_matter` or `metadata`. Pass `lang` to edit \
    ONE language of the article: the content is written to that variant only - every other \
    language is left untouched - and the variant is created if the article did not carry it \
    yet. A `title` sent WITH `lang` is that translation's title and the article keeps its \
    own; sent without one it renames the article. **This is how an article gets a second \
    language** - the same `article_id` with a different `lang`, never a second `create`, \
    which would produce two unrelated articles. Content may be pasted as a whole `.md` \
    file: a YAML front matter header is split off and MERGED into the article's own \
    header, never stored as part of the body. To change header keys ALONE, pass \
    `front_matter` instead of resending the body: it is PER LANGUAGE when you pass `lang`, \
    so a translation carries its own `description` - which is what search reads, so a \
    translation without one is a page nobody finds in that language. Typed fields belong to \
    the ARTICLE, not to a translation. Example: `{article_id, title, updated, revision, \
    variant: {lang, is_default}}`.";

const MOVE: &str = "Re-parent an article under a different parent - this is how a space \
    gets REORGANISED. Precondition: you must be able to write both the article and the new \
    parent. Cannot move an article under itself or its own descendant, nor into another \
    space. `position` optionally places it first or last among the new siblings. The access \
    the article inherited from its old place is replaced by the new place's. Example: \
    `{article_id, new_parent_id, moved}`.";

const TRASH: &str = "Send an article to the trash. This is a SOFT, reversible delete: \
    there is no hard delete through this server. Precondition: you must be able to write \
    the article. Trashing an article also trashes its descendants, and takes them out of \
    search. Example: `{article_id, trashed}`.";

fn create_schema() -> Json {
    schema(
        &[
            (
                "parent_id",
                integer("Id of the parent article to create under."),
            ),
            (
                "space",
                integer("Space id, to create a top-level article instead of using parent_id."),
            ),
            ("title", text("Title of the new article.")),
            ("markdown", text("Inline Markdown body.")),
            ("position", position()),
            ("front_matter", object(FRONT_MATTER_NEW)),
            ("metadata", object(METADATA_NEW)),
            ("lang", lang()),
        ],
        &["title"],
        &[&["parent_id"], &["space"]],
    )
}

fn create_tree_schema() -> Json {
    let node = json!({
        "type": "object",
        "required": ["title"],
        "properties": {
            "title": {"type": "string"},
            "markdown": {"type": "string", "description": "Inline Markdown body."},
            "front_matter": object(FRONT_MATTER_NEW),
            "metadata": object(METADATA_NEW),
            "children": {"type": "array", "description": "Nested child nodes (same shape)."},
        },
    });
    schema(
        &[
            (
                "parent_id",
                integer("Id of the parent article to build under."),
            ),
            (
                "space",
                integer("Space id, to build at the top level instead of using parent_id."),
            ),
            (
                "nodes",
                json!({
                    "type": "array",
                    "description": format!("Nested nodes to create (at most {MAX_NODES} in all)."),
                    "items": node,
                }),
            ),
            ("lang", lang()),
        ],
        &["nodes"],
        &[&["parent_id"], &["space"]],
    )
}

fn update_schema() -> Json {
    schema(
        &[
            ("article_id", integer("Id of the article to update.")),
            (
                "title",
                text(
                    "New title (optional). With `lang` it is the title OF THAT translation and \
                     the article keeps its own; without it, the article is renamed.",
                ),
            ),
            (
                "markdown",
                text(
                    "New inline Markdown body (optional). A YAML front matter header is \
                     accepted and kept as the article's header.",
                ),
            ),
            (
                "front_matter",
                object(
                    "Header keys to set, as a mapping, WITHOUT resending the body, e.g. \
                     {\"description\": \"...\", \"icon\": \"x\"}. Merged into the article's \
                     header and reported in `front_matter`. PER LANGUAGE when you pass `lang`.",
                ),
            ),
            (
                "metadata",
                object(
                    "Typed metadata to set, as a mapping of declared field to value, e.g. \
                     {\"status\": \"approved\"}. Call metadata_schema first: it gives an \
                     `expects` for each field, which is what to write. `false` clears a field. \
                     A value the space cannot take is refused with the reason.",
                ),
            ),
            ("lang", lang()),
        ],
        &["article_id"],
        &[],
    )
}

fn move_schema() -> Json {
    schema(
        &[
            ("article_id", article_id()),
            (
                "new_parent_id",
                integer("Id of the destination parent article."),
            ),
            ("position", position()),
        ],
        &["article_id", "new_parent_id"],
        &[],
    )
}

fn trash_schema() -> Json {
    schema(
        &[("article_id", integer("Id of the article to send to the trash."))],
        &["article_id"],
        &[],
    )
}
