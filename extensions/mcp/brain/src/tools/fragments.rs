//! The pieces of input schema the tools share, so that a field is described
//! once and the same way wherever it appears.

use serde_json::json;

use super::Json;

pub const MAX_NODES: usize = 200;

pub fn schema(properties: &[(&str, Json)], required: &[&str], any_of: &[&[&str]]) -> Json {
    let mut schema = json!({
        "type": "object",
        "properties": properties
            .iter()
            .map(|(name, property)| ((*name).to_owned(), property.clone()))
            .collect::<serde_json::Map<String, Json>>(),
    });
    if !required.is_empty() {
        schema["required"] = json!(required);
    }
    if !any_of.is_empty() {
        schema["anyOf"] = any_of
            .iter()
            .map(|names| json!({"required": names}))
            .collect();
    }
    schema
}

pub fn text(description: &str) -> Json {
    json!({"type": "string", "description": description})
}

pub fn integer(description: &str) -> Json {
    json!({"type": "integer", "description": description})
}

pub fn lang() -> Json {
    text(
        "Optional ISO language code ('es' or 'en') selecting the language variant of the \
         article. Omit it to use the article's default variant.",
    )
}

pub fn article_id() -> Json {
    integer("Id of the article.")
}

pub fn path() -> Json {
    text("Path of the article, as the other tools return it, e.g. handbook/holidays.")
}

pub fn position() -> Json {
    json!({
        "type": "string",
        "enum": ["first", "last"],
        "description": "Place among the siblings (default 'last').",
    })
}

pub fn where_filter() -> Json {
    json!({
        "type": "object",
        "description": "Optional metadata filter, as a mapping of declared field to value, \
            e.g. {\"status\": \"approved\"} or {\"owner\": false} for articles nobody signs. \
            The fields are the ones the spaces declare (see metadata_schema); an unknown \
            one is refused and the answer lists what is available.",
    })
}

pub fn object(description: &str) -> Json {
    json!({"type": "object", "description": description})
}

pub const FRONT_MATTER_NEW: &str = "Header keys to set on the new article, as a mapping, e.g. \
    {\"description\": \"...\", \"icon\": \"x\"}. The same keys a pasted `.md` header carries, \
    merged the same way and reported back in `front_matter`; a key given here wins over the \
    same key in a pasted header. A key the space DECLARES as a typed field is written as \
    typed metadata, under the same rules as `metadata`.";

pub const METADATA_NEW: &str = "Typed metadata to set, as a mapping of declared field to \
    value, e.g. {\"status\": \"draft\", \"owner\": \"j.smith\"}. Call metadata_schema first: \
    it names the fields the space declares and gives an `expects` for each, which is what to \
    write - a selection takes one KEY from `options`, tags a list of keys, dates \
    'YYYY-MM-DD'. A value the space cannot take is refused with the reason. Naming a field \
    the space does not declare is refused outright: nothing is created.";
