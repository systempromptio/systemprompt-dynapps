use dynbrain::brain::fault::BrainFault;
use dynbrain::properties::PropertyError;
use rmcp::ErrorData;
use systemprompt_mcp_brain::tools::fault::{tool_error, undeclared_filter};

#[test]
fn a_fault_becomes_the_tool_error_a_client_reads() {
    assert_eq!(
        tool_error(BrainFault::NotFound).code,
        ErrorData::resource_not_found("x", None).code
    );
    assert_eq!(
        tool_error(BrainFault::Invalid("pass an id")).code,
        ErrorData::invalid_params("x", None).code
    );
    assert_eq!(
        tool_error(BrainFault::TooLarge(10)).code,
        ErrorData::invalid_params("x", None).code
    );
    assert_eq!(
        tool_error(BrainFault::Unavailable("files are off")).code,
        ErrorData::internal_error("x", None).code
    );
    let forbidden = tool_error(BrainFault::Forbidden);
    assert!(forbidden.message.starts_with("Forbidden"), "{forbidden:?}");
}

#[test]
fn a_not_found_does_not_say_whether_the_article_exists() {
    let error = tool_error(BrainFault::NotFound);
    assert_eq!(error.message, "not found");
}

#[test]
fn an_undeclared_filter_is_an_empty_result_with_its_error() {
    let fault = BrainFault::Property(PropertyError::NotDeclared {
        name: "owner".to_owned(),
        declared: "status".to_owned(),
    });
    let reply = undeclared_filter(fault, "results").expect("an empty reply");
    assert_eq!(reply["results"], serde_json::json!([]));
    assert_eq!(reply["total"], 0);
    assert!(
        reply["error"]
            .as_str()
            .is_some_and(|error| error.contains("'owner' is not a metadata field")),
        "{reply}"
    );
    let list = undeclared_filter(
        BrainFault::Property(PropertyError::NotDeclared {
            name: "owner".to_owned(),
            declared: "none".to_owned(),
        }),
        "documents",
    )
    .expect("an empty reply");
    assert_eq!(list["documents"], serde_json::json!([]));
}

#[test]
fn any_other_fault_stays_a_tool_error() {
    let refused = undeclared_filter(BrainFault::Forbidden, "results");
    assert!(refused.is_err());
    let other = undeclared_filter(
        BrainFault::Property(PropertyError::Unknown("owner".to_owned())),
        "results",
    );
    assert!(other.is_err());
}
