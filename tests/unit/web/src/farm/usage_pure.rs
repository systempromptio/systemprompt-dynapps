//! Pure halves of the Farm console's `usage` module.

use systemprompt_web_admin_agent_farm::sp::{MAX_CALLS, SpCall, SpModelTotal};
use systemprompt_web_admin_agent_farm::usage::{
    CallsRead, ContextsRead, client_session_id, format_latency, format_usd, gateway_session_id,
    sp_task_view,
};

#[test]
fn usd_has_sensible_decimals() {
    assert_eq!(format_usd(0), "$0.00");
    assert_eq!(format_usd(2_374), "$0.0024");
    assert_eq!(format_usd(4_557), "$0.0046");
    assert_eq!(format_usd(10_000), "$0.01");
    assert_eq!(format_usd(752_500), "$0.75");
    assert_eq!(format_usd(6_002_500), "$6.00");
    assert_eq!(format_usd(123_456_789), "$123.46");
}

#[test]
fn latency_reads_for_humans() {
    assert_eq!(format_latency(Some(450)), "450 ms");
    assert_eq!(format_latency(Some(8_200)), "8.2 s");
    assert_eq!(format_latency(None), "—");
    assert_eq!(format_latency(Some(-1)), "—");
}

#[test]
fn session_ids_are_checked_before_any_query() {
    assert_eq!(
        client_session_id(Some("sess_547de7ed-4450-4a6a-8dff-d8cf287f9746")).as_deref(),
        Some("547de7ed-4450-4a6a-8dff-d8cf287f9746")
    );
    assert_eq!(client_session_id(Some("sess_")), None);
    assert_eq!(client_session_id(Some("x' OR 1=1")), None);
    assert_eq!(client_session_id(None), None);
    assert_eq!(
        gateway_session_id(Some("sess_9e954023-f3a5-491f-b050-203823c9390c")).as_deref(),
        Some("sess_9e954023-f3a5-491f-b050-203823c9390c")
    );
    assert_eq!(gateway_session_id(Some("9e954023")), None);
    assert_eq!(gateway_session_id(Some("sess_zz")), None);
}

#[test]
fn a_context_that_is_not_a_uuid_gets_no_link() {
    let view = sp_task_view(
        None,
        CallsRead::NoSession,
        ContextsRead::Read {
            client_session_id: "abc".to_owned(),
            contexts: vec!["not-a-uuid".to_owned()],
        },
    );
    assert!(view.conversations.is_empty());
    assert!(view.conversation_note.is_some());
}

#[test]
fn many_calls_say_the_list_is_cut() {
    let call = SpCall {
        id: "r".to_owned(),
        created_at: chrono::DateTime::default(),
        model: None,
        status: "completed".to_owned(),
        request_kind: "turn".to_owned(),
        input_tokens: None,
        output_tokens: None,
        cache_read_tokens: None,
        cache_creation_tokens: None,
        cost_microdollars: 1,
        latency_ms: None,
    };
    let calls = vec![call; MAX_CALLS as usize];
    let totals = vec![SpModelTotal {
        calls: 900,
        cost_microdollars: 900,
        ..SpModelTotal::default()
    }];
    let view = sp_task_view(
        Some("sess_1"),
        CallsRead::Read { calls, totals },
        ContextsRead::NoSession,
    );
    assert!(view.truncated.is_some());
    assert_eq!(view.total_calls, "900 calls");
    assert_eq!(view.calls[0].model, "—");
    assert_eq!(view.calls[0].status_tone, "ok");
}
