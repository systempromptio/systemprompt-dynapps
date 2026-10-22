//! One `AuthzCaches` generation backs every authorization cache for a
//! database: a value read through a cache misses after `invalidate_all`, and
//! caches on the same generation drop together.

use std::sync::Arc;
use std::time::Duration;

use systemprompt_web_admin_core::authz::AuthzCaches;
use systemprompt_web_admin_core::ttl_cache::TtlCache;

#[test]
fn invalidate_all_makes_a_cached_grant_miss() {
    let caches = AuthzCaches::new();
    let key = ("gateway_route".to_owned(), "claude-sonnet".to_owned());
    caches.entity_grants.insert(key.clone(), (None, Vec::new()));
    assert!(caches.entity_grants.get(&key).is_some());

    caches.invalidate_all();

    assert!(
        caches.entity_grants.get(&key).is_none(),
        "an entry stamped with an older generation is a miss"
    );
}

#[test]
fn caches_sharing_a_generation_drop_together() {
    let caches = AuthzCaches::new();
    let subjects: TtlCache<u8, Vec<String>> =
        TtlCache::new(Duration::from_secs(30), caches.generation());
    subjects.insert(1, vec!["group-a".to_owned()]);
    caches.user_roles.insert(
        systemprompt::identifiers::UserId::new("user-1".to_owned()),
        vec!["user".to_owned()],
    );

    caches.invalidate_all();

    assert!(subjects.get(&1).is_none());
    assert!(
        caches
            .user_roles
            .get(&systemprompt::identifiers::UserId::new("user-1".to_owned()))
            .is_none()
    );
}

#[test]
fn a_fresh_insert_after_invalidation_is_served() {
    let caches = AuthzCaches::new();
    caches.invalidate_all();
    let private: TtlCache<&str, u8> = TtlCache::new(Duration::from_secs(30), caches.generation());
    private.insert("k", 7);
    assert_eq!(private.get(&"k"), Some(7));
    let _unused = Arc::clone(&caches.generation());
}
