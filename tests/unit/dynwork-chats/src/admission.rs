//! Who may use which model, and that every failure refuses: a hook built once
//! and only when first asked, a roles lookup that fails or finds no user, a
//! hook that cannot be built (the deny-all hook) and a secret store that
//! cannot be listed.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use dynwork_chats::admission::{Admission, LazyHook};
use dynwork_chats::auth::{Caller, Client};
use dynwork_chats::error::ChatsError;
use dynwork_chats::roles::Roles;
use dynwork_chats::store::ModelRef;
use systemprompt::governance::authz::{AllowAllHook, DenyAllHook, SharedAuthzHook};
use systemprompt::identifiers::UserId;
use systemprompt::traits::auth::AuthProviderError;

use crate::catalog::fixture;

#[derive(Debug, Clone)]
enum Lookup {
    Roles(Vec<String>),
    NoSuchUser,
    Fails,
}

#[derive(Debug)]
struct FixedRoles(Lookup);

#[async_trait]
impl Roles for FixedRoles {
    async fn of(&self, _user: &UserId) -> Result<Option<Vec<String>>, ChatsError> {
        match &self.0 {
            Lookup::Roles(roles) => Ok(Some(roles.clone())),
            Lookup::NoSuchUser => Ok(None),
            Lookup::Fails => Err(ChatsError::Users(AuthProviderError::InvalidCredentials)),
        }
    }
}

fn caller() -> Caller {
    Caller {
        user: UserId::new("u1"),
        client: Client::Desktop,
        peer: None,
        client_id: None,
        session: None,
    }
}

fn model(provider: &str, id: &str) -> ModelRef {
    ModelRef {
        provider_id: provider.to_owned(),
        model_id: id.to_owned(),
    }
}

fn names(held: &[&str]) -> Arc<dyn Fn() -> BTreeSet<String> + Send + Sync> {
    let held: BTreeSet<String> = held.iter().map(|name| (*name).to_owned()).collect();
    Arc::new(move || held.clone())
}

fn admission(
    secrets: &[&str],
    hook: impl Fn() -> SharedAuthzHook + Send + Sync + 'static,
    roles: Lookup,
) -> Admission {
    Admission::new(
        names(secrets),
        LazyHook::new(hook),
        Arc::new(FixedRoles(roles)),
    )
}

async fn admits(admission: &Admission, provider: &str, id: &str) -> Result<bool, ChatsError> {
    let listing = admission.listing(&fixture(), &caller()).await?;
    Ok(listing.contains(&model(provider, id)))
}

fn allow_all() -> SharedAuthzHook {
    Arc::new(AllowAllHook::null())
}

fn staff() -> Lookup {
    Lookup::Roles(vec!["staff".to_owned()])
}

#[tokio::test]
async fn the_hook_is_built_when_first_asked_and_only_once() {
    let built = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&built);
    let admission = admission(
        &["alpha"],
        move || {
            counter.fetch_add(1, Ordering::SeqCst);
            allow_all()
        },
        staff(),
    );
    assert_eq!(built.load(Ordering::SeqCst), 0, "not at construction");
    let catalog = fixture();
    admission.listing(&catalog, &caller()).await.expect("a list");
    admission.listing(&catalog, &caller()).await.expect("a list");
    admits(&admission, "alpha", "m-big").await.expect("an answer");
    assert_eq!(built.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_model_is_admitted_when_it_is_listed_under_that_provider() {
    let admission = admission(&["alpha"], allow_all, staff());
    assert!(admits(&admission, "alpha", "m-big").await.expect("an answer"));
    for (provider, id, why) in [
        ("beta", "m-big", "another provider"),
        ("alpha", "m-hidden", "not advertised"),
        ("alpha", "m-beta", "its provider has no secret"),
        ("alpha", "no-such-model", "unknown"),
    ] {
        assert!(!admits(&admission, provider, id).await.expect("an answer"), "{why}");
    }
}

#[tokio::test]
async fn a_hook_that_cannot_be_built_refuses_every_model() {
    let catalog = fixture();
    let admission = admission(&["alpha"], || Arc::new(DenyAllHook::null()), staff());
    let listing = admission.listing(&catalog, &caller()).await.expect("a list");
    assert!(listing.models.is_empty());
    assert_eq!(listing.default, None);
    assert!(!admits(&admission, "alpha", "m-big").await.expect("an answer"));
}

#[tokio::test]
async fn a_failed_roles_lookup_is_an_error_never_the_catalog() {
    let catalog = fixture();
    let admission = admission(&["alpha"], allow_all, Lookup::Fails);
    let listing = admission.listing(&catalog, &caller()).await;
    assert!(matches!(listing, Err(ChatsError::Users(_))), "{listing:?}");
    let admitted = admits(&admission, "alpha", "m-big").await;
    assert!(matches!(admitted, Err(ChatsError::Users(_))), "{admitted:?}");
}

#[tokio::test]
async fn a_user_the_instance_does_not_have_is_unauthorized() {
    let catalog = fixture();
    let admission = admission(&["alpha"], allow_all, Lookup::NoSuchUser);
    let listing = admission.listing(&catalog, &caller()).await;
    assert!(matches!(listing, Err(ChatsError::Auth(_))), "{listing:?}");
    let admitted = admits(&admission, "alpha", "m-big").await;
    assert!(matches!(admitted, Err(ChatsError::Auth(_))), "{admitted:?}");
}

#[tokio::test]
async fn secrets_that_cannot_be_listed_serve_no_model() {
    let catalog = fixture();
    let admission = admission(&[], allow_all, staff());
    let listing = admission.listing(&catalog, &caller()).await.expect("a list");
    assert!(listing.models.is_empty());
}
