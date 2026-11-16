//! What a Teams meeting leaves behind: its transcripts, recordings and
//! attendance reports.
//!
//! A transcript's content (`WebVTT`) is kept as one of the caller's documents
//! and comes back as a `FileOut` (the Belgian server returned the whole text
//! to the model). A recording is listed and read as Graph's metadata only, as
//! the Belgian server did: its video is not downloaded.

use reqwest::Method;
use rmcp::ErrorData as McpError;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use systemprompt_mcp_shared::files::{DownloadedFile, FileCall, McpFiles};

use super::file_tool::file_tools;
use super::graph_tool::graph_tools;
use super::meetings::meeting_path;
use super::reply::{GraphItem, GraphList};
use super::teams::list_page;
use crate::graph::sanitize::validate_id;
use crate::graph::{GraphClient, GraphError, GraphQuery, GraphRequest};

pub const TRANSCRIPT_MIME: &str = "text/vtt";
const NAME_TAIL_CHARS: usize = 12;

const LIST_MEETING_TRANSCRIPTS: &str = "List transcripts for an online meeting. Page with `skip`.";
const GET_MEETING_TRANSCRIPT_CONTENT: &str = "Get the content of a meeting transcript (WebVTT) \
    as a file: it is saved to the user's documents and returned as a reference with a \
    download link (the text is not inlined).";
const LIST_MEETING_RECORDINGS: &str = "List recordings for an online meeting. Page with `skip`.";
const GET_MEETING_RECORDING: &str =
    "Get details of a specific meeting recording (its metadata, not the video).";
const LIST_MEETING_ATTENDANCE_REPORTS: &str =
    "List attendance reports for an online meeting. Page with `skip`.";

graph_tools! {
    "d365_list_meeting_transcripts" => list_meeting_transcripts, MeetingSkipInput, GraphList,
        LIST_MEETING_TRANSCRIPTS;
    "d365_list_meeting_recordings" => list_meeting_recordings, MeetingSkipInput, GraphList,
        LIST_MEETING_RECORDINGS;
    "d365_get_meeting_recording" => get_meeting_recording, RecordingIdInput, GraphItem,
        GET_MEETING_RECORDING;
    "d365_list_meeting_attendance_reports" => list_meeting_attendance_reports,
        MeetingSkipInput, GraphList, LIST_MEETING_ATTENDANCE_REPORTS;
}

file_tools! {
    "d365_get_meeting_transcript_content" => get_meeting_transcript_content, TranscriptIdInput,
        GraphItem, GET_MEETING_TRANSCRIPT_CONTENT;
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MeetingSkipInput {
    #[schemars(description = "The online meeting ID.")]
    pub meeting_id: String,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RecordingIdInput {
    #[schemars(description = "The online meeting ID.")]
    pub meeting_id: String,
    #[schemars(description = "The recording ID.")]
    pub recording_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TranscriptIdInput {
    #[schemars(description = "The online meeting ID.")]
    pub meeting_id: String,
    #[schemars(description = "The transcript ID.")]
    pub transcript_id: String,
}

async fn list_under(
    graph: &GraphClient,
    meeting_id: &str,
    collection: &str,
    skip: i64,
) -> Result<GraphList, GraphError> {
    let path = format!("{}/{collection}", meeting_path(meeting_id)?);
    list_page(graph, &path, GraphQuery::new(), skip, None).await
}

fn transcript_name(transcript_id: &str) -> String {
    let skipped = transcript_id
        .chars()
        .count()
        .saturating_sub(NAME_TAIL_CHARS);
    let tail: String = transcript_id.chars().skip(skipped).collect();
    format!("transcript-{tail}.vtt")
}

pub async fn list_meeting_transcripts(
    graph: &GraphClient,
    input: MeetingSkipInput,
) -> Result<GraphList, GraphError> {
    list_under(graph, &input.meeting_id, "transcripts", input.skip).await
}

pub async fn list_meeting_recordings(
    graph: &GraphClient,
    input: MeetingSkipInput,
) -> Result<GraphList, GraphError> {
    list_under(graph, &input.meeting_id, "recordings", input.skip).await
}

pub async fn list_meeting_attendance_reports(
    graph: &GraphClient,
    input: MeetingSkipInput,
) -> Result<GraphList, GraphError> {
    list_under(graph, &input.meeting_id, "attendanceReports", input.skip).await
}

pub async fn get_meeting_recording(
    graph: &GraphClient,
    input: RecordingIdInput,
) -> Result<GraphItem, GraphError> {
    let recording = validate_id(&input.recording_id, "recording_id")?;
    let path = format!(
        "{}/recordings/{recording}",
        meeting_path(&input.meeting_id)?
    );
    let data = graph.get_json(&path, &GraphQuery::new()).await?;
    Ok(GraphItem::new(data))
}

pub async fn get_meeting_transcript_content(
    graph: &GraphClient,
    files: &McpFiles,
    call: FileCall<'_>,
    input: TranscriptIdInput,
) -> Result<GraphItem, McpError> {
    let transcript = validate_id(&input.transcript_id, "transcript_id")?;
    let path = format!(
        "{}/transcripts/{transcript}/content",
        meeting_path(&input.meeting_id)?
    );
    let request = GraphRequest::new(Method::GET, path).with_header("accept", TRANSCRIPT_MIME);
    let response = graph.send(&request).await?;
    let download = DownloadedFile {
        name: transcript_name(transcript),
        mime: response
            .content_type
            .unwrap_or_else(|| TRANSCRIPT_MIME.to_owned()),
        bytes: response.body,
    };
    let file = files.store_download(call, download).await?;
    let file = serde_json::to_value(&file).map_err(GraphError::Decode)?;
    Ok(GraphItem::new(json!({
        "meeting_id": input.meeting_id,
        "transcript_id": input.transcript_id,
        "file": file,
    })))
}
