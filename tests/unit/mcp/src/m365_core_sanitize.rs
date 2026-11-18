//! The `microsoft-365` value checks ported from the Belgian `sanitize.py`,
//! the endpoint guard and the query encoding: what may reach a Graph path,
//! an `OData` expression or the wire.

use systemprompt::traits::SecretLookup;
use systemprompt_mcp_microsoft_365::graph::sanitize::{
    clamp_skip, clamp_top, sanitize_odata_datetime, sanitize_odata_string, sanitize_search,
    sanitize_select, validate_folder, validate_hostname, validate_id, validate_path,
};
use systemprompt_mcp_microsoft_365::graph::{
    DEFAULT_GRAPH_URL, GRAPH_URL_SECRET, GraphEndpoint, GraphError, GraphQuery,
};

use crate::m365_support::Secrets;

#[test]
fn ids_accept_real_graph_shapes_and_refuse_traversal() {
    for id in [
        "AAMkAGI2TAAA=",
        "19:abc_def@unq.gbl.spaces",
        "contoso.sharepoint.com,1c2d,3e4f",
        "5f0c-11e9!a",
    ] {
        assert_eq!(validate_id(id, "id").unwrap(), id);
    }
    for bad in [
        "", "../me", "a/b", "a\\b", "a..b", "a b", "a?b", "a#b", "a%2Fb",
    ] {
        assert!(
            matches!(validate_id(bad, "message_id"), Err(GraphError::Invalid(m)) if m.contains("message_id")),
            "{bad:?} should be refused"
        );
    }
}

#[test]
fn folders_take_well_known_names_in_any_case_or_ids() {
    assert_eq!(validate_folder("Inbox").unwrap(), "Inbox");
    assert_eq!(validate_folder("sentitems").unwrap(), "sentitems");
    assert_eq!(validate_folder("AAMkAD=").unwrap(), "AAMkAD=");
    assert!(validate_folder("inbox/../../users").is_err());
}

#[test]
fn paths_and_hostnames_are_checked() {
    assert_eq!(
        validate_path("/Docs/Q3 report.xlsx/").unwrap(),
        "Docs/Q3 report.xlsx"
    );
    assert!(validate_path("Docs/../secret").is_err());
    assert_eq!(
        validate_hostname("contoso.sharepoint.com").unwrap(),
        "contoso.sharepoint.com"
    );
    for bad in [
        "",
        "-bad.com",
        "a..b",
        "host/path",
        "bad_host.com",
        "a.-b.com",
    ] {
        assert!(validate_hostname(bad).is_err(), "{bad:?} should be refused");
    }
}

#[test]
fn odata_literals_escape_quotes() {
    assert_eq!(sanitize_odata_string("o'brien"), "o''brien");
    assert_eq!(sanitize_search(r#"it's "q3""#), r#"it''s \"q3\""#);
}

#[test]
fn datetimes_must_be_iso_8601_with_a_zone() {
    for ok in [
        "2024-01-01T00:00:00Z",
        "2024-01-01T00:00:00.123Z",
        "2024-12-31T23:59:59+02:00",
        "2024-12-31T23:59:59.5-05:30",
    ] {
        assert_eq!(sanitize_odata_datetime(ok).unwrap(), ok);
    }
    for bad in [
        "2024-01-01",
        "2024-01-01T00:00:00",
        "2024-01-01T00:00Z",
        "2024-01-01T00:00:00Z or 1 eq 1",
        "'2024-01-01T00:00:00Z'",
        "2024-01-01T00:00:00.Z",
    ] {
        assert!(
            sanitize_odata_datetime(bad).is_err(),
            "{bad:?} should be refused"
        );
    }
}

#[test]
fn select_lists_take_fields_and_type_casts_only() {
    assert_eq!(
        sanitize_select(
            " id, displayName ,microsoft.graph.aadUserConversationMember/email",
            "select"
        )
        .unwrap(),
        "id,displayName,microsoft.graph.aadUserConversationMember/email"
    );
    for bad in ["", " , ", "id,name)", "id;drop", "1id", "id,'x'"] {
        assert!(
            sanitize_select(bad, "select").is_err(),
            "{bad:?} should be refused"
        );
    }
}

#[test]
fn top_and_skip_are_clamped() {
    assert_eq!(clamp_top(0, 50), 1);
    assert_eq!(clamp_top(25, 50), 25);
    assert_eq!(clamp_top(500, 50), 50);
    assert_eq!(clamp_skip(-3), 0);
    assert_eq!(clamp_skip(40), 40);
}

#[test]
fn query_values_are_percent_encoded_with_spaces_as_percent_20() {
    let query = GraphQuery::new()
        .with("$filter", "isRead eq false and subject eq 'a+b&c'")
        .with("$top", "5");
    assert_eq!(
        query.encode(),
        "%24filter=isRead%20eq%20false%20and%20subject%20eq%20%27a%2Bb%26c%27&%24top=5"
    );
}

#[test]
fn the_endpoint_defaults_to_graph_and_refuses_unsafe_overrides() {
    let none = Secrets::with(&[]);
    let lookup: &dyn SecretLookup = none.as_ref();
    let default = GraphEndpoint::from_lookup(lookup).unwrap();
    assert_eq!(default.base().as_str(), DEFAULT_GRAPH_URL);

    let set = Secrets::with(&[(GRAPH_URL_SECRET, "http://127.0.0.1:9/v1.0/")]);
    let simulated = GraphEndpoint::from_lookup(set.as_ref()).unwrap();
    assert_eq!(simulated.base().as_str(), "http://127.0.0.1:9/v1.0");

    for bad in [
        "http://graph.example.com/v1.0",
        "https://graph.microsoft.com/v1.0?x=1",
        "https://user:pw@graph.microsoft.com/v1.0",
        "not a url",
    ] {
        assert!(
            GraphEndpoint::parse(bad).is_err(),
            "{bad:?} should be refused"
        );
    }
}

#[test]
fn paths_stay_under_the_endpoint_and_links_must_share_its_origin() {
    let endpoint = GraphEndpoint::parse(DEFAULT_GRAPH_URL).unwrap();
    assert_eq!(
        endpoint
            .resolve("/me/mailFolders/inbox/messages")
            .unwrap()
            .as_str(),
        "https://graph.microsoft.com/v1.0/me/mailFolders/inbox/messages"
    );
    for bad in [
        "me/messages",
        "/../beta/me",
        "/me?$top=1",
        "/me#x",
        "/me/./x",
    ] {
        assert!(endpoint.resolve(bad).is_err(), "{bad:?} should be refused");
    }
    assert!(
        endpoint
            .admit_link("https://graph.microsoft.com/v1.0/me/messages?$skiptoken=x")
            .is_ok()
    );
    for bad in [
        "https://evil.example/v1.0/me/messages",
        "http://graph.microsoft.com/v1.0/me",
        "https://graph.microsoft.com/beta/me",
        "https://graph.microsoft.com/v1.0x/me",
        "https://a:b@graph.microsoft.com/v1.0/me",
    ] {
        assert!(
            matches!(endpoint.admit_link(bad), Err(GraphError::OffEndpoint)),
            "{bad:?} should be refused"
        );
    }
}
