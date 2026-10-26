//! Authorization, safe rendering, and malformed-tool transcript/export
//! regression.
use crate::app::{App, Call};
use crate::globals;
use crate::principal::{self, Principal};
use crate::seed::{self, RequestSpec};
use crate::tempdb::TempDb;
use axum::http::StatusCode;

#[tokio::test]
async fn platform_data_pages_are_admin_only_and_escape_rows() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    sqlx::query("INSERT INTO logs(level,module,message,metadata) VALUES('ERROR','console-test','<script>alert(1)</script>','{broken metadata')")
        .execute(&*db.pool).await.unwrap();
    for path in ["/admin/logs", "/admin/sql"] {
        let (status, body) = app.call(Call::get(path, Principal::PlatformAdmin)).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert!(body.contains("Read-only SQL") || body.contains("Persisted application"));
        assert_eq!(
            app.call(Call::get(path, Principal::Admin)).await.0,
            StatusCode::FORBIDDEN
        );
        assert!(
            !app.call(Call::get(path, Principal::NonAdmin))
                .await
                .0
                .is_success()
        );
    }
    let (_, body) = app
        .call(Call::get(
            "/admin/logs?module=console-test",
            Principal::PlatformAdmin,
        ))
        .await;
    assert!(body.contains("&lt;script&gt;"));
    assert!(!body.contains("<script>alert(1)</script>"));
    let form = "query=SELECT+1+AS+answer";
    let call = Call {
        method: "post",
        path: "/admin/sql/query",
        principal: Principal::Admin,
        content_type: Some("application/x-www-form-urlencoded"),
        body: Some(form),
    };
    assert_eq!(app.call(call).await.0, StatusCode::FORBIDDEN);
    db.cleanup().await;
}

#[tokio::test]
async fn malformed_tool_arguments_keep_transcript_and_exports_readable() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let user = credentials.admin_user_id.clone();
    let context = uuid::Uuid::new_v4().to_string();
    seed::insert_context(&db.pool, &context, &user, None, "Malformed tool arguments").await;
    let request = seed::unique("malformed-request");
    seed::insert_request(
        &db.pool,
        &RequestSpec {
            id: request.clone(),
            user_id: &user,
            session_id: None,
            trace_id: None,
            context_id: Some(&context),
            status: "completed",
        },
    )
    .await;
    seed::insert_offered_tools(&db.pool, &request).await;
    sqlx::query("INSERT INTO ai_request_messages(request_id,role,sequence_number,content) VALUES($1,'user',0,'Please write the file')")
        .bind(&request).execute(&*db.pool).await.unwrap();
    sqlx::query("INSERT INTO ai_request_messages(request_id,role,sequence_number,content) VALUES($1,'assistant',1,$2)")
        .bind(&request).bind("[tool_use:Write {}]").execute(&*db.pool).await.unwrap();
    let raw = "{\"content\":\"unfinished";
    sqlx::query("INSERT INTO ai_request_tool_calls(id,request_id,tool_name,tool_input,sequence_number,ai_tool_call_id) VALUES($1,$2,'Write',$3,0,'toolu_contract')")
        .bind(seed::unique("tool")).bind(&request).bind(raw).execute(&*db.pool).await.unwrap();
    let app = App::new(&db.pool, credentials);
    for suffix in [
        format!("/admin/contexts/{context}"),
        format!("/admin/export/transcripts/{context}"),
        format!("/admin/export/transcripts/{context}?format=markdown"),
    ] {
        let (status, body) = app.call(Call::get(&suffix, Principal::Admin)).await;
        assert_eq!(status, StatusCode::OK, "{suffix}: {body}");
        if suffix.contains("format=markdown") {
            assert!(body.contains("Stored arguments could not be parsed"));
        } else if suffix.contains("export") {
            let doc: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert_eq!(doc["tool_calls"][0]["tool_input_raw"], raw);
        }
    }
    db.cleanup().await;
}

#[tokio::test]
async fn platform_forms_run_queries_and_clear_only_after_confirmation() {
    assert!(globals::init());
    let db = TempDb::create().await.expect("PostgreSQL required");
    sqlx::raw_sql(
        "CREATE TABLE dc_form(id integer PRIMARY KEY); INSERT INTO dc_form VALUES(1),(2)",
    )
    .execute(&*db.pool)
    .await
    .unwrap();
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let origin = globals::boot().config().api_external_url.clone();
    let request = |path, body| Call {
        method: "post",
        path,
        principal: Principal::PlatformAdmin,
        content_type: Some("application/x-www-form-urlencoded"),
        body: Some(body),
    };
    let form = "query=SELECT+42+AS+answer";
    assert_eq!(
        app.call(request("/admin/sql/query", form)).await.0,
        StatusCode::FORBIDDEN
    );
    let (status, body) = app
        .call_with_headers(request("/admin/sql/query", form), &[("origin", &origin)])
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains("Query completed: 1 rows displayed."));
    assert!(body.contains("42"));
    let selection = r#"{"table":"public.dc_form","filters":[]}"#;
    let form = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("selection", selection)
        .append_pair("mode", "all")
        .append_pair("source", "sql")
        .finish();
    let (status, body) = app
        .call_with_headers(
            request("/admin/data/clear/preview", &form),
            &[("origin", &origin)],
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains("Confirm clearing public.dc_form"), "{body}");
    let token = body
        .split("name=\"token\" value=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM dc_form")
            .fetch_one(&*db.pool)
            .await
            .unwrap(),
        2
    );
    let form = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("token", token)
        .append_pair("confirmation", "public.dc_form")
        .append_pair("acknowledge", "yes")
        .append_pair("source", "sql")
        .finish();
    let (status, body) = app
        .call_with_headers(
            request("/admin/data/clear/confirm", &form),
            &[("origin", &origin)],
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains("Clearing completed."), "{body}");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM dc_form")
            .fetch_one(&*db.pool)
            .await
            .unwrap(),
        0
    );
    db.cleanup().await;
}
