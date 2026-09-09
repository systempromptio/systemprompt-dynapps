use std::collections::BTreeMap;
use systemprompt_evaluation::experiments::execution::{ExecutionLimits, FrozenWorkspace};

#[test]
fn frozen_workspace_rejects_portable_path_escapes_and_limits_content() {
    for path in ["../a", "/a", "C:/a", "a\\b", "a//b", "a/./b", "a\nb"] {
        assert!(
            FrozenWorkspace {
                files: BTreeMap::from([(path.into(), "x".into())])
            }
            .validate()
            .is_err()
        );
    }
    let mut workspace = FrozenWorkspace {
        files: BTreeMap::from([("references/context.md".into(), "source".into())]),
    };
    let before = workspace.digest().unwrap();
    workspace
        .files
        .insert("references/context.md".into(), "changed source".into());
    assert_ne!(before, workspace.digest().unwrap());
    workspace
        .files
        .insert("large".into(), "x".repeat(8 * 1024 * 1024));
    assert!(workspace.validate().is_err());
    assert!(
        ExecutionLimits {
            max_turns: 0,
            ..Default::default()
        }
        .validate()
        .is_err()
    );
}

#[cfg(unix)]
#[test]
fn supervised_owned_child_preserves_exit_status_and_output() {
    use std::process::{Command, Stdio};
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "printf supervised; exit 7"])
        .stdout(Stdio::piped());
    let child = systemprompt::models::subprocess::spawn_owned_supervised(command).unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, b"supervised");
}

#[test]
fn evaluation_pricing_rejects_unknown_and_nonfinite_rates() {
    use systemprompt::models::services::{ModelPricing, ProviderRegistry};
    use systemprompt_api::services::gateway::evaluation::request_bound;
    use systemprompt_api::services::gateway::pricing::resolve;
    assert!(resolve("missing", &["unpriced"], None, &ProviderRegistry::default()).is_err());
    assert!(request_bound(&ModelPricing::default(), 100, 10).is_err());
    let pricing = ModelPricing {
        input_per_million: 1.0,
        output_per_million: 5.0,
        ..Default::default()
    };
    assert_eq!(request_bound(&pricing, 100, 10).unwrap(), 4246);
    assert!(
        request_bound(
            &ModelPricing {
                input_per_million: f64::NAN,
                ..pricing
            },
            100,
            10
        )
        .is_err()
    );
}
