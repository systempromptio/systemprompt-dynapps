//! `map_fields`: fetched items to the secrets document.

use std::collections::BTreeMap;

use serde_json::json;
use systemprompt_secrets_onepassword::item::Item;
use systemprompt_secrets_onepassword::mapping::map_fields;
use systemprompt_secrets_onepassword::{Mapping, OnePasswordError, OpReference};

fn item(vault: &str, id: &str, title: &str, fields: &serde_json::Value) -> Item {
    let mut item: Item = serde_json::from_value(json!({
        "id": id,
        "title": title,
        "fields": fields,
    }))
    .unwrap();
    vault.clone_into(&mut item.vault);
    item
}

fn platform() -> Item {
    item(
        "prod",
        "abc123",
        "platform",
        &json!([
            {"id": "f1", "label": "anthropic", "type": "CONCEALED", "value": "sk-ant"},
            {"id": "f2", "label": "region", "type": "STRING", "value": "eu"},
            {"id": "f3", "label": "notes", "type": "OTP", "value": "123456"},
            {"id": "f4", "label": "empty", "type": "CONCEALED"},
        ]),
    )
}

fn refs(pairs: &[(&str, &str)]) -> Mapping {
    Mapping::Items(
        pairs
            .iter()
            .map(|(k, r)| ((*k).to_owned(), OpReference::parse(r).unwrap()))
            .collect::<BTreeMap<_, _>>(),
    )
}

#[test]
fn maps_by_label() {
    let doc = map_fields(
        &[platform()],
        &refs(&[
            ("anthropic", "op://prod/platform/anthropic"),
            ("r", "op://prod/platform/region"),
        ]),
    )
    .unwrap();
    assert_eq!(doc.key_names(), vec!["anthropic", "r"]);
}

#[test]
fn falls_back_to_field_id_and_item_id() {
    let doc = map_fields(&[platform()], &refs(&[("k", "op://prod/abc123/f1")])).unwrap();
    assert_eq!(doc.key_names(), vec!["k"]);
}

#[test]
fn non_text_fields_are_not_read() {
    let err = map_fields(&[platform()], &refs(&[("k", "op://prod/platform/notes")])).unwrap_err();
    assert!(matches!(err, OnePasswordError::FieldNotFound { .. }));
}

#[test]
fn field_without_value_is_not_found() {
    let err = map_fields(&[platform()], &refs(&[("k", "op://prod/platform/empty")])).unwrap_err();
    assert!(matches!(err, OnePasswordError::FieldNotFound { .. }));
}

#[test]
fn missing_item_and_wrong_vault() {
    let err = map_fields(&[platform()], &refs(&[("k", "op://prod/other/anthropic")])).unwrap_err();
    assert!(matches!(err, OnePasswordError::ItemNotFound { .. }));
    let err = map_fields(
        &[platform()],
        &refs(&[("k", "op://dev/platform/anthropic")]),
    )
    .unwrap_err();
    assert!(matches!(err, OnePasswordError::ItemNotFound { .. }));
}

#[test]
fn duplicate_item_title_is_ambiguous() {
    let mut twin = platform();
    "zzz".clone_into(&mut twin.id);
    let err = map_fields(
        &[platform(), twin],
        &refs(&[("k", "op://prod/platform/anthropic")]),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        OnePasswordError::AmbiguousItem { what: "item", .. }
    ));
}

#[test]
fn duplicate_field_label_is_ambiguous() {
    let dup = item(
        "prod",
        "id",
        "dup",
        &json!([
            {"id": "a", "label": "key", "type": "CONCEALED", "value": "1"},
            {"id": "b", "label": "key", "type": "STRING", "value": "2"},
        ]),
    );
    let err = map_fields(
        std::slice::from_ref(&dup),
        &refs(&[("k", "op://prod/dup/key")]),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        OnePasswordError::AmbiguousItem { what: "field", .. }
    ));
    let err = map_fields(
        &[dup],
        &Mapping::SingleItem {
            item: "dup".to_owned(),
        },
    )
    .unwrap_err();
    assert!(matches!(err, OnePasswordError::AmbiguousItem { .. }));
}

#[test]
fn single_item_takes_every_text_field_by_label() {
    let doc = map_fields(
        &[platform()],
        &Mapping::SingleItem {
            item: "platform".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(doc.key_names(), vec!["anthropic", "region"]);
}

#[test]
fn errors_never_carry_values() {
    let err = map_fields(&[platform()], &refs(&[("k", "op://prod/platform/missing")])).unwrap_err();
    assert!(!err.to_string().contains("sk-ant"));
}
