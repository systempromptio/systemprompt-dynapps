//! The single-event calendar tools against a simulated Graph: the method,
//! path and JSON body each sends (UTC times, the all-day midnight form,
//! required attendees, a Teams meeting, a PATCH carrying only what was
//! given, whole-list `categories`), the invitation answers, and the Belgian
//! reply of each.

use serde_json::json;
use systemprompt_mcp_microsoft_365::tools::calendar_event::{
    EVENT_DETAIL_FIELDS, accept_event, create_event, decline_event, delete_event, get_event,
    tentatively_accept_event, update_event,
};

use crate::m365_pim_support::{args, empty, graph, is_invalid};
use crate::m365_support::{Reply, Sim};

async fn echo() -> Sim {
    Sim::start(|seen, _| match seen.method.as_str() {
        "DELETE" => empty(204),
        "POST" if seen.path.ends_with("ccept") || seen.path.ends_with("decline") => empty(202),
        _ => Reply::json(200, &json!({"id": "ev1", "subject": "Sync"})),
    })
    .await
}

#[tokio::test]
async fn m365_calendar_get_event_reads_the_detail_fields() {
    let sim = echo().await;
    let reply = get_event(&graph(&sim), args(json!({"event_id": "AAMkAD="})))
        .await
        .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.method, "GET");
    assert_eq!(call.path, "/v1.0/me/events/AAMkAD=");
    assert_eq!(call.q("$select"), Some(EVENT_DETAIL_FIELDS));
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": {"id": "ev1", "subject": "Sync"}, "message": "success"})
    );
}

#[tokio::test]
async fn m365_calendar_create_event_posts_a_timed_teams_meeting() {
    let sim = echo().await;
    let input = args(json!({
        "subject": "Kick-off",
        "start_datetime": "2024-06-15T09:00:00",
        "end_datetime": "2024-06-15T10:00:00",
        "body": "<p>Agenda</p>",
        "location": "Room 1",
        "attendees": ["ana@contoso.com", "bo@contoso.com"],
        "is_online": true,
        "calendar_id": "cal-2"
    }));
    let reply = create_event(&graph(&sim), input).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.method, "POST");
    assert_eq!(call.path, "/v1.0/me/calendars/cal-2/events");
    assert_eq!(
        call.json(),
        json!({
            "subject": "Kick-off",
            "start": {"dateTime": "2024-06-15T09:00:00", "timeZone": "UTC"},
            "end": {"dateTime": "2024-06-15T10:00:00", "timeZone": "UTC"},
            "isAllDay": false,
            "body": {"contentType": "HTML", "content": "<p>Agenda</p>"},
            "location": {"displayName": "Room 1"},
            "attendees": [
                {"emailAddress": {"address": "ana@contoso.com"}, "type": "required"},
                {"emailAddress": {"address": "bo@contoso.com"}, "type": "required"}
            ],
            "isOnlineMeeting": true,
            "onlineMeetingProvider": "teamsForBusiness"
        })
    );
    assert_eq!(reply.data["id"], "ev1");
}

#[tokio::test]
async fn m365_calendar_create_all_day_event_sends_midnight_datetimes_not_a_date_property() {
    let sim = echo().await;
    let input = args(json!({
        "subject": "Holiday",
        "start_datetime": "2024-08-01T13:45:00",
        "end_datetime": "2024-08-03",
        "is_all_day": true
    }));
    create_event(&graph(&sim), input).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.path, "/v1.0/me/events");
    assert_eq!(
        call.json(),
        json!({
            "subject": "Holiday",
            "start": {"dateTime": "2024-08-01T00:00:00", "timeZone": "UTC"},
            "end": {"dateTime": "2024-08-03T00:00:00", "timeZone": "UTC"},
            "isAllDay": true
        })
    );
}

#[tokio::test]
async fn m365_calendar_update_event_patches_only_given_fields_and_replaces_lists() {
    let sim = echo().await;
    let input = args(json!({
        "event_id": "ev1",
        "subject": "Renamed",
        "end_datetime": "2024-06-15T11:00:00",
        "location": null,
        "categories": []
    }));
    update_event(&graph(&sim), input).await.unwrap();
    let input = args(json!({"event_id": "ev1", "attendees": ["ana@contoso.com"], "body": ""}));
    update_event(&graph(&sim), input).await.unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].method, "PATCH");
    assert_eq!(calls[0].path, "/v1.0/me/events/ev1");
    assert_eq!(
        calls[0].json(),
        json!({
            "subject": "Renamed",
            "end": {"dateTime": "2024-06-15T11:00:00", "timeZone": "UTC"},
            "categories": []
        })
    );
    assert_eq!(
        calls[1].json(),
        json!({
            "body": {"contentType": "HTML", "content": ""},
            "attendees": [{"emailAddress": {"address": "ana@contoso.com"}, "type": "required"}]
        })
    );
}

#[tokio::test]
async fn m365_calendar_delete_event_answers_like_the_belgian_server() {
    let sim = echo().await;
    let reply = delete_event(&graph(&sim), args(json!({"event_id": "ev1"})))
        .await
        .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.method, "DELETE");
    assert_eq!(call.path, "/v1.0/me/events/ev1");
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": {"deleted": true, "event_id": "ev1"}, "message": "success"})
    );
}

#[tokio::test]
async fn m365_calendar_invitations_post_each_answer_with_its_comment() {
    let sim = echo().await;
    let client = graph(&sim);
    let accepted = accept_event(
        &client,
        args(json!({"event_id": "ev1", "comment": "See you"})),
    )
    .await
    .unwrap();
    let declined = decline_event(
        &client,
        args(json!({"event_id": "ev2", "send_response": false})),
    )
    .await
    .unwrap();
    let tentative = tentatively_accept_event(&client, args(json!({"event_id": "ev3"})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert!(calls.iter().all(|c| c.method == "POST"));
    assert_eq!(calls[0].path, "/v1.0/me/events/ev1/accept");
    assert_eq!(
        calls[0].json(),
        json!({"sendResponse": true, "comment": "See you"})
    );
    assert_eq!(calls[1].path, "/v1.0/me/events/ev2/decline");
    assert_eq!(calls[1].json(), json!({"sendResponse": false}));
    assert_eq!(calls[2].path, "/v1.0/me/events/ev3/tentativelyAccept");
    assert_eq!(calls[2].json(), json!({"sendResponse": true}));
    assert_eq!(accepted.data, json!({"accepted": true, "event_id": "ev1"}));
    assert_eq!(declined.data, json!({"declined": true, "event_id": "ev2"}));
    assert_eq!(
        tentative.data,
        json!({"tentatively_accepted": true, "event_id": "ev3"})
    );
}

#[tokio::test]
async fn m365_calendar_event_ids_are_checked_before_any_write() {
    let sim = echo().await;
    let client = graph(&sim);
    let errors = [
        get_event(&client, args(json!({"event_id": "a/b"})))
            .await
            .unwrap_err(),
        delete_event(&client, args(json!({"event_id": ".."})))
            .await
            .unwrap_err(),
        update_event(&client, args(json!({"event_id": "x?y", "subject": "s"})))
            .await
            .unwrap_err(),
        accept_event(&client, args(json!({"event_id": "x#y"})))
            .await
            .unwrap_err(),
        create_event(
            &client,
            args(json!({
                "subject": "s",
                "start_datetime": "2024-06-15T09:00:00",
                "end_datetime": "2024-06-15T10:00:00",
                "calendar_id": "../other"
            })),
        )
        .await
        .unwrap_err(),
    ];

    assert!(errors.iter().all(is_invalid), "{errors:?}");
    assert!(sim.calls().is_empty());
}
