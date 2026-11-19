//! The calendar list tools against a simulated Graph: the request each
//! builds (path, fields, ordering, quoted date filters, the attendee lambda
//! with `ConsistencyLevel: eventual`, `$search` only alone), the view range,
//! the Belgian list reply, and the shared generic handler.

use serde_json::json;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt_mcp_microsoft_365::tools::calendar::{
    CALENDAR_FIELDS, CALENDAR_VIEW_FIELDS, EVENT_LIST_FIELDS, ListCalendarsInput,
    list_calendar_view, list_calendars, list_events, list_shared_calendar_events,
};
use systemprompt_mcp_microsoft_365::tools::graph_tool::GraphTool;
use systemprompt_mcp_microsoft_365::tools::reply::{GraphList, MORE_HINT};

use crate::m365_pim_support::{args, assert_domain_registered, context, graph, is_invalid};
use crate::m365_support::{Reply, Sim, collection};

async fn events() -> Sim {
    Sim::start(|_, _| Reply::json(200, &collection(&[json!({"id": "e1"})], None))).await
}

#[test]
fn m365_calendar_all_eleven_belgian_tools_are_registered_as_pinned() {
    assert_domain_registered("calendar", 11);
}

#[tokio::test]
async fn m365_calendar_list_calendars_selects_fields_and_pages_without_top() {
    let sim = Sim::start(|seen, _| {
        let next = format!(
            "http://{}/v1.0/me/calendars?$skip=4",
            seen.header("host").unwrap()
        );
        Reply::json(200, &collection(&[json!({"id": "c1"})], Some(&next)))
    })
    .await;
    let reply = list_calendars(&graph(&sim), args(json!({"skip": 2})))
        .await
        .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.path, "/v1.0/me/calendars");
    assert_eq!(call.q("$select"), Some(CALENDAR_FIELDS));
    assert_eq!(call.q("$skip"), Some("2"));
    assert_eq!(call.q("$top"), None);
    let wire = serde_json::to_value(&reply).unwrap();
    assert_eq!(
        wire,
        json!({
            "data": [{"id": "c1"}],
            "message": "success",
            "has_more": true,
            "total_returned": 1,
            "hint": MORE_HINT
        })
    );
}

#[tokio::test]
async fn m365_calendar_list_events_filters_dates_and_attendee_with_eventual_consistency() {
    let sim = Sim::start(|_, _| Reply::json(200, &collection(&[], None))).await;
    let input = args(json!({
        "calendar_id": "AAMk=",
        "start_date": "2024-01-01",
        "end_date": "2024-12-31",
        "attendee": "o'brien@contoso.com",
        "query": "subject:budget",
        "top": 99,
        "skip": 25
    }));
    let reply = list_events(&graph(&sim), input).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.method, "GET");
    assert_eq!(call.path, "/v1.0/me/calendars/AAMk=/events");
    assert_eq!(call.q("$select"), Some(EVENT_LIST_FIELDS));
    assert_eq!(call.q("$orderby"), Some("start/dateTime"));
    assert_eq!(call.q("$top"), Some("50"));
    assert_eq!(call.q("$skip"), Some("25"));
    assert_eq!(
        call.q("$filter"),
        Some(
            "start/dateTime ge '2024-01-01' and start/dateTime le '2024-12-31' and \
             attendees/any(a: a/emailAddress/address eq 'o''brien@contoso.com')"
        )
    );
    assert_eq!(
        call.q("$search"),
        None,
        "Graph refuses $search beside $filter"
    );
    assert_eq!(call.header("consistencylevel"), Some("eventual"));
    assert_eq!(reply.data.len(), 0);
    assert_eq!(reply.has_more, None);
}

#[tokio::test]
async fn m365_calendar_list_events_searches_alone_on_the_default_calendar() {
    let sim = Sim::start(|_, _| Reply::json(200, &collection(&[json!({"id": "e1"})], None))).await;
    let input = args(json!({"query": "say \"hi\""}));
    let reply = list_events(&graph(&sim), input).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.path, "/v1.0/me/events");
    assert_eq!(call.q("$search"), Some("\"say \\\"hi\\\"\""));
    assert_eq!(call.q("$filter"), None);
    assert_eq!(call.q("$top"), Some("25"));
    assert_eq!(call.q("$skip"), None);
    assert_eq!(call.header("consistencylevel"), None);
    assert_eq!(reply.data, vec![json!({"id": "e1"})]);
}

#[tokio::test]
async fn m365_calendar_date_filters_alone_need_no_eventual_consistency() {
    let sim = events().await;
    let input = args(json!({"start_date": "2024-01-01T00:00:00"}));
    list_events(&graph(&sim), input).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(
        call.q("$filter"),
        Some("start/dateTime ge '2024-01-01T00:00:00'")
    );
    assert_eq!(call.header("consistencylevel"), None);
}

#[tokio::test]
async fn m365_calendar_view_sends_the_range_and_a_checked_select() {
    let sim = Sim::start(|_, _| Reply::json(200, &collection(&[], None))).await;
    let input = args(json!({
        "start_datetime": "2024-06-01T00:00:00Z",
        "end_datetime": "2024-06-30T23:59:59Z",
        "select": "id, subject,start",
        "query": "standup"
    }));
    list_calendar_view(&graph(&sim), input).await.unwrap();
    let input = args(json!({
        "start_datetime": "2024-06-01T00:00:00Z",
        "end_datetime": "2024-06-30T23:59:59Z",
        "calendar_id": "cal-2",
        "attendee": "ana@contoso.com",
        "query": "ignored"
    }));
    list_calendar_view(&graph(&sim), input).await.unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/calendarView");
    assert_eq!(calls[0].q("startDateTime"), Some("2024-06-01T00:00:00Z"));
    assert_eq!(calls[0].q("endDateTime"), Some("2024-06-30T23:59:59Z"));
    assert_eq!(calls[0].q("$select"), Some("id,subject,start"));
    assert_eq!(calls[0].q("$orderby"), Some("start/dateTime"));
    assert_eq!(calls[0].q("$search"), Some("\"standup\""));
    assert_eq!(calls[0].q("$filter"), None);
    assert_eq!(calls[1].path, "/v1.0/me/calendars/cal-2/calendarView");
    assert_eq!(calls[1].q("$select"), Some(CALENDAR_VIEW_FIELDS));
    assert_eq!(
        calls[1].q("$filter"),
        Some("attendees/any(a: a/emailAddress/address eq 'ana@contoso.com')")
    );
    assert_eq!(calls[1].q("$search"), None);
    assert_eq!(calls[1].header("consistencylevel"), Some("eventual"));
}

#[tokio::test]
async fn m365_calendar_bad_ids_and_selects_never_reach_graph() {
    let sim = events().await;
    let traversal = list_events(&graph(&sim), args(json!({"calendar_id": "../me"}))).await;
    let select = list_calendar_view(
        &graph(&sim),
        args(json!({
            "start_datetime": "2024-06-01T00:00:00Z",
            "end_datetime": "2024-06-02T00:00:00Z",
            "select": "id;drop"
        })),
    )
    .await;
    let user = list_shared_calendar_events(&graph(&sim), args(json!({"user_id": "a/b"}))).await;

    assert!(is_invalid(&traversal.unwrap_err()));
    assert!(is_invalid(&select.unwrap_err()));
    assert!(is_invalid(&user.unwrap_err()));
    assert!(sim.calls().is_empty());
}

#[tokio::test]
async fn m365_calendar_shared_events_use_a_view_only_with_both_dates() {
    let sim = events().await;
    let both = args(json!({
        "user_id": "ana@contoso.com",
        "start_date": "2024-06-01",
        "end_date": "2024-06-30",
        "attendee": "bo@contoso.com"
    }));
    list_shared_calendar_events(&graph(&sim), both)
        .await
        .unwrap();
    let one = args(json!({"user_id": "ana@contoso.com", "start_date": "2024-06-01", "skip": 5}));
    list_shared_calendar_events(&graph(&sim), one)
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/users/ana@contoso.com/calendarView");
    assert_eq!(calls[0].q("startDateTime"), Some("2024-06-01"));
    assert_eq!(calls[0].q("endDateTime"), Some("2024-06-30"));
    assert_eq!(calls[0].q("$select"), Some(EVENT_LIST_FIELDS));
    assert_eq!(
        calls[0].q("$filter"),
        Some("attendees/any(a: a/emailAddress/address eq 'bo@contoso.com')")
    );
    assert_eq!(calls[0].header("consistencylevel"), Some("eventual"));
    assert_eq!(calls[1].path, "/v1.0/users/ana@contoso.com/events");
    assert_eq!(calls[1].q("startDateTime"), None);
    assert_eq!(
        calls[1].q("$filter"),
        Some("start/dateTime ge '2024-06-01'")
    );
    assert_eq!(calls[1].q("$skip"), Some("5"));
}

#[tokio::test]
async fn m365_calendar_eventual_consistency_rides_on_every_walked_page() {
    let sim = Sim::start(|seen, index| {
        let host = seen.header("host").unwrap().to_owned();
        match index {
            0 => Reply::json(
                400,
                &json!({"error": {"code": "BadRequest", "message": "'$skip' is not supported by the service."}}),
            ),
            1 => {
                let next = format!("http://{host}/v1.0/me/events?page=2");
                Reply::json(200, &collection(&[json!({"id": "e1"}), json!({"id": "e2"})], Some(&next)))
            },
            _ => Reply::json(200, &collection(&[json!({"id": "e3"})], None)),
        }
    })
    .await;
    let input = args(json!({"attendee": "ana@contoso.com", "skip": 1, "top": 2}));
    let reply = list_events(&graph(&sim), input).await.unwrap();

    let calls = sim.calls();
    assert_eq!(calls.len(), 3);
    assert!(
        calls
            .iter()
            .all(|c| c.header("consistencylevel") == Some("eventual"))
    );
    assert_eq!(calls[1].q("$skip"), None);
    assert_eq!(reply.data, vec![json!({"id": "e2"}), json!({"id": "e3"})]);
}

#[tokio::test]
async fn m365_calendar_generic_handler_runs_the_tool_and_reads_its_hints() {
    let sim = events().await;
    let client = graph(&sim);
    let tool = GraphTool::<ListCalendarsInput, GraphList> {
        name: "d365_list_calendars",
        description: "List calendars.",
        graph: Some(&client),
        run: |graph, input| Box::pin(list_calendars(graph, input)),
    };
    let execution = McpExecutionId::new("m365-calendar-exec");
    let (reply, summary) = tool
        .handle(args(json!({})), &context(), &execution)
        .await
        .unwrap();
    let detached = GraphTool {
        graph: None,
        ..tool
    };
    let refused = detached
        .handle(args(json!({})), &context(), &execution)
        .await
        .unwrap_err();

    assert!(tool.read_only());
    assert_eq!(tool.tool_name(), "d365_list_calendars");
    assert_eq!(reply.data, vec![json!({"id": "e1"})]);
    assert_eq!(summary, "1 items returned");
    assert!(refused.message.contains("not attached"), "{refused:?}");
    assert_eq!(sim.calls().len(), 1);
}
