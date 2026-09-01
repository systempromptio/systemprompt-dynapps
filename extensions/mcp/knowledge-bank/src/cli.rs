//! Argument parsing for the binary's two modes.
//!
//! Argument parsing for the binary's two modes: serve (the default, what core
//! spawns) and `ingest`, the operator's one-shot run of the same ingestion
//! library the scheduler and the `sync_knowledge` tool drive.

use crate::ingest::RunOptions;

/// What the process was asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Serve,
    Ingest(RunOptions),
    Help,
}

pub const USAGE: &str =
    "systemprompt-mcp-knowledge-bank [serve | ingest [--rebuild] [--source <id>]... | --help]

  serve     Serve the knowledge-bank MCP tools (default; what the platform spawns).
  ingest    Run one ingestion pass and exit. --rebuild drops the selected categories
            first; --source limits the run to a category id (repeatable).";

pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Command, String> {
    let mut args = args.into_iter();
    let Some(mode) = args.next() else {
        return Ok(Command::Serve);
    };
    match mode.as_str() {
        "serve" => Ok(Command::Serve),
        "--help" | "-h" | "help" => Ok(Command::Help),
        "ingest" => {
            let mut options = RunOptions::default();
            while let Some(flag) = args.next() {
                match flag.as_str() {
                    "--rebuild" => options.rebuild = true,
                    "--source" => {
                        let id = args
                            .next()
                            .ok_or_else(|| "--source needs a category id".to_owned())?;
                        options.sources.push(id);
                    },
                    other => return Err(format!("unknown ingest flag '{other}'\n\n{USAGE}")),
                }
            }
            Ok(Command::Ingest(options))
        },
        other => Err(format!("unknown command '{other}'\n\n{USAGE}")),
    }
}
