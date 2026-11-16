//! The online meeting tools against a simulated Graph and an in-memory file
//! store: lookups by join URL (quoted, with eventual consistency, first match
//! or a null), detail reads with the Belgian `$select`, transcripts,
//! recordings and attendance paged with `skip`, a transcript's `WebVTT` kept as
//! the caller's document instead of returned inline, meeting creation with
//! attendees only, and the delete reply.

use serde_json::{Value, json};
use systemprompt::identifiers::McpExecutionId;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_microsoft_365::tools::meetings::{
    create_online_meeting, delete_online_meeting, get_online_meeting, meeting_by_join_url,
};
use systemprompt_mcp_microsoft_365::tools::meetings_media::{
    get_meeting_recording, get_meeting_transcript_content, list_meeting_attendance_reports,
    list_meeting_recordings, list_meeting_transcripts,
};
use systemprompt_mcp_shared::files::{FileAction, FileCall};

use crate::files_support::{ALICE, exec_id, harness, signed_in};
use crate::m365_pim_support::{args, assert_domain_registered, empty, graph, is_invalid};
use crate::m365_support::{Reply, Sim, collection};

const VTT: &[u8] = b"WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Ana>Hola equipo</v>\n";

fn as_caller<'a>(ctx: &'a RequestContext, exec: &'a McpExecutionId) -> FileCall<'a> {
    FileCall {
        ctx,
        tool: "d365_get_meeting_transcript_content",
        exec_id: exec,
        hop: None,
    }
}

async fn echo() -> Sim {
    Sim::start(|seen, _| match seen.method.as_str() {
        "DELETE" => empty(204),
        "POST" => Reply::json(201, &json!({"id": "mt-new", "joinWebUrl": "https://x"})),
        _ if seen.path.ends_with("/R1") || seen.path.ends_with("/MT1") => {
            Reply::json(200, &json!({"id": "one"}))
        },
        _ => Reply::json(200, &collection(&[json!({"id": "x"})], None)),
    })
    .await
}

#[test]
fn m365_meetings_all_ten_belgian_tools_are_registered_as_pinned() {
    assert_domain_registered("meetings", 10);
}

#[tokio::test]
async fn m365_meetings_join_url_lookup_quotes_the_url_and_answers_the_first_match() {
    let meetings = vec![json!({"id": "mt1"}), json!({"id": "mt2"})];
    let sim = Sim::start(move |_, _| Reply::json(200, &collection(&meetings, None))).await;
    let url = "https://teams.microsoft.com/l/meetup-join/19%3ameeting_x/0?context=O'Neil";
    let reply = meeting_by_join_url(&graph(&sim), args(json!({"join_url": url})))
        .await
        .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.method, "GET");
    assert_eq!(call.path, "/v1.0/me/onlineMeetings");
    assert_eq!(
        call.q("$filter").unwrap(),
        format!("JoinWebUrl eq '{}'", url.replace('\'', "''"))
    );
    assert_eq!(call.header("consistencylevel"), Some("eventual"));
    assert_eq!((call.q("$top"), call.q("$select")), (None, None));
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": {"id": "mt1"}, "message": "success"})
    );
}

#[tokio::test]
async fn m365_meetings_join_url_without_a_match_answers_a_null_with_a_message() {
    let sim = Sim::start(|_, _| Reply::json(200, &collection(&[], None))).await;
    let reply = meeting_by_join_url(&graph(&sim), args(json!({"join_url": "https://x"})))
        .await
        .unwrap();

    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": null, "message": "No meeting found for the given join URL"})
    );
}

#[tokio::test]
async fn m365_meetings_meeting_and_recording_are_read_by_id() {
    let sim = echo().await;
    let client = graph(&sim);
    let meeting = get_online_meeting(&client, args(json!({"meeting_id": "MT1"})))
        .await
        .unwrap();
    let recording = get_meeting_recording(
        &client,
        args(json!({"meeting_id": "MSo1N2Y5ZGFjYy03MWJmLTQ3NDMtYjQxMy01M2EdFGkdRWHJlQ", "recording_id": "R1"})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/onlineMeetings/MT1");
    assert_eq!(
        calls[0].q("$select"),
        Some(
            "id,subject,startDateTime,endDateTime,joinWebUrl,participants,chatInfo,\
             videoTeleconferenceId"
        )
    );
    assert_eq!(
        calls[1].path,
        "/v1.0/me/onlineMeetings/MSo1N2Y5ZGFjYy03MWJmLTQ3NDMtYjQxMy01M2EdFGkdRWHJlQ/recordings/R1"
    );
    assert!(calls[1].q("$select").is_none());
    assert_eq!(meeting.data["id"], "one");
    assert_eq!(recording.data["id"], "one");
}

#[tokio::test]
async fn m365_meetings_transcripts_recordings_and_attendance_page_with_skip() {
    let sim = echo().await;
    let client = graph(&sim);
    let transcripts = list_meeting_transcripts(&client, args(json!({"meeting_id": "MT1"})))
        .await
        .unwrap();
    list_meeting_recordings(&client, args(json!({"meeting_id": "MT1", "skip": 3})))
        .await
        .unwrap();
    list_meeting_attendance_reports(&client, args(json!({"meeting_id": "MT1", "skip": 1})))
        .await
        .unwrap();

    let calls = sim.calls();
    let paths: Vec<&str> = calls.iter().map(|c| c.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "/v1.0/me/onlineMeetings/MT1/transcripts",
            "/v1.0/me/onlineMeetings/MT1/recordings",
            "/v1.0/me/onlineMeetings/MT1/attendanceReports",
        ]
    );
    let skips: Vec<Option<&str>> = calls.iter().map(|c| c.q("$skip")).collect();
    assert_eq!(skips, [None, Some("3"), Some("1")]);
    assert!(calls.iter().all(|c| c.q("$top").is_none()));
    assert_eq!(transcripts.data, vec![json!({"id": "x"})]);
}

#[tokio::test]
async fn m365_meetings_transcript_content_is_kept_as_a_document_not_inlined() {
    let sim = Sim::start(|_, _| Reply {
        status: 200,
        headers: vec![("content-type", "text/vtt; charset=utf-8".to_owned())],
        body: VTT.to_vec(),
    })
    .await;
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let transcript = "MSMjMCMjZDAwYWU3NjUtNmM2Yi00NjQxLTgwMWQtMTkzMmFmMjEzNzdh";
    let input = args(json!({"meeting_id": "MT1", "transcript_id": transcript}));
    let reply =
        get_meeting_transcript_content(&graph(&sim), &h.files, as_caller(&ctx, &exec), input)
            .await
            .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(
        call.path,
        format!("/v1.0/me/onlineMeetings/MT1/transcripts/{transcript}/content")
    );
    assert_eq!(call.header("accept"), Some("text/vtt"));
    let data = serde_json::to_value(&reply).unwrap()["data"].clone();
    assert_eq!(data["meeting_id"], "MT1");
    assert_eq!(data["transcript_id"], transcript);
    assert_eq!(data["file"]["name"], "transcript-MmFmMjEzNzdh.vtt");
    assert_eq!(data["file"]["mime"], "text/vtt");
    assert_eq!(data["file"]["size_bytes"], VTT.len());
    assert!(!data.to_string().contains("Hola equipo"), "{data}");
    let kept: Vec<Value> = h.store.with(|state| {
        state
            .records
            .values()
            .map(|(record, bytes)| json!({"name": record.name, "bytes": bytes}))
            .collect()
    });
    assert_eq!(
        kept,
        vec![json!({"name": "transcript-MmFmMjEzNzdh.vtt", "bytes": VTT})]
    );
    assert!(
        h.audit
            .events()
            .iter()
            .any(|event| matches!(event.action, FileAction::Stored))
    );
}

#[tokio::test]
async fn m365_meetings_create_lists_participants_as_attendees_only() {
    let sim = echo().await;
    let client = graph(&sim);
    let window = json!({
        "subject": "Kick-off",
        "start_datetime": "2026-11-20T09:00:00Z",
        "end_datetime": "2026-11-20T10:00:00Z"
    });
    let mut with_people = window.clone();
    with_people["participants"] = json!(["ana@x.test", "luis@x.test"]);
    let bare = create_online_meeting(&client, args(window)).await.unwrap();
    create_online_meeting(&client, args(with_people))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls.len(), 2, "no /me lookup: {calls:#?}");
    assert_eq!(calls[0].method, "POST");
    assert_eq!(calls[0].path, "/v1.0/me/onlineMeetings");
    assert_eq!(
        calls[0].json(),
        json!({
            "subject": "Kick-off",
            "startDateTime": "2026-11-20T09:00:00Z",
            "endDateTime": "2026-11-20T10:00:00Z"
        })
    );
    assert_eq!(
        calls[1].json()["participants"],
        json!({"attendees": [
            {"upn": "ana@x.test", "role": "attendee"},
            {"upn": "luis@x.test", "role": "attendee"}
        ]})
    );
    assert_eq!(bare.data["id"], "mt-new");
}

#[tokio::test]
async fn m365_meetings_delete_answers_like_belgium_and_ids_are_checked() {
    let sim = echo().await;
    let client = graph(&sim);
    let deleted = delete_online_meeting(&client, args(json!({"meeting_id": "MT1"})))
        .await
        .unwrap();
    assert_eq!(sim.calls()[0].method, "DELETE");
    assert_eq!(sim.calls()[0].path, "/v1.0/me/onlineMeetings/MT1");
    assert_eq!(
        serde_json::to_value(&deleted).unwrap(),
        json!({"data": {"deleted": true, "meeting_id": "MT1"}, "message": "success"})
    );

    let errors = [
        get_online_meeting(&client, args(json!({"meeting_id": "../x"})))
            .await
            .unwrap_err(),
        list_meeting_transcripts(&client, args(json!({"meeting_id": ""})))
            .await
            .unwrap_err(),
        get_meeting_recording(
            &client,
            args(json!({"meeting_id": "MT1", "recording_id": "R/1"})),
        )
        .await
        .unwrap_err(),
        delete_online_meeting(&client, args(json!({"meeting_id": "MT 1"})))
            .await
            .unwrap_err(),
    ];
    assert!(errors.iter().all(is_invalid), "{errors:?}");
    assert_eq!(sim.calls().len(), 1);
}

#[tokio::test]
async fn m365_meetings_transcript_ids_are_checked_before_graph_or_the_store() {
    let sim = echo().await;
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let input = args(json!({"meeting_id": "MT1", "transcript_id": "../../me/messages"}));
    let refused =
        get_meeting_transcript_content(&graph(&sim), &h.files, as_caller(&ctx, &exec), input)
            .await
            .unwrap_err();

    assert!(refused.message.contains("transcript_id"), "{refused:?}");
    assert!(sim.calls().is_empty());
    assert!(h.store.with(|state| state.records.is_empty()));
}
