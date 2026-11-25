//! Typed properties: the definitions a space may declare and the values a
//! variant may carry.

use dynbrain::properties::{PropertyError, PropertySchema, PropertyType};
use serde_json::{Map, Value, json};

fn schema() -> PropertySchema {
    PropertySchema::parse(&json!([
        {"name": "published", "type": "boolean"},
        {"name": "pages", "type": "integer"},
        {"name": "score", "type": "float"},
        {"name": "notes", "type": "text"},
        {"name": "code", "string": "Code", "type": "char"},
        {"name": "review_on", "type": "date"},
        {"name": "reviewed_at", "type": "datetime"},
        {"name": "status", "type": "selection", "selection": [["draft", "Draft"], ["decided", "Decided"]]},
        {"name": "areas", "type": "tags", "tags": [["sales", "Sales"], ["hr", "People"]]},
        {"name": "owner", "type": "user"},
        {"name": "source", "type": "link"}
    ]))
    .expect("a schema with every type")
}

fn values(value: &Value) -> Map<String, Value> {
    value.as_object().expect("an object").clone()
}

fn refusal(document: &Value) -> PropertyError {
    PropertySchema::parse(document).expect_err("the schema is refused")
}

#[test]
fn a_schema_with_every_type_parses_and_round_trips() {
    let parsed = schema();
    assert_eq!(parsed.0.len(), 11);
    assert_eq!(parsed.0[7].kind, PropertyType::Selection);
    let document = parsed.to_document().expect("serialises");
    assert_eq!(
        PropertySchema::parse(&document).expect("parses back"),
        parsed
    );
    assert_eq!(
        PropertySchema::parse(&json!([])).expect("empty"),
        PropertySchema::default()
    );
}

#[test]
fn a_definition_that_is_not_the_expected_shape_is_refused() {
    assert!(matches!(refusal(&json!({})), PropertyError::Shape(_)));
    assert!(matches!(
        refusal(&json!([{"name": "x", "type": "colour"}])),
        PropertyError::Shape(_)
    ));
    assert!(matches!(
        refusal(&json!([{"name": "x", "type": "text", "lookup": []}])),
        PropertyError::Shape(_)
    ));
}

#[test]
fn a_property_name_must_be_lowercase_snake_case() {
    for name in ["Status", "1st", "_x", "with space", "a-b", ""] {
        assert!(
            matches!(
                refusal(&json!([{"name": name, "type": "text"}])),
                PropertyError::InvalidName(_)
            ),
            "{name:?}"
        );
    }
}

#[test]
fn a_property_name_cannot_repeat() {
    let error = refusal(&json!([
        {"name": "status", "type": "text"},
        {"name": "status", "type": "char"}
    ]));
    assert!(matches!(error, PropertyError::DuplicateName(name) if name == "status"));
}

#[test]
fn options_belong_to_selection_and_tags_only() {
    assert!(matches!(
        refusal(&json!([{"name": "status", "type": "selection"}])),
        PropertyError::MissingOptions(_)
    ));
    assert!(matches!(
        refusal(&json!([{"name": "areas", "type": "tags", "tags": []}])),
        PropertyError::MissingOptions(_)
    ));
    assert!(matches!(
        refusal(&json!([{"name": "notes", "type": "text", "selection": [["a", "A"]]}])),
        PropertyError::UnexpectedOptions(_)
    ));
    assert!(matches!(
        refusal(
            &json!([{"name": "status", "type": "selection", "selection": [["a", "A"]], "tags": [["b", "B"]]}])
        ),
        PropertyError::UnexpectedOptions(_)
    ));
    assert!(matches!(
        refusal(
            &json!([{"name": "status", "type": "selection", "selection": [["a", "A"], ["a", "B"]]}])
        ),
        PropertyError::DuplicateOption { .. }
    ));
}

#[test]
fn a_value_of_every_type_is_accepted() {
    let written = values(&json!({
        "published": true,
        "pages": 12,
        "score": 4.5,
        "notes": "two\nlines",
        "code": "ABC-1",
        "review_on": "2026-11-30",
        "reviewed_at": "2026-11-25T10:00:00+01:00",
        "status": "decided",
        "areas": ["sales", "hr"],
        "owner": "8f7c1a2e-5b0d-4c3e-9a1f-2d6b7e8c9a0b",
        "source": "https://example.test/doc"
    }));
    schema()
        .validate(&written)
        .expect("every value fits its type");
}

#[test]
fn a_value_of_the_wrong_type_is_refused() {
    let cases = [
        ("published", json!("yes")),
        ("pages", json!(1.5)),
        ("score", json!("high")),
        ("notes", json!(3)),
        ("code", json!("two\nlines")),
        ("review_on", json!("30/11/2026")),
        ("reviewed_at", json!("2026-11-25")),
        ("status", json!("archived")),
        ("areas", json!("sales")),
        ("owner", json!("")),
        ("source", json!("ftp://example.test/doc")),
        ("source", json!("not a link")),
    ];
    for (name, value) in cases {
        let error = schema()
            .validate(&std::iter::once((name.to_owned(), value)).collect())
            .expect_err("refused");
        assert!(
            matches!(&error, PropertyError::WrongType { property, .. } if property == name),
            "{name}: {error:?}"
        );
    }
}

#[test]
fn an_undeclared_property_is_refused_and_null_unsets_any() {
    let error = schema()
        .validate(&values(&json!({"colour": "red"})))
        .expect_err("refused");
    assert!(matches!(error, PropertyError::Unknown(name) if name == "colour"));
    schema()
        .validate(&values(
            &json!({"status": null, "areas": null, "pages": null}),
        ))
        .expect("null clears a value");
}

#[test]
fn tags_must_be_declared_and_distinct() {
    for areas in [json!(["sales", "sales"]), json!(["finance"]), json!([1])] {
        assert!(
            schema()
                .validate(&values(&json!({ "areas": areas })))
                .is_err(),
            "{areas}"
        );
    }
    schema()
        .validate(&values(&json!({"areas": []})))
        .expect("no tag is a valid set");
}
