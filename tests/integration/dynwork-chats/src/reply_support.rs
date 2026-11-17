//! A scripted model in place of the instance's AI service, the prompt and
//! log calls, and a reader of the `/log` event stream.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use dynwork_chats::catalog::ModelInfo;
use dynwork_chats::ids::ThreadId;
use dynwork_chats::reply::model::{
    ChunkStream, ModelChunk, ModelSource, ReplyError, ReplyModel, ReplyRequest, Usage,
};
use dynwork_chats::store::ModelRef;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use systemprompt::ai::AiService;
use tokio::sync::mpsc;

use crate::http_support::{Answer, App, desktop_token};

pub(crate) const WAIT: Duration = Duration::from_secs(5);

// The model of a test: each reply streams what the test sends on the script
// queued for it, and ends when the test drops that script.
#[derive(Debug, Default)]
pub(crate) struct FakeModel {
    scripts: Mutex<VecDeque<mpsc::UnboundedReceiver<ModelChunk>>>,
    requests: Mutex<Vec<ReplyRequest>>,
}

impl FakeModel {
    pub(crate) fn requests(&self) -> Vec<ReplyRequest> {
        self.requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

#[async_trait]
impl ReplyModel for FakeModel {
    fn default_model(&self) -> Option<ModelRef> {
        Some(ModelRef {
            provider_id: "anthropic".to_owned(),
            model_id: "claude-sonnet".to_owned(),
        })
    }

    async fn stream(&self, request: ReplyRequest) -> Result<ChunkStream, ReplyError> {
        self.requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(request);
        let script = self
            .scripts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop_front()
            .ok_or_else(|| ReplyError::new("provider.error", "no script for this reply"))?;
        Ok(Box::pin(futures_util::stream::unfold(
            script,
            |mut script| async move { script.recv().await.map(|chunk| (chunk, script)) },
        )))
    }
}

#[derive(Debug)]
pub(crate) struct FakeSource {
    model: Arc<FakeModel>,
    available: AtomicBool,
}

impl FakeSource {
    pub(crate) fn new(model: Arc<FakeModel>) -> Self {
        Self {
            model,
            available: AtomicBool::new(true),
        }
    }

    pub(crate) fn switch_off(&self) {
        self.available.store(false, Ordering::SeqCst);
    }
}

pub(crate) fn listed_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "claude-sonnet".to_owned(),
            name: "Claude Sonnet".to_owned(),
            provider_id: "anthropic".to_owned(),
            provider_name: "Anthropic".to_owned(),
            reasoning: true,
            tools: true,
        },
        ModelInfo {
            id: "deepseek-chat".to_owned(),
            name: "deepseek-chat".to_owned(),
            provider_id: "deepseek".to_owned(),
            provider_name: "deepseek".to_owned(),
            reasoning: false,
            tools: false,
        },
    ]
}

impl ModelSource for FakeSource {
    fn pick(&self, gateway: Option<Arc<dyn ReplyModel>>) -> Option<Arc<dyn ReplyModel>> {
        assert!(gateway.is_none(), "no AI service is mounted in the suites");
        let model: Arc<dyn ReplyModel> = Arc::<FakeModel>::clone(&self.model);
        self.available.load(Ordering::SeqCst).then_some(model)
    }

    fn list(&self, ai: Option<&AiService>) -> Option<Vec<ModelInfo>> {
        assert!(ai.is_none(), "no AI service is mounted in the suites");
        self.available.load(Ordering::SeqCst).then(listed_models)
    }
}

// What the test makes the model say, chunk by chunk.
pub(crate) struct Script(mpsc::UnboundedSender<ModelChunk>);

impl Script {
    pub(crate) fn text(&self, text: &str) {
        self.send(ModelChunk::Text(text.to_owned()));
    }

    pub(crate) fn usage(&self, input: u64, output: u64) {
        self.send(ModelChunk::Usage(Usage {
            input,
            output,
            raw_finish: Some("end_turn"),
            ..Usage::default()
        }));
    }

    pub(crate) fn fail(&self, kind: &str, message: &str) {
        self.send(ModelChunk::Error(ReplyError::new(kind, message)));
    }

    fn send(&self, chunk: ModelChunk) {
        assert!(self.0.send(chunk).is_ok(), "the reply stopped reading");
    }
}

impl App {
    // Queues the script of the next reply.
    pub(crate) fn script(&self) -> Script {
        let (sender, receiver) = mpsc::unbounded_channel();
        self.model
            .scripts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push_back(receiver);
        Script(sender)
    }

    // Queues a whole reply: these texts, then usage, then the end.
    pub(crate) fn scripted(&self, texts: &[&str]) {
        let script = self.script();
        for text in texts {
            script.text(text);
        }
        script.usage(120, 14);
    }

    pub(crate) async fn create_thread(&self, owner: &str, id: &str) {
        let created = self
            .call(Method::PUT, &format!("/threads/{id}"))
            .bearer(&desktop_token(owner))
            .json(&json!({ "title": "Cloud chat", "model": { "providerID": "anthropic", "id": "claude-sonnet" } }))
            .send()
            .await;
        assert_eq!(created.status, StatusCode::OK, "{:?}", created.body);
    }

    pub(crate) async fn prompt(
        &self,
        caller: &str,
        thread: &str,
        message: &str,
        text: &str,
    ) -> Answer {
        self.call(Method::POST, &format!("/threads/{thread}/prompt"))
            .bearer(&desktop_token(caller))
            .json(&json!({ "messageID": message, "parts": [{ "type": "text", "text": text }] }))
            .send()
            .await
    }

    pub(crate) async fn interrupt(&self, caller: &str, thread: &str) -> Answer {
        self.call(Method::POST, &format!("/threads/{thread}/interrupt"))
            .bearer(&desktop_token(caller))
            .send()
            .await
    }

    // Waits until the thread's reply has ended and been compacted.
    pub(crate) async fn settled(&self, thread: &str) {
        let thread: ThreadId = thread.parse().expect("a thread id");
        assert!(
            self.state.replies.settled(&thread, WAIT).await,
            "the reply did not end"
        );
    }

    pub(crate) async fn log(&self, caller: &str, thread: &str, query: &str) -> Log {
        self.log_with(caller, thread, query, None).await
    }

    pub(crate) async fn log_with(
        &self,
        caller: &str,
        thread: &str,
        query: &str,
        last_event_id: Option<&str>,
    ) -> Log {
        let mut call = self
            .call(Method::GET, &format!("/threads/{thread}/log{query}"))
            .bearer(&desktop_token(caller));
        if let Some(id) = last_event_id {
            call = call.header("last-event-id", id);
        }
        let response = call.open().await;
        let status = response.status();
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_owned();
        Log {
            status,
            content_type,
            body: response.into_body(),
            pending: String::new(),
            ended: false,
        }
    }

    pub(crate) async fn messages(&self, caller: &str, thread: &str) -> Vec<Value> {
        let answer = self
            .call(Method::GET, &format!("/threads/{thread}/messages"))
            .bearer(&desktop_token(caller))
            .send()
            .await;
        assert_eq!(answer.status, StatusCode::OK, "{:?}", answer.body);
        answer.body["messages"]
            .as_array()
            .cloned()
            .unwrap_or_default()
    }
}

// One event read from `/log`: its SSE `id` and its JSON `data`.
#[derive(Debug, Clone)]
pub(crate) struct LogEvent {
    pub(crate) id: Option<i64>,
    pub(crate) data: Value,
}

impl LogEvent {
    pub(crate) fn kind(&self) -> &str {
        self.data["type"].as_str().unwrap_or("")
    }
}

pub(crate) struct Log {
    pub(crate) status: StatusCode,
    pub(crate) content_type: String,
    body: Body,
    pending: String,
    ended: bool,
}

pub(crate) const TERMINAL: [&str; 3] = [
    "session.execution.succeeded",
    "session.execution.failed",
    "session.execution.interrupted",
];

impl Log {
    // The next event, or `None` when the stream ended or nothing came in time.
    pub(crate) async fn next(&mut self, wait: Duration) -> Option<LogEvent> {
        loop {
            if let Some(at) = self.pending.find("\n\n") {
                let block: String = self.pending.drain(..at + 2).collect();
                if let Some(event) = parse(&block) {
                    return Some(event);
                }
                continue;
            }
            if self.ended {
                return None;
            }
            match tokio::time::timeout(wait, self.body.frame()).await {
                Ok(Some(Ok(frame))) => {
                    if let Ok(data) = frame.into_data() {
                        self.pending.push_str(&String::from_utf8_lossy(&data));
                    }
                },
                Ok(Some(Err(error))) => panic!("the log stream failed: {error}"),
                Ok(None) => self.ended = true,
                Err(_elapsed) => return None,
            }
        }
    }

    // Events up to and including the reply's terminal one.
    pub(crate) async fn until_terminal(&mut self) -> Vec<LogEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.next(WAIT).await {
            let done = TERMINAL.contains(&event.kind());
            events.push(event);
            if done {
                return events;
            }
        }
        panic!("the log ended without a terminal event: {events:?}");
    }

    // Every event until the stream closes.
    pub(crate) async fn until_closed(&mut self) -> Vec<LogEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.next(WAIT).await {
            events.push(event);
        }
        assert!(self.ended, "the log did not close: {events:?}");
        events
    }
}

fn parse(block: &str) -> Option<LogEvent> {
    let mut id = None;
    let mut data = Vec::new();
    for line in block.lines() {
        if let Some(value) = line.strip_prefix("id:") {
            id = value.trim().parse().ok();
        } else if let Some(value) = line.strip_prefix("data:") {
            data.push(value.strip_prefix(' ').unwrap_or(value));
        }
    }
    if data.is_empty() {
        return None;
    }
    let data = serde_json::from_str(&data.join("\n")).expect("an event is JSON");
    Some(LogEvent { id, data })
}

pub(crate) fn kinds(events: &[LogEvent]) -> Vec<&str> {
    events.iter().map(LogEvent::kind).collect()
}

pub(crate) fn deltas(events: &[LogEvent]) -> String {
    events
        .iter()
        .filter(|event| event.kind() == "session.text.delta")
        .filter_map(|event| event.data["data"]["delta"].as_str())
        .collect()
}
