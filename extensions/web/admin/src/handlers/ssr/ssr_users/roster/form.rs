//! The roster's filter form: the hidden fields that keep a reader's
//! selection across a scope change, and the role select's options.

use super::RosterUrlState;
use super::context::RoleOptionView;

pub(super) fn preserved_hidden(url: &RosterUrlState) -> Vec<(String, String)> {
    vec![
        ("filter".to_owned(), url.filter.as_str().to_owned()),
        ("role".to_owned(), url.role.clone().unwrap_or_default()),
        ("q".to_owned(), url.search.clone().unwrap_or_default()),
        ("sort".to_owned(), url.sort.column.to_owned()),
        (
            "dir".to_owned(),
            if url.sort.descending { "desc" } else { "asc" }.to_owned(),
        ),
    ]
}


pub(super) fn role_options(roles: &[String], selected: Option<&str>) -> Vec<RoleOptionView> {
    let mut out = vec![RoleOptionView {
        value: String::new(),
        label: "All roles".to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(roles.iter().map(|role| RoleOptionView {
        selected: selected == Some(role.as_str()),
        value: role.clone(),
        label: role.clone(),
    }));
    out
}
