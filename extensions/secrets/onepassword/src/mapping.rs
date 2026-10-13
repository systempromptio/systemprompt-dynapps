//! Turns fetched items into the `secrets.json`-shaped document core validates.
//! Pure: no I/O, so every rule here is unit-testable.
//!
//! Only CONCEALED and STRING fields are read. A reference resolves its item by
//! title, then id, and its field by label, then id; two matches at the same
//! step are an error rather than a guess.

use serde_json::{Map, Value};
use systemprompt::config::SecretsDocument;

use crate::config::Mapping;
use crate::error::OnePasswordError;
use crate::item::{Field, Item};
use crate::reference::OpReference;

pub fn map_fields(items: &[Item], mapping: &Mapping) -> Result<SecretsDocument, OnePasswordError> {
    let mut doc = Map::new();
    match mapping {
        Mapping::Items(refs) => {
            for (key, reference) in refs {
                let item = find_item(items, &reference.vault, &reference.item)?;
                let value = field_value(item, reference)?;
                doc.insert(key.clone(), Value::String(value));
            }
        },
        Mapping::SingleItem { item } => {
            let found = items
                .iter()
                .filter(|i| i.title == *item || i.id == *item)
                .collect::<Vec<_>>();
            let found = single(found, "item", item, || OnePasswordError::ItemNotFound {
                vault: items.first().map(|i| i.vault.clone()).unwrap_or_default(),
                item: item.clone(),
            })?;
            for field in found.fields.iter().filter(|f| f.is_readable()) {
                let (Some(label), Some(value)) = (&field.label, &field.value) else {
                    continue;
                };
                if label.is_empty() {
                    continue;
                }
                if doc.contains_key(label) {
                    return Err(OnePasswordError::AmbiguousItem {
                        what: "field label",
                        name: label.clone(),
                    });
                }
                doc.insert(label.clone(), Value::String(value.clone()));
            }
        },
    }
    Ok(SecretsDocument::new(doc))
}

fn find_item<'a>(items: &'a [Item], vault: &str, name: &str) -> Result<&'a Item, OnePasswordError> {
    let in_vault: Vec<&Item> = items.iter().filter(|i| i.vault == vault).collect();
    let not_found = || OnePasswordError::ItemNotFound {
        vault: vault.to_owned(),
        item: name.to_owned(),
    };
    let by_title: Vec<&Item> = in_vault
        .iter()
        .copied()
        .filter(|i| i.title == name)
        .collect();
    if !by_title.is_empty() {
        return single(by_title, "item", name, not_found);
    }
    let by_id = in_vault.into_iter().filter(|i| i.id == name).collect();
    single(by_id, "item", name, not_found)
}

fn field_value(item: &Item, reference: &OpReference) -> Result<String, OnePasswordError> {
    let name = reference.field.as_str();
    let readable: Vec<&Field> = item.fields.iter().filter(|f| f.is_readable()).collect();
    let not_found = || OnePasswordError::FieldNotFound {
        item: reference.item.clone(),
        field: name.to_owned(),
    };
    let by_label: Vec<&Field> = readable
        .iter()
        .copied()
        .filter(|f| f.label.as_deref() == Some(name))
        .collect();
    let field = if by_label.is_empty() {
        let by_id = readable.into_iter().filter(|f| f.id == name).collect();
        single(by_id, "field", name, not_found)?
    } else {
        single(by_label, "field", name, not_found)?
    };
    field.value.clone().ok_or_else(not_found)
}

fn single<T>(
    mut found: Vec<T>,
    what: &'static str,
    name: &str,
    not_found: impl FnOnce() -> OnePasswordError,
) -> Result<T, OnePasswordError> {
    match found.len() {
        0 => Err(not_found()),
        1 => Ok(found.remove(0)),
        _ => Err(OnePasswordError::AmbiguousItem {
            what,
            name: name.to_owned(),
        }),
    }
}
