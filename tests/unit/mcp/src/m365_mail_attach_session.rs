//! `d365_upload_attachment_chunk` takes only an `uploadUrl` that
//! `d365_create_mail_attachment_upload_session` returned to the same user on
//! this server, before it expires. A session opened in another tenant, one
//! opened for another user or one past its expiry is refused before the file
//! is opened or Graph gets a request, and so is a chunk over Graph's 4 MiB
//! limit, as a `file` or as base64. Microsoft's upload hosts stay a second
//! barrier behind the binding, and every call's Graph client shares one
//! bounded registry through `GraphHost`.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{TimeDelta, Utc};
use serde_json::{Value, json};
use systemprompt::identifiers::{McpExecutionId, UserId};
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_microsoft_365::GraphHost;
use systemprompt_mcp_microsoft_365::graph::upload::MAX_CHUNK_BYTES;
use systemprompt_mcp_microsoft_365::graph::upload_sessions::{
    MAX_UPLOAD_SESSIONS, MAX_UPLOAD_SESSIONS_PER_USER, UPLOAD_SESSION_MAX_MINUTES, UploadSessions,
};
use systemprompt_mcp_microsoft_365::graph::{
    BROKER_PATH, Broker, GraphClient, GraphLimits, http_client,
};
use systemprompt_mcp_microsoft_365::tools::mail_upload::{
    create_upload_session, upload_attachment_chunk,
};
use systemprompt_mcp_shared::files::FileCall;

use crate::files_support::{
    ALICE, BOB, Harness, anonymous, document_of, exec_id, harness, signed_in,
};
use crate::m365_support::{FakeTokens, Reply, Secrets, Seen, Sim, client, fast_limits};

const TOTAL: u64 = 9_000_000;
const ATTACKER_SESSION: &str = "https://outlook.office.com/api/v2.0/Users('mallory@attacker.example')/Messages('m1')/AttachmentSessions('s1')?authtoken=attacker";

fn parse<T: serde::de::DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

fn as_caller<'a>(ctx: &'a RequestContext, exec: &'a McpExecutionId) -> FileCall<'a> {
    FileCall {
        ctx,
        tool: "d365_mail_test",
        exec_id: exec,
        hop: None,
    }
}

// A Graph whose createUploadSession opens a session on its own origin,
// expiring at `expires` when given, and whose chunk PUT answers 200.
fn session_graph(
    expires: Option<String>,
) -> impl Fn(&Seen, usize) -> Reply + Send + Sync + 'static {
    move |seen, _| {
        if seen.method != "POST" {
            return Reply::json(200, &json!({"nextExpectedRanges": ["more"]}));
        }
        let host = seen.header("host").unwrap_or_default();
        let mut session = json!({"uploadUrl": format!("http://{host}/upload/s1?authtoken=sas")});
        if let Some(at) = &expires {
            session["expirationDateTime"] = json!(at);
        }
        Reply::json(201, &session)
    }
}

async fn open_session(graph: &GraphClient, h: &Harness, ctx: &RequestContext) -> String {
    let exec = exec_id();
    let input = parse(json!({"message_id": "d1", "filename": "big.bin", "size_bytes": TOTAL}));
    let reply = create_upload_session(graph, &h.files, as_caller(ctx, &exec), input)
        .await
        .unwrap();
    serde_json::to_value(&reply).unwrap()["data"]["uploadUrl"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn stored(h: &Harness, owner: &str, bytes: &[u8]) -> Value {
    let (record, document) = document_of(owner, "part.bin", "application/zip", bytes.len() as u64);
    h.store.put(record, bytes);
    json!({"document_id": document.to_string()})
}

async fn send_chunk(
    graph: &GraphClient,
    h: &Harness,
    ctx: &RequestContext,
    url: &str,
    file: Value,
) -> Result<Value, rmcp::ErrorData> {
    let exec = exec_id();
    let input = parse(json!({
        "upload_url": url, "byte_range_start": 0, "total_size": TOTAL, "file": file
    }));
    upload_attachment_chunk(graph, &h.files, as_caller(ctx, &exec), input)
        .await
        .map(|reply| serde_json::to_value(&reply).unwrap()["data"].clone())
}

fn puts(sim: &Sim) -> usize {
    sim.calls()
        .iter()
        .filter(|seen| seen.method == "PUT")
        .count()
}

#[tokio::test]
async fn m365_mail_attach_session_takes_a_session_opened_for_the_same_user() {
    let sim = Sim::start(session_graph(None)).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let (h, alice) = (harness(), signed_in(ALICE));
    let url = open_session(&graph, &h, &alice).await;
    let file = stored(&h, ALICE, b"abcd");
    let data = send_chunk(&graph, &h, &alice, &url, file).await.unwrap();

    let calls = sim.calls();
    assert_eq!(calls.len(), 2, "{calls:#?}");
    assert_eq!(
        (calls[1].method.as_str(), calls[1].path.as_str()),
        ("PUT", "/upload/s1")
    );
    assert_eq!(
        calls[1].header("content-range"),
        Some(format!("bytes 0-3/{TOTAL}").as_str())
    );
    assert_eq!(calls[1].header("authorization"), None);
    assert_eq!(calls[1].body, b"abcd");
    assert_eq!(data["status_code"], 200);
}

#[tokio::test]
async fn m365_mail_attach_session_refuses_a_session_of_another_tenant_before_reading_the_file() {
    let sim = Sim::start(session_graph(None)).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let (h, alice) = (harness(), signed_in(ALICE));
    let file = stored(&h, ALICE, b"payroll");
    let never_opened = format!("{}/upload/s1?authtoken=sas", sim.base);
    for url in [
        ATTACKER_SESSION,
        "https://attacker.sharepoint.com/sites/x/_api/upload?tempauth=attacker",
        never_opened.as_str(),
    ] {
        let error = send_chunk(&graph, &h, &alice, url, file.clone())
            .await
            .unwrap_err();
        assert!(
            error
                .message
                .contains("d365_create_mail_attachment_upload_session"),
            "{url}: {error:?}"
        );
    }

    assert!(sim.calls().is_empty(), "Graph never got a request");
    assert_eq!(
        h.store.with(|state| state.opened),
        0,
        "the file was not read"
    );
    assert!(
        h.audit.events().is_empty(),
        "the file layer was not reached"
    );
}

#[tokio::test]
async fn m365_mail_attach_session_refuses_a_session_opened_for_another_user() {
    let sim = Sim::start(session_graph(None)).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let h = harness();
    let (alice, bob) = (signed_in(ALICE), signed_in(BOB));
    let url = open_session(&graph, &h, &alice).await;
    let bobs = stored(&h, BOB, b"bob's bytes");
    let error = send_chunk(&graph, &h, &bob, &url, bobs).await.unwrap_err();

    assert!(
        error
            .message
            .contains("d365_create_mail_attachment_upload_session"),
        "{error:?}"
    );
    assert_eq!(puts(&sim), 0);
    assert_eq!(h.store.with(|state| state.opened), 0);

    let alices = stored(&h, ALICE, b"abcd");
    send_chunk(&graph, &h, &alice, &url, alices).await.unwrap();
    assert_eq!(puts(&sim), 1, "the session is still Alice's");
}

#[tokio::test]
async fn m365_mail_attach_session_refuses_a_session_past_its_expiry() {
    let expired = (Utc::now() - TimeDelta::minutes(1)).to_rfc3339();
    let sim = Sim::start(session_graph(Some(expired))).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let (h, alice) = (harness(), signed_in(ALICE));
    let url = open_session(&graph, &h, &alice).await;
    let file = stored(&h, ALICE, b"abcd");
    let error = send_chunk(&graph, &h, &alice, &url, file)
        .await
        .unwrap_err();

    assert!(error.message.contains("expired"), "{error:?}");
    assert!(
        error
            .message
            .contains("d365_create_mail_attachment_upload_session"),
        "the model is told to open a new session: {error:?}"
    );
    assert_eq!(puts(&sim), 0);
    assert_eq!(h.store.with(|state| state.opened), 0);
}

#[tokio::test]
async fn m365_mail_attach_session_refuses_a_file_over_four_mib_before_graph() {
    let sim = Sim::start(session_graph(None)).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let (h, alice) = (harness(), signed_in(ALICE));
    let url = open_session(&graph, &h, &alice).await;
    let file = stored(&h, ALICE, &vec![3u8; MAX_CHUNK_BYTES + 1]);
    let error = send_chunk(&graph, &h, &alice, &url, file)
        .await
        .unwrap_err();

    assert!(error.message.contains("4 MiB"), "{error:?}");
    assert_eq!(puts(&sim), 0, "Graph never got the chunk");
}

#[tokio::test]
async fn m365_mail_attach_session_takes_a_file_of_exactly_four_mib() {
    let sim = Sim::start(session_graph(None)).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let (h, alice) = (harness(), signed_in(ALICE));
    let url = open_session(&graph, &h, &alice).await;
    let file = stored(&h, ALICE, &vec![3u8; MAX_CHUNK_BYTES]);
    send_chunk(&graph, &h, &alice, &url, file).await.unwrap();

    let calls = sim.calls();
    assert_eq!(calls[1].body.len(), MAX_CHUNK_BYTES);
    assert_eq!(
        calls[1].header("content-range"),
        Some(format!("bytes 0-{}/{TOTAL}", MAX_CHUNK_BYTES - 1).as_str())
    );
}

#[tokio::test]
async fn m365_mail_attach_session_refuses_base64_over_four_mib_before_decoding_it() {
    let sim = Sim::start(session_graph(None)).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let (h, alice) = (harness(), signed_in(ALICE));
    let url = open_session(&graph, &h, &alice).await;
    let exec = exec_id();
    let input = parse(json!({
        "upload_url": url, "byte_range_start": 0, "total_size": TOTAL,
        "chunk_base64": STANDARD.encode(vec![3u8; MAX_CHUNK_BYTES + 1])
    }));
    let error = upload_attachment_chunk(&graph, &h.files, as_caller(&alice, &exec), input)
        .await
        .unwrap_err();

    assert!(error.message.contains("4 MiB"), "{error:?}");
    assert_eq!(puts(&sim), 0);
    assert!(
        h.audit.events().is_empty(),
        "refused before the file layer decoded it"
    );
}

#[tokio::test]
async fn m365_mail_attach_session_an_anonymous_caller_opens_and_uses_no_session() {
    let sim = Sim::start(session_graph(None)).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let (h, nobody, exec) = (harness(), anonymous(), exec_id());
    let input = parse(json!({"message_id": "d1", "filename": "big.bin", "size_bytes": TOTAL}));
    let opened = create_upload_session(&graph, &h.files, as_caller(&nobody, &exec), input).await;
    let used = send_chunk(&graph, &h, &nobody, ATTACKER_SESSION, json!({})).await;

    assert!(opened.is_err());
    assert!(used.is_err());
    assert!(sim.calls().is_empty());
    assert!(graph.upload_sessions().is_empty());
}

#[tokio::test]
async fn m365_mail_attach_session_an_issued_url_off_microsoft_hosts_is_still_refused() {
    let off_host = [
        "https://evil.example.com/upload?authtoken=sas",
        "http://outlook.office.com/api/v2.0/sessions/1",
        "https://outlook.office.com.evil.example/api",
        "https://user:pw@outlook.office.com/api",
        "https://evilsharepoint.com/upload",
    ];
    let sim = Sim::start(move |_, index| {
        Reply::json(201, &json!({"uploadUrl": off_host[index % off_host.len()]}))
    })
    .await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let (h, alice) = (harness(), signed_in(ALICE));
    for expected in off_host {
        let url = open_session(&graph, &h, &alice).await;
        assert_eq!(url, expected);
        let exec = exec_id();
        let input = parse(json!({
            "upload_url": url, "chunk_base64": STANDARD.encode(b"abcd"),
            "byte_range_start": 0, "total_size": 4
        }));
        let result =
            upload_attachment_chunk(&graph, &h.files, as_caller(&alice, &exec), input).await;
        assert!(result.is_err(), "{url}");
    }
    assert_eq!(puts(&sim), 0);
}

#[test]
fn m365_mail_attach_session_expiry_is_graphs_capped_at_the_servers_own() {
    let sessions = UploadSessions::default();
    let alice = UserId::new(ALICE);
    let now = Utc::now();
    let cap = TimeDelta::minutes(UPLOAD_SESSION_MAX_MINUTES);
    let second = TimeDelta::seconds(1);
    let long = "https://outlook.office.com/api/v2.0/sessions/long?authtoken=a";
    let short = "https://outlook.office.com/api/v2.0/sessions/short?authtoken=b";
    let undated = "https://outlook.office.com/api/v2.0/sessions/undated?authtoken=c";
    let in_a_day = (now + TimeDelta::days(1)).to_rfc3339();
    // Graph writes seven fractional digits and a `Z`.
    let in_ten_minutes = (now + TimeDelta::minutes(10))
        .format("%Y-%m-%dT%H:%M:%S%.6f0Z")
        .to_string();
    sessions.issue(long, &alice, Some(&in_a_day), now);
    sessions.issue(short, &alice, Some(&in_ten_minutes), now);
    sessions.issue(undated, &alice, Some("not a date"), now);

    assert!(
        sessions.admit(long, &UserId::new(BOB), now).is_err(),
        "a live session is only its owner's"
    );
    assert!(sessions.admit(long, &alice, now + cap - second).is_ok());
    assert!(sessions.admit(long, &alice, now + cap).is_err());
    assert!(
        sessions
            .admit(short, &alice, now + TimeDelta::minutes(9))
            .is_ok()
    );
    assert!(
        sessions
            .admit(short, &alice, now + TimeDelta::minutes(11))
            .is_err()
    );
    let spelled_otherwise = "  https://OUTLOOK.office.com/api/v2.0/sessions/undated?authtoken=c ";
    assert!(
        sessions
            .admit(spelled_otherwise, &alice, now + cap - second)
            .is_ok()
    );
    assert!(sessions.admit(undated, &alice, now + cap + second).is_err());
}

#[test]
fn m365_mail_attach_session_the_registry_is_bounded_per_user_and_overall() {
    let sessions = UploadSessions::default();
    let alice = UserId::new(ALICE);
    let now = Utc::now();
    let url =
        |owner: &str, n: usize| format!("https://outlook.office.com/api/{owner}/sessions/{n}");
    for n in 0..=MAX_UPLOAD_SESSIONS_PER_USER {
        let expires = (now + TimeDelta::minutes(n as i64 + 1)).to_rfc3339();
        sessions.issue(&url("alice", n), &alice, Some(&expires), now);
    }
    assert_eq!(sessions.len(), MAX_UPLOAD_SESSIONS_PER_USER);
    assert!(
        sessions.admit(&url("alice", 0), &alice, now).is_err(),
        "Alice's session closest to expiry made room for her newest"
    );
    let newest = url("alice", MAX_UPLOAD_SESSIONS_PER_USER);
    assert!(sessions.admit(&newest, &alice, now).is_ok());

    for n in 0..MAX_UPLOAD_SESSIONS {
        let user = UserId::new(format!("user-{n}"));
        sessions.issue(&url("someone", n), &user, None, now);
    }
    assert_eq!(sessions.len(), MAX_UPLOAD_SESSIONS);

    let later = now + TimeDelta::minutes(UPLOAD_SESSION_MAX_MINUTES + 1);
    sessions.issue(&url("alice", 999), &alice, None, later);
    assert_eq!(sessions.len(), 1, "expired sessions are purged");
}

#[tokio::test]
async fn m365_mail_attach_session_every_call_of_the_server_shares_one_registry() {
    let limits = GraphLimits::default();
    let host = GraphHost {
        http: http_client(&limits).unwrap(),
        broker: Broker::new(
            format!("http://127.0.0.1:9{BROKER_PATH}"),
            Secrets::with(&[]),
            "microsoft-365",
        ),
        secrets: Secrets::with(&[]),
        limits,
        upload_sessions: UploadSessions::default(),
    };
    let cloned = host.clone();
    let first_call = host.client_for("alice-jwt".to_owned(), None).unwrap();
    let other_session = cloned.client_for("alice-jwt".to_owned(), None).unwrap();
    let (alice, now) = (UserId::new(ALICE), Utc::now());
    first_call
        .upload_sessions()
        .issue(ATTACKER_SESSION, &alice, None, now);

    assert!(
        other_session
            .upload_sessions()
            .admit(ATTACKER_SESSION, &alice, now)
            .is_ok()
    );
    assert_eq!(host.upload_sessions.len(), 1);
    assert_eq!(cloned.upload_sessions.len(), 1);
}
