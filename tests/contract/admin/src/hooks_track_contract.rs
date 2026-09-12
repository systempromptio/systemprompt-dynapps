//! Hook ingestion accepts validated, identified events and rejects malformed
//! payloads. A successful response must leave durable evidence; delivery
//! retries must not duplicate events or counters.

use axum::http::StatusCode;
use systemprompt::models::auth::{JwtAudience, Permission};

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::seed::{self, TokenSpec};
use crate::tempdb::TempDb;
use crate::{globals, principal};

const TRACK: &str = "/hooks/track";

// A hook request carries its own bearer, so it is issued as an anonymous
// principal with the token spelled out rather than through `Credentials`.
fn hook_call<'a>(token: &'a str, body: &'a str) -> (Call<'a>, &'a str) {
    (
        Call {
            method: "post",
            path: TRACK,
            principal: Principal::Anonymous,
            content_type: Some("application/json"),
            body: Some(body),
        },
        token,
    )
}

// One event payload and the event name the handler must record for it.
struct EventCase {
    label: &'static str,
    body: String,
    // None marks an invalid event that must return 400.
    recorded_as: Option<&'static str>,
}

fn common(session: &str, event: &str) -> String {
    format!(
        r#""session_id":"{session}","event_id":"{session}-{event}","cwd":"/tmp/contract","permission_mode":"default","transcript_path":"/tmp/contract/t.jsonl","hook_event_name":"{event}""#
    )
}

fn cases(session: &str) -> Vec<EventCase> {
    let ev = |label: &'static str,
              name: &'static str,
              rest: &str,
              recorded_as: Option<&'static str>| EventCase {
        label,
        body: format!("{{{},{rest}}}", common(session, name)),
        recorded_as,
    };

    vec![
        ev(
            "session start",
            "SessionStart",
            r#""source":"startup","model":"claude-contract-model""#,
            Some("SessionStart"),
        ),
        ev(
            "session end",
            "SessionEnd",
            r#""reason":"clear""#,
            Some("SessionEnd"),
        ),
        ev(
            "user prompt",
            "UserPromptSubmit",
            r#""prompt":"Explain the governance chain in one paragraph.""#,
            Some("UserPromptSubmit"),
        ),
        // Recorded, not dropped. The governance *decision* still belongs to
        // `/hooks/govern`; this row is the attempt itself, which the grant-rate
        // proxy needs paired with its PostToolUse. It does not double-count
        // tool use: the rollup's `tool_uses` filters on PostToolUse and
        // PostToolUseFailure only.
        ev(
            "pre tool use",
            "PreToolUse",
            r#""tool_name":"Bash","tool_input":{"command":"ls"},"tool_use_id":"tu-1""#,
            Some("PreToolUse"),
        ),
        ev(
            "post tool use",
            "PostToolUse",
            r#""tool_name":"Read","tool_input":{"file_path":"/tmp/x.rs"},"tool_response":{"ok":true},"tool_use_id":"tu-2""#,
            Some("PostToolUse"),
        ),
        ev(
            "post tool use failure",
            "PostToolUseFailure",
            r#""tool_name":"Bash","tool_input":{"command":"false"},"tool_use_id":"tu-3","error":"exit status 1","is_interrupt":false"#,
            Some("PostToolUseFailure"),
        ),
        ev(
            "permission request",
            "PermissionRequest",
            r#""tool_name":"Write","tool_input":{"file_path":"/etc/hosts"},"permission_suggestions":[{"mode":"allow"}]"#,
            Some("PermissionRequest"),
        ),
        ev(
            "stop",
            "Stop",
            r#""stop_hook_active":false,"last_assistant_message":"Done.""#,
            Some("Stop"),
        ),
        ev(
            "subagent start",
            "SubagentStart",
            r#""agent_id":"agent-1","agent_type":"Explore""#,
            Some("SubagentStart"),
        ),
        ev(
            "subagent stop",
            "SubagentStop",
            r#""agent_id":"agent-1","agent_type":"Explore","stop_hook_active":false,"agent_transcript_path":"/tmp/a.jsonl","last_assistant_message":"Found it.""#,
            Some("SubagentStop"),
        ),
        ev(
            "task completed",
            "TaskCompleted",
            r#""task_id":"task-1","task_subject":"Ship the contract suite","teammate_name":"claude","team_name":"contract""#,
            Some("TaskCompleted"),
        ),
        ev(
            "teammate idle",
            "TeammateIdle",
            r#""teammate_name":"claude","team_name":"contract""#,
            Some("TeammateIdle"),
        ),
        ev(
            "notification",
            "Notification",
            r#""message":"Permission needed","title":"Claude Code","notification_type":"permission""#,
            Some("Notification"),
        ),
        ev(
            "config change",
            "ConfigChange",
            r#""source":"settings","file_path":"/tmp/settings.json""#,
            Some("ConfigChange"),
        ),
        ev(
            "worktree create",
            "WorktreeCreate",
            r#""name":"feature-x""#,
            Some("WorktreeCreate"),
        ),
        ev(
            "worktree remove",
            "WorktreeRemove",
            r#""worktree_path":"/tmp/wt/feature-x""#,
            Some("WorktreeRemove"),
        ),
        ev(
            "pre compact",
            "PreCompact",
            r#""trigger":"auto","custom_instructions":"keep the plan""#,
            Some("PreCompact"),
        ),
        ev(
            "instructions loaded",
            "InstructionsLoaded",
            r#""file_path":"/tmp/CLAUDE.md","memory_type":"project","load_reason":"startup","globs":["**/*.rs"],"trigger_file_path":null,"parent_file_path":null"#,
            Some("InstructionsLoaded"),
        ),
        // Unsupported event names are rejected at the ingestion boundary.
        ev(
            "unrecognised event name",
            "SomeFutureEvent",
            r#""whatever":true"#,
            None,
        ),
        // Malformed recognized events must not become successful audit entries.
        EventCase {
            label: "recognised name with a malformed body",
            body: format!(
                "{{{},\"stop_hook_active\":\"not-a-bool\"}}",
                common(session, "Stop")
            ),
            recorded_as: None,
        },
    ]
}

async fn count_events(pool: &sqlx::PgPool, session: &str, event_type: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM plugin_usage_events WHERE session_id = $1 AND event_type = $2",
    )
    .bind(session)
    .bind(event_type)
    .fetch_one(pool)
    .await
    .expect("count hook events")
}

#[tokio::test(flavor = "multi_thread")]
async fn hook_track_validates_and_records_supported_event_kinds() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping hook-track suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let user_id = seed::unique("hook-user");
    seed::insert_user(&db.pool, &user_id, &format!("{user_id}@contract.test")).await;
    let token = seed::mint(&TokenSpec::hook(&user_id));
    let session = seed::unique("hook-session");

    let mut failures = Vec::new();
    for case in cases(&session) {
        let (call, tok) = hook_call(&token, &case.body);
        let (status, body) = app.call_with_bearer(call, tok).await;
        let expected = if case.recorded_as.is_some() {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };
        if status != expected {
            failures.push(format!(
                "  {} -> {} (unexpected status): {}",
                case.label,
                status.as_u16(),
                body.chars().take(200).collect::<String>()
            ));
            continue;
        }
        let Some(event_type) = case.recorded_as else {
            continue;
        };
        if count_events(&db.pool, &session, event_type).await == 0 {
            failures.push(format!(
                "  {} -> 200 but no plugin_usage_events row with event_type {event_type:?}",
                case.label
            ));
        }
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} hook event(s) were not ingested:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// The entity detector: which skill, agent, or MCP server an event belongs to.
//
// Detection runs off the tool name and the tool input, and the result lands in
// `session_entity_links`. Asserting on that table rather than the response is
// the only way to tell a detection that fired from one that returned `None` —
// both answer `200`.
#[tokio::test(flavor = "multi_thread")]
async fn hook_track_links_events_to_the_entity_they_name() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let user_id = seed::unique("entity-user");
    seed::insert_user(&db.pool, &user_id, &format!("{user_id}@contract.test")).await;
    let token = seed::mint(&TokenSpec::hook(&user_id));

    // (label, tool payload, expected entity_type, expected entity_name)
    let expectations: [(&str, String, &str, &str); 6] = [
        (
            "skill invocation",
            r#""tool_name":"Skill","tool_input":{"skill":"development:rust-dev-guide"},"tool_response":{}"#.to_owned(),
            "skill",
            "development:rust-dev-guide",
        ),
        (
            "mcp tool",
            r#""tool_name":"mcp__systemprompt__list_skills","tool_input":{},"tool_response":{}"#.to_owned(),
            "mcp_tool",
            "systemprompt",
        ),
        (
            "agent by subagent_type",
            r#""tool_name":"Agent","tool_input":{"subagent_type":"Explore"},"tool_response":{}"#.to_owned(),
            "agent",
            "Explore",
        ),
        (
            "agent falling back to description",
            r#""tool_name":"Agent","tool_input":{"description":"sweep the handlers"},"tool_response":{}"#.to_owned(),
            "agent",
            "sweep the handlers",
        ),
        (
            "agent with neither hint",
            r#""tool_name":"Agent","tool_input":{},"tool_response":{}"#.to_owned(),
            "agent",
            "subagent",
        ),
        (
            "a plain tool links nothing",
            r#""tool_name":"Bash","tool_input":{"command":"ls"},"tool_response":{}"#.to_owned(),
            "",
            "",
        ),
    ];

    let mut failures = Vec::new();
    for (label, payload, entity_type, entity_name) in expectations {
        let session = seed::unique("entity-session");
        let body = format!(
            "{{{},\"tool_use_id\":\"{session}-tool\",{payload}}}",
            common(&session, "PostToolUse")
        );
        let (call, tok) = hook_call(&token, &body);
        let (status, _) = app.call_with_bearer(call, tok).await;
        assert_eq!(status, StatusCode::OK, "{label}: hook track rejected");

        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT entity_type, entity_name FROM session_entity_links WHERE session_id = $1",
        )
        .bind(&session)
        .fetch_all(&*db.pool)
        .await
        .expect("read session entity links");

        if entity_type.is_empty() {
            if !rows.is_empty() {
                failures.push(format!("  {label} -> linked {rows:?}, expected nothing"));
            }
            continue;
        }
        if !rows
            .iter()
            .any(|(t, n)| t == entity_type && n == entity_name)
        {
            failures.push(format!(
                "  {label} -> links {rows:?}, expected ({entity_type}, {entity_name})"
            ));
        }
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} entity detection(s) went wrong:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// The dedup key, the session rollup, and the derived title.
//
// These are the three side effects a caller can observe without an AI service
// configured, and each is a branch the status contract never reaches.
#[tokio::test(flavor = "multi_thread")]
async fn hook_track_deduplicates_and_rolls_up_the_session() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let user_id = seed::unique("dedup-user");
    seed::insert_user(&db.pool, &user_id, &format!("{user_id}@contract.test")).await;
    let token = seed::mint(&TokenSpec::hook(&user_id));
    let session = seed::unique("dedup-session");

    let prompt = format!(
        r#"{{{},"prompt":"Rebuild the governance audit page from the trace spine."}}"#,
        common(&session, "UserPromptSubmit")
    );

    // The same event posted twice is one row: a hook that retries on a slow
    // response must not double-count.
    for _ in 0..2 {
        let (call, tok) = hook_call(&token, &prompt);
        let (status, _) = app.call_with_bearer(call, tok).await;
        assert_eq!(status, StatusCode::OK);
    }
    assert_eq!(
        count_events(&db.pool, &session, "UserPromptSubmit").await,
        1,
        "an identical repost must deduplicate rather than insert a second row"
    );

    let mut changed: serde_json::Value = serde_json::from_str(&prompt).expect("fixture JSON");
    changed["prompt"] = "Different content with the same delivery ID".into();
    let changed_body = changed.to_string();
    let (call, tok) = hook_call(&token, &changed_body);
    assert_eq!(
        app.call_with_bearer(call, tok).await.0,
        StatusCode::CONFLICT
    );
    let mut repeated: serde_json::Value = serde_json::from_str(&prompt).expect("fixture JSON");
    repeated["prompt_id"] = seed::unique("new-prompt").into();
    let repeated_body = repeated.to_string();
    let (call, tok) = hook_call(&token, &repeated_body);
    assert_eq!(app.call_with_bearer(call, tok).await.0, StatusCode::OK);
    assert_eq!(
        count_events(&db.pool, &session, "UserPromptSubmit").await,
        2,
        "a new identified prompt must survive even when its text is identical"
    );

    // The first prompt seeds the session title, so the session page has
    // something to show before any AI summary exists.
    let title: Option<String> =
        sqlx::query_scalar("SELECT ai_title FROM plugin_session_summaries WHERE session_id = $1")
            .bind(&session)
            .fetch_optional(&*db.pool)
            .await
            .expect("read the session summary")
            .flatten();
    assert!(
        title.is_some_and(|t| !t.is_empty()),
        "the first UserPromptSubmit must derive a session title"
    );

    // A daily aggregation row is what the usage dashboards read; without it the
    // event is recorded but invisible.
    let daily: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM plugin_usage_daily WHERE user_id = $1")
            .bind(&user_id)
            .fetch_one(&*db.pool)
            .await
            .expect("count daily aggregations");
    assert!(daily > 0, "ingestion must upsert the daily usage rollup");

    // `Stop` runs the APM calculation and the concurrent-session count, and
    // `SessionEnd` closes the summary — the two paths gated on event type.
    for event in ["Stop", "SessionEnd"] {
        let body = format!(
            r#"{{{},"stop_hook_active":false,"reason":"clear"}}"#,
            common(&session, event)
        );
        let (call, tok) = hook_call(&token, &body);
        let (status, _) = app.call_with_bearer(call, tok).await;
        assert_eq!(status, StatusCode::OK, "{event} was rejected");
    }
    let ended: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT ended_at FROM plugin_session_summaries WHERE session_id = $1")
            .bind(&session)
            .fetch_optional(&*db.pool)
            .await
            .expect("read the session summary")
            .flatten();
    assert!(ended.is_some(), "SessionEnd must close the session summary");

    db.cleanup().await;
}

// The token gate. Each case is wrong in exactly one way, so a `401` names the
// check that caught it rather than "some token problem".
#[tokio::test(flavor = "multi_thread")]
async fn hook_track_refuses_every_token_that_is_not_a_hook_token() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let subject = seed::unique("reject-user");
    let body = format!(
        r#"{{{},"source":"startup","model":"m"}}"#,
        common("reject-session", "SessionStart")
    );

    let wrong_audience = seed::mint(&TokenSpec {
        subject: &subject,
        audiences: vec![JwtAudience::Api],
        scopes: vec![Permission::HookTrack],
        plugin_id: Some("contract-plugin"),
    });
    let wrong_scope = seed::mint(&TokenSpec {
        subject: &subject,
        audiences: vec![JwtAudience::Hook],
        // `hook:govern` is the *other* hook endpoint's scope; a token minted
        // for the governance gate must not be able to write tracking rows.
        scopes: vec![Permission::HookGovern],
        plugin_id: Some("contract-plugin"),
    });
    let no_plugin = seed::mint(&TokenSpec {
        subject: &subject,
        audiences: vec![JwtAudience::Hook],
        scopes: vec![Permission::HookTrack],
        plugin_id: None,
    });

    let rejected: [(&str, Option<&str>); 5] = [
        ("no authorization header", None),
        ("a token that is not a JWT", Some("not-a-jwt")),
        ("aud=api instead of aud=hook", Some(&wrong_audience)),
        ("scope hook:govern, not hook:track", Some(&wrong_scope)),
        ("no plugin_id claim", Some(&no_plugin)),
    ];

    let mut failures = Vec::new();
    for (label, token) in rejected {
        let call = Call {
            method: "post",
            path: TRACK,
            principal: Principal::Anonymous,
            content_type: Some("application/json"),
            body: Some(&body),
        };
        let (status, _) = match token {
            Some(t) => app.call_with_bearer(call, t).await,
            None => app.call(call).await,
        };
        if status != StatusCode::UNAUTHORIZED {
            failures.push(format!("  {label} -> {} (expected 401)", status.as_u16()));
        }
    }

    // A body that is not JSON at all is refused by the extractor, before the
    // token is ever read.
    let (status, _) = app
        .call(Call {
            method: "post",
            path: TRACK,
            principal: Principal::Anonymous,
            content_type: Some("application/json"),
            body: Some("{not json"),
        })
        .await;
    if !status.is_client_error() {
        failures.push(format!(
            "  a malformed JSON body -> {} (expected a 4xx)",
            status.as_u16()
        ));
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} hook-track rejection(s) did not hold:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
