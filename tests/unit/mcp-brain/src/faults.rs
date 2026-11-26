use dynbrain::brain::fault::{BrainFault, SourceFault};
use dynbrain::properties::PropertyError;
use rmcp::ErrorData;
use systemprompt_mcp_brain::tools::fault::{tool_error, undeclared_filter};

#[test]
fn a_fault_becomes_the_tool_error_a_client_reads() {
    assert_eq!(
        tool_error(&BrainFault::NotFound).code,
        ErrorData::resource_not_found("x", None).code
    );
    assert_eq!(
        tool_error(&BrainFault::Invalid("pass an id")).code,
        ErrorData::invalid_params("x", None).code
    );
    assert_eq!(
        tool_error(&BrainFault::TooLarge(10)).code,
        ErrorData::invalid_params("x", None).code
    );
    assert_eq!(
        tool_error(&BrainFault::Unavailable("files are off")).code,
        ErrorData::internal_error("x", None).code
    );
    let forbidden = tool_error(&BrainFault::Forbidden);
    assert!(forbidden.message.starts_with("Forbidden"), "{forbidden:?}");
}

#[test]
fn a_not_found_does_not_say_whether_the_article_exists() {
    let error = tool_error(&BrainFault::NotFound);
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

#[test]
fn a_refusal_of_the_source_is_the_tool_error_of_the_native_one() {
    let forbidden = tool_error(&BrainFault::Source(SourceFault::Refused(
        "token revoked".to_owned(),
    )));
    assert!(forbidden.message.starts_with("Forbidden"), "{forbidden:?}");
    assert!(forbidden.message.contains("token revoked"), "{forbidden:?}");
    assert_eq!(
        tool_error(&BrainFault::Source(SourceFault::NotFound(
            "gone".to_owned()
        )))
        .code,
        ErrorData::resource_not_found("x", None).code
    );
    assert_eq!(
        tool_error(&BrainFault::Source(SourceFault::Invalid("bad".to_owned()))).code,
        ErrorData::invalid_params("x", None).code
    );
    assert_eq!(
        tool_error(&BrainFault::ManagedAtSource).code,
        ErrorData::invalid_params("x", None).code
    );
    let arguments = serde_json::from_str::<u32>("x").expect_err("not a number");
    let invalid = tool_error(&BrainFault::Arguments(arguments));
    assert!(
        invalid.message.contains("Invalid tool input"),
        "{invalid:?}"
    );
}
