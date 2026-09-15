//! Active-device security and transaction locking on both installation paths.

use sqlx::PgPool;

pub(super) async fn contracts(pool: &PgPool) {
    for (name, volatility) in [
        ("active_device_identity", "v"),
        ("active_devices_for_consumer", "s"),
    ] {
        let (security_definer, actual_volatility, configuration): (bool, String, Vec<String>) =
            sqlx::query_as(
                "SELECT p.prosecdef, p.provolatile::text, p.proconfig FROM pg_proc p \
                 JOIN pg_namespace n ON n.oid=p.pronamespace \
                 WHERE n.nspname='public' AND p.proname=$1 \
                 AND pg_get_function_identity_arguments(p.oid) IN \
                   ('requested_device text','requested_consumer text')",
            )
            .bind(name)
            .fetch_one(pool)
            .await
            .expect("public active-device function");
        assert!(!security_definer, "{name} must preserve caller privileges");
        assert_eq!(actual_volatility, volatility, "{name} volatility");
        assert_eq!(configuration, ["search_path=pg_catalog, public"]);
    }

    sqlx::query("INSERT INTO user_device_certs(id,user_id,fingerprint,label) VALUES('upgrade-lock-device','feedback-consumer','upgrade-lock-fingerprint','acceptance')")
        .execute(pool).await.expect("active fixture device");
    let mut reader = pool.begin().await.expect("identity transaction");
    let identity: (String, String) = sqlx::query_as(
        "SELECT device_id,consumer_id FROM public.active_device_identity('upgrade-lock-device')",
    )
    .fetch_one(&mut *reader)
    .await
    .expect("active identity");
    assert_eq!(
        identity,
        ("upgrade-lock-device".into(), "feedback-consumer".into())
    );

    let mut revoker = pool
        .begin()
        .await
        .expect("independent revocation transaction");
    sqlx::query("SET LOCAL lock_timeout='150ms'")
        .execute(&mut *revoker)
        .await
        .expect("bounded lock probe");
    let error =
        sqlx::query("UPDATE user_device_certs SET revoked_at=now() WHERE id='upgrade-lock-device'")
            .execute(&mut *revoker)
            .await
            .expect_err("identity holds a shared row lock until transaction end");
    assert_eq!(
        error
            .as_database_error()
            .and_then(|error| error.code())
            .as_deref(),
        Some("55P03")
    );
    revoker.rollback().await.expect("release failed writer");
    reader.commit().await.expect("release identity lock");

    let mut revoker = pool
        .begin()
        .await
        .expect("revocation after reader completion");
    sqlx::query("SET LOCAL lock_timeout='150ms'")
        .execute(&mut *revoker)
        .await
        .expect("bounded post-release probe");
    assert_eq!(
        sqlx::query("UPDATE user_device_certs SET revoked_at=now() WHERE id='upgrade-lock-device'")
            .execute(&mut *revoker)
            .await
            .expect("released lock allows revocation")
            .rows_affected(),
        1
    );
    revoker.commit().await.expect("commit revocation");

    for query in [
        "SELECT count(*) FROM public.active_device_identity('upgrade-lock-device')",
        "SELECT count(*) FROM public.active_devices_for_consumer('feedback-owner') WHERE device_id='feedback-device'",
        "SELECT count(*) FROM public.active_devices_for_consumer('feedback-consumer') WHERE device_id='upgrade-lock-device'",
    ] {
        let count: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(query))
            .fetch_one(pool)
            .await
            .expect("scoped revoked-device projection");
        assert_eq!(count, 0);
    }
    sqlx::query("DELETE FROM user_device_certs WHERE id='upgrade-lock-device'")
        .execute(pool)
        .await
        .expect("remove acceptance-only device");
}
