use systemprompt_mcp_agent::CliLocation;

pub(crate) fn cli() -> CliLocation {
    CliLocation {
        bin: std::path::PathBuf::from("systemprompt"),
        workdir: std::env::temp_dir(),
    }
}
