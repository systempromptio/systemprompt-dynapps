//! Database console behavior in disposable databases, never in live data.
use crate::tempdb::TempDb;
use systemprompt_web_admin_platform::repositories::data_console::browse::{
    self, Filter, Selection,
};
use systemprompt_web_admin_platform::repositories::data_console::{clear, query};

async fn fixture() -> Option<TempDb> {
    let db = TempDb::create().await?;
    sqlx::raw_sql("CREATE TABLE dc_parent(id integer PRIMARY KEY, label text);
        CREATE TABLE dc_child(id integer PRIMARY KEY, parent_id integer REFERENCES dc_parent(id), label text);
        CREATE TABLE dc_null(id integer PRIMARY KEY, parent_id integer REFERENCES dc_parent(id) ON DELETE SET NULL);
        INSERT INTO dc_parent VALUES(1,'delete'),(2,'keep');
        INSERT INTO dc_child VALUES(1,1,'delete'),(2,2,'keep');
        INSERT INTO dc_null VALUES(1,1),(2,2);").execute(&*db.pool).await.unwrap();
    Some(db)
}
fn selection() -> Selection {
    Selection {
        table: "public.dc_parent".into(),
        filters: vec![Filter {
            column: "id".into(),
            value: "1".into(),
        }],
        ..Selection::default()
    }
}

#[tokio::test]
async fn data_console_filtered_clear_includes_restrictive_children_and_updates() {
    let Some(db) = fixture().await else {
        // skip-ok: fixture delegates to TempDb::create, which fails hard under CI
        // without a database.
        return;
    };
    let preview = clear::insert_clear_preview(&db.pool, "actor", &selection())
        .await
        .unwrap();
    assert_eq!(
        preview
            .plan
            .effects
            .iter()
            .find(|e| e.table == "public.dc_child")
            .unwrap()
            .deleted,
        1
    );
    assert_eq!(
        preview
            .plan
            .effects
            .iter()
            .find(|e| e.table == "public.dc_null")
            .unwrap()
            .updated,
        1
    );
    assert!(
        clear::delete_confirmed_rows(&db.pool, "other", &preview.token, "public.dc_parent")
            .await
            .is_err()
    );
    assert!(
        clear::delete_confirmed_rows(&db.pool, "actor", &preview.token, "wrong")
            .await
            .is_err()
    );
    let result =
        clear::delete_confirmed_rows(&db.pool, "actor", &preview.token, "public.dc_parent")
            .await
            .unwrap();
    assert_eq!(result.effects.iter().map(|e| e.deleted).sum::<i64>(), 2);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM dc_parent")
            .fetch_one(&*db.pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, Option<i32>>("SELECT parent_id FROM dc_null WHERE id=1")
            .fetch_one(&*db.pool)
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM platform_data_clear_audit")
            .fetch_one(&*db.pool)
            .await
            .unwrap(),
        1
    );
    assert!(
        clear::delete_confirmed_rows(&db.pool, "actor", &preview.token, "public.dc_parent")
            .await
            .is_err()
    );
    db.cleanup().await;
}
#[tokio::test]
async fn data_console_changed_data_or_expired_ticket_cannot_clear() {
    let Some(db) = fixture().await else {
        // skip-ok: fixture delegates to TempDb::create, which fails hard under CI
        // without a database.
        return;
    };
    let preview = clear::insert_clear_preview(&db.pool, "actor", &selection())
        .await
        .unwrap();
    sqlx::query("UPDATE dc_parent SET label='changed' WHERE id=1")
        .execute(&*db.pool)
        .await
        .unwrap();
    assert!(
        clear::delete_confirmed_rows(&db.pool, "actor", &preview.token, "public.dc_parent")
            .await
            .is_err()
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM dc_parent")
            .fetch_one(&*db.pool)
            .await
            .unwrap(),
        2
    );
    let preview = clear::insert_clear_preview(&db.pool, "actor", &selection())
        .await
        .unwrap();
    sqlx::query("UPDATE platform_data_clear_previews SET expires_at=now()-interval '1 second'")
        .execute(&*db.pool)
        .await
        .unwrap();
    assert!(
        clear::delete_confirmed_rows(&db.pool, "actor", &preview.token, "public.dc_parent")
            .await
            .is_err()
    );
    db.cleanup().await;
}
#[tokio::test]
async fn data_console_constraint_failure_rolls_back_whole_clear() {
    let Some(db) = fixture().await else {
        // skip-ok: fixture delegates to TempDb::create, which fails hard under CI
        // without a database.
        return;
    };
    sqlx::raw_sql("CREATE TABLE dc_default(id integer PRIMARY KEY,parent_id integer DEFAULT 99 REFERENCES dc_parent(id) ON DELETE SET DEFAULT);
        INSERT INTO dc_default VALUES(1,1);").execute(&*db.pool).await.unwrap();
    let preview = clear::insert_clear_preview(&db.pool, "actor", &selection())
        .await
        .unwrap();
    assert!(
        clear::delete_confirmed_rows(&db.pool, "actor", &preview.token, "public.dc_parent")
            .await
            .is_err()
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM dc_child")
            .fetch_one(&*db.pool)
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM platform_data_clear_audit")
            .fetch_one(&*db.pool)
            .await
            .unwrap(),
        0
    );
    db.cleanup().await;
}
#[tokio::test]
async fn data_console_keyset_pagination_and_limits_are_bounded() {
    let Some(db) = fixture().await else {
        // skip-ok: fixture delegates to TempDb::create, which fails hard under CI
        // without a database.
        return;
    };
    sqlx::query("INSERT INTO dc_parent SELECT n, repeat('x',1500) FROM generate_series(3,103) n")
        .execute(&*db.pool)
        .await
        .unwrap();
    let s = Selection {
        table: "public.dc_parent".into(),
        ..Selection::default()
    };
    let first = browse::get_table_page(&db.pool, &s, None).await.unwrap();
    assert_eq!(first.rows.len(), 50);
    assert!(first.next.is_some());
    let second = browse::get_table_page(&db.pool, &s, first.next.as_deref())
        .await
        .unwrap();
    assert_eq!(second.rows.len(), 50);
    assert_ne!(first.row_keys[49], second.row_keys[0]);
    assert!(
        first.rows[2]["label"]
            .as_str()
            .unwrap()
            .ends_with(" [display truncated]")
    );
    let detail = browse::find_row_detail(&db.pool, "public.dc_parent", &first.row_keys[2])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(detail["label"].as_str().unwrap().len(), 1500);
    assert!(
        browse::get_table_page(&db.pool, &s, Some("injected cursor"))
            .await
            .is_err()
    );
    let mut injection = selection();
    injection.filters[0].column = "id); DELETE FROM dc_parent;--".into();
    assert!(
        browse::get_table_page(&db.pool, &injection, None)
            .await
            .is_err()
    );
    let result = query::get_query_result(&db.pool, "SELECT n FROM generate_series(1,300) n")
        .await
        .unwrap();
    assert_eq!(result.rows.len(), 200);
    assert!(result.limited);
    db.cleanup().await;
}
#[tokio::test]
async fn data_console_nondeferrable_cycles_are_blocked() {
    let Some(db) = fixture().await else {
        // skip-ok: fixture delegates to TempDb::create, which fails hard under CI
        // without a database.
        return;
    };
    sqlx::raw_sql("CREATE TABLE dc_a(id integer PRIMARY KEY,b integer); CREATE TABLE dc_b(id integer PRIMARY KEY,a integer REFERENCES dc_a(id));
        ALTER TABLE dc_a ADD CONSTRAINT dc_cycle FOREIGN KEY(b) REFERENCES dc_b(id);").execute(&*db.pool).await.unwrap();
    let s = Selection {
        table: "public.dc_a".into(),
        ..Selection::default()
    };
    assert!(
        clear::insert_clear_preview(&db.pool, "actor", &s)
            .await
            .is_err()
    );
    db.cleanup().await;
}
#[tokio::test]
async fn data_console_clearing_audit_table_still_records_the_operation() {
    let Some(db) = fixture().await else {
        // skip-ok: fixture delegates to TempDb::create, which fails hard under CI
        // without a database.
        return;
    };
    let preview = clear::insert_clear_preview(&db.pool, "actor", &selection())
        .await
        .unwrap();
    clear::delete_confirmed_rows(&db.pool, "actor", &preview.token, "public.dc_parent")
        .await
        .unwrap();
    let s = Selection {
        table: "public.platform_data_clear_audit".into(),
        ..Selection::default()
    };
    let preview = clear::insert_clear_preview(&db.pool, "actor", &s)
        .await
        .unwrap();
    clear::delete_confirmed_rows(&db.pool, "actor", &preview.token, &s.table)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM platform_data_clear_audit")
            .fetch_one(&*db.pool)
            .await
            .unwrap(),
        1
    );
    db.cleanup().await;
}

#[tokio::test]
async fn data_console_logs_cursor_handles_identical_timestamps_and_indexed_reads() {
    let Some(db) = fixture().await else {
        // skip-ok: fixture delegates to TempDb::create, which fails hard under CI
        // without a database.
        return;
    };
    sqlx::query("INSERT INTO logs(id,timestamp,level,module,message) SELECT 'dc-log-'||lpad(n::text,4,'0'), '2026-10-04T02:00:00Z'::timestamptz,'ERROR','data-console-cursor','line' FROM generate_series(1,101) n")
        .execute(&*db.pool).await.unwrap();
    let s = Selection {
        table: "public.logs".into(),
        filters: vec![Filter {
            column: "module".into(),
            value: "data-console-cursor".into(),
        }],
        ..Selection::default()
    };
    let first = browse::get_table_page(&db.pool, &s, None).await.unwrap();
    let second = browse::get_table_page(&db.pool, &s, first.next.as_deref())
        .await
        .unwrap();
    assert_eq!(first.rows.len(), 50);
    assert_eq!(second.rows.len(), 50);
    assert!(
        first
            .row_keys
            .iter()
            .all(|key| !second.row_keys.contains(key))
    );
    let plan:serde_json::Value=sqlx::query_scalar("EXPLAIN (FORMAT JSON) SELECT id FROM logs WHERE level='ERROR' AND timestamp >= '2026-10-04T01:00:00Z' ORDER BY timestamp DESC,id DESC LIMIT 51")
        .fetch_one(&*db.pool).await.unwrap();
    assert!(
        plan.to_string().contains("idx_logs_level_timestamp_id"),
        "{plan}"
    );
    db.cleanup().await;
}
#[tokio::test]
async fn data_console_sql_timeouts_and_transaction_readonly_are_enforced() {
    let Some(db) = fixture().await else {
        // skip-ok: fixture delegates to TempDb::create, which fails hard under CI
        // without a database.
        return;
    };
    sqlx::raw_sql("CREATE FUNCTION dc_write() RETURNS integer LANGUAGE plpgsql AS $$ BEGIN INSERT INTO dc_parent VALUES(3,'forbidden'); RETURN 1; END $$;").execute(&*db.pool).await.unwrap();
    assert!(
        query::get_query_result(&db.pool, "SELECT dc_write()")
            .await
            .is_err()
    );
    assert!(
        query::get_query_result(&db.pool, "SELECT pg_sleep(5)")
            .await
            .is_err()
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM dc_parent")
            .fetch_one(&*db.pool)
            .await
            .unwrap(),
        2
    );
    assert!(
        query::get_query_result(&db.pool, "SELECT 1").await.is_ok(),
        "connection remains usable after rollback"
    );
    db.cleanup().await;
}
#[tokio::test]
async fn data_console_wide_tables_and_composite_keys_work() {
    let Some(db) = fixture().await else {
        // skip-ok: fixture delegates to TempDb::create, which fails hard under CI
        // without a database.
        return;
    };
    let fields = (0..70)
        .map(|n| format!("c{n} text DEFAULT 'value'"))
        .collect::<Vec<_>>()
        .join(",");
    let ddl = format!(
        "CREATE TABLE dc_wide(a integer,b integer,{fields},PRIMARY KEY(a,b)); INSERT INTO dc_wide(a,b) SELECT n/3,n FROM generate_series(1,101) n;"
    );
    sqlx::raw_sql(sqlx::AssertSqlSafe(ddl))
        .execute(&*db.pool)
        .await
        .unwrap();
    let s = Selection {
        table: "public.dc_wide".into(),
        ..Selection::default()
    };
    let first = browse::get_table_page(&db.pool, &s, None).await.unwrap();
    let second = browse::get_table_page(&db.pool, &s, first.next.as_deref())
        .await
        .unwrap();
    sqlx::raw_sql("CREATE TABLE dc_no_key(value text); INSERT INTO dc_no_key SELECT n::text FROM generate_series(1,60) n;").execute(&*db.pool).await.unwrap();
    let no_key = browse::get_table_page(
        &db.pool,
        &Selection {
            table: "public.dc_no_key".into(),
            ..Selection::default()
        },
        None,
    )
    .await
    .unwrap();
    assert_eq!(no_key.rows.len(), 50);
    assert!(no_key.next.is_none());
    assert_eq!(first.rows[0].as_object().unwrap().len(), 72);
    assert_eq!(second.rows.len(), 50);
    assert!(
        first
            .row_keys
            .iter()
            .all(|key| !second.row_keys.contains(key))
    );
    db.cleanup().await;
}

#[tokio::test]
async fn data_console_includes_application_schemas_with_nonreserved_pg_prefixes() {
    let Some(db) = fixture().await else {
        // skip-ok: fixture delegates to TempDb::create, which fails hard under CI
        // without a database.
        return;
    };
    sqlx::raw_sql("CREATE SCHEMA pgapp; CREATE TABLE pgapp.example(id integer PRIMARY KEY); INSERT INTO pgapp.example VALUES(1)")
        .execute(&*db.pool).await.unwrap();
    let selected = Selection {
        table: "pgapp.example".into(),
        ..Selection::default()
    };
    let page = browse::get_table_page(&db.pool, &selected, None)
        .await
        .unwrap();
    assert_eq!(page.rows.len(), 1);
    db.cleanup().await;
}
