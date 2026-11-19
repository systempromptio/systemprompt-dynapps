//! What the connector broker's answer becomes before it reaches Graph: the
//! broker returns the whole `Authorization` value (`Bearer <token>`), and the
//! Graph client keeps only the credential, refusing anything else rather than
//! forwarding it.

use systemprompt_mcp_microsoft_365::graph::{GraphError, bearer_credential};

#[test]
fn m365_core_broker_bearer_keeps_only_the_credential() {
    assert_eq!(bearer_credential("Bearer abc.def").unwrap(), "abc.def");
    assert_eq!(bearer_credential("bearer  abc ").unwrap(), "abc");
    assert_eq!(bearer_credential(" abc ").unwrap(), "abc");
}

#[test]
fn m365_core_broker_refuses_other_schemes_and_empty_credentials() {
    for answer in [
        "Basic dXNlcjpwYXNz",
        "Bearer",
        "Bearer   ",
        "",
        "Bearer a b",
    ] {
        assert!(
            matches!(bearer_credential(answer), Err(GraphError::Config(_))),
            "{answer:?} was forwarded"
        );
    }
}
