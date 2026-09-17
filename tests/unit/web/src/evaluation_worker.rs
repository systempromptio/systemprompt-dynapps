//! How a trigger lands on the run row: the scheduler leaves no requester,
//! a click records the admin and is marked manual.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

use systemprompt::identifiers::UserId;
use systemprompt_web_admin::evaluation::Trigger;
use systemprompt_web_admin::repositories::evaluation::runs::RunTrigger;

#[test]
fn automatic_trigger_records_no_requester() {
    let trigger = Trigger::Automatic;
    assert_eq!(trigger.kind(), RunTrigger::Automatic);
    assert_eq!(trigger.kind().as_str(), "automatic");
    assert!(trigger.requested_by().is_none());
}

#[test]
fn manual_trigger_records_the_admin() {
    let admin = UserId::new("admin-1");
    let trigger = Trigger::Manual {
        requested_by: admin.clone(),
    };
    assert_eq!(trigger.kind(), RunTrigger::Manual);
    assert_eq!(trigger.kind().as_str(), "manual");
    assert_eq!(trigger.requested_by(), Some(admin));
}
