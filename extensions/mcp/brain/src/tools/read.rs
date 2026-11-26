//! `search`, `answer` and `get`: finding and reading.

use super::fragments::{article_id, integer, lang, path, schema, text, where_filter};
use super::{Json, ToolKind, ToolSpec};

pub const SPECS: &[ToolSpec] = &[
    ToolSpec {
        name: "search",
        title: "Search the Brain",
        kind: ToolKind::Read,
        description: SEARCH,
        schema: search_schema,
    },
    ToolSpec {
        name: "answer",
        title: "Answer from the Brain",
        kind: ToolKind::Read,
        description: ANSWER,
        schema: answer_schema,
    },
    ToolSpec {
        name: "get",
        title: "Get an article",
        kind: ToolKind::Read,
        description: GET,
        schema: get_schema,
    },
];

const SEARCH: &str = "Hybrid semantic + keyword search across the whole documentation corpus \
    the caller may read. Ask a full natural-language question (not just keywords); results are \
    ranked by an embeddings model and a cross-encoder reranker, so conceptually related \
    passages match even when they share no exact words. Search is cross-lingual: a query in \
    one language finds relevant articles in ANY language - each result is tagged with its \
    `lang` so you can read it directly, and titled in the language of the passage it returns, \
    so a hit and its snippet never disagree. Built for very large corpora: returns short \
    passages, not whole articles. Example result: `{results: [{article_id, path, title, \
    snippet, score, lang, space}], total, mode, confidence, top_score}`. To read a full \
    article afterwards, call `get` with the returned `article_id` or `path`. To restrict to \
    one language pass `lang`; to restrict to one space pass `space` (discover values with \
    `list_spaces`). Each result carries a `score` in [0,1]: how well a cross-encoder judged \
    that passage against your question, independently of the other results - so the LEVEL is \
    meaningful and the gap between results is not. The response carries a `confidence` for \
    the query as a whole. THIS TOOL ALWAYS RETURNS ITS BEST MATCHES, EVEN WHEN THE \
    DOCUMENTATION DOES NOT CONTAIN THE ANSWER: `confidence: \"low\"` means exactly that - read \
    the results as related material, and say the answer is not documented rather than \
    assembling one out of them. `confidence: \"unrated\"` means the reranking model was \
    unavailable for this call; the results are the same articles, only worse ordered, and no \
    confidence can be given. `mode: \"keyword\"` means the semantic arm was unavailable and \
    the results are keyword matches.";

const ANSWER: &str = "Retrieve the most relevant documentation for a natural-language \
    question and hand back the CONTEXT an answer is written from, with its sources. This \
    tool never writes the answer: YOU write it from `context`, citing `[n]` against \
    `citations` (each block of the context is numbered the same way), and say the answer is \
    not documented when `confidence` is \"low\". Use it when the user wants a direct answer \
    rather than a list of results. Retrieval is cross-lingual (sources may be in any \
    language). Example: `{context, citations: [{path, title, lang, snippet}], mode: \
    \"context\", confidence, top_score}`. `confidence` means the same here as in `search`: \
    \"low\" says retrieval was weak and the answer is probably not documented, so treat \
    anything built on it with suspicion; \"unrated\" says the reranking model was \
    unavailable and no confidence can be given.";

const GET: &str = "Return the full Markdown of a single article, by `path` or `article_id` \
    (exactly one). Small articles are returned inline in `markdown`; when a large one comes \
    back with a short-lived `markdown_url` instead of the text, fetch that URL directly \
    (out of band, served with your own permissions) and read it selectively, so big \
    articles never burn your context. An article may exist in several languages: pass \
    `lang` to read a specific one - omit it and you get the article's default variant. The \
    response always reports the variant it actually served in `variant`, which falls back \
    to the default one when the article does not carry the one you asked for; `title` and \
    `description` are that variant's too. An EMPTY `description` on a translation is not a \
    bug: it means that translation has none of its own yet, which is what search would \
    index, and it is set with `update {lang, front_matter: {description}}`. `markdown` is \
    the article's own text, exactly as stored, so it is safe to edit and send back with \
    `update`; the articles filed under it are listed separately in `children`, never \
    inside the text. Images and files the text names as `asset://<id>` come with a \
    short-lived signed URL each in `asset_urls`. Example: `{found, article_id, space, path, \
    title, lang, variant: {lang, is_default}, markdown, front_matter, metadata, revision, \
    can_write, children: [{article_id, title, description}], asset_urls}`. Typically called \
    after `search` / `answer` on a result they returned.";

fn search_schema() -> Json {
    schema(
        &[
            (
                "query",
                text("Full natural-language question (not just keywords)."),
            ),
            ("limit", integer("Max results (<=50, default 10).")),
            (
                "lang",
                text(
                    "Optional ISO language code to restrict the search to one language \
                     (default: all languages).",
                ),
            ),
            (
                "space",
                integer(
                    "Optional space id to restrict the search (see list_spaces). Leave it \
                     off entirely to search everything.",
                ),
            ),
            ("where", where_filter()),
        ],
        &["query"],
        &[],
    )
}

fn answer_schema() -> Json {
    schema(
        &[
            ("question", text("Natural-language question to answer.")),
            (
                "lang",
                text("Optional ISO language code to restrict retrieval."),
            ),
            (
                "space",
                integer("Optional space id to restrict retrieval (see list_spaces)."),
            ),
            (
                "max_sources",
                integer("Max source passages to retrieve (<=20, default 8)."),
            ),
        ],
        &["question"],
        &[],
    )
}

fn get_schema() -> Json {
    schema(
        &[
            ("path", path()),
            ("article_id", article_id()),
            ("lang", lang()),
        ],
        &[],
        &[&["path"], &["article_id"]],
    )
}
