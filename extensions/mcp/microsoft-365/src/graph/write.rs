//! Writes to Graph with a JSON body: create (POST), update (PATCH) and
//! delete. Each answers the resource Graph returns, or `Null` for Graph's
//! empty 202/204, so a tool decides what to tell the model about a write
//! Graph does not echo.

use reqwest::Method;

use super::{GraphBody, GraphClient, GraphError, GraphRequest};

impl GraphClient {
    // JSON: Graph request and response bodies are Graph resources.
    pub async fn post_json(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, GraphError> {
        self.write(Method::POST, path, Some(body)).await
    }

    // Why: Graph answers 411 to a bodiless POST action (`send`) that carries
    // no `Content-Length`, and hyper only writes one for a body.
    pub async fn post_empty(&self, path: &str) -> Result<serde_json::Value, GraphError> {
        let request = GraphRequest::new(Method::POST, path).with_header("content-length", "0");
        self.send(&request).await?.json()
    }

    // JSON: Graph request and response bodies are Graph resources.
    pub async fn patch_json(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, GraphError> {
        self.write(Method::PATCH, path, Some(body)).await
    }

    pub async fn delete(&self, path: &str) -> Result<(), GraphError> {
        self.write(Method::DELETE, path, None).await.map(|_| ())
    }

    // JSON: Graph request and response bodies are Graph resources.
    async fn write(
        &self,
        method: Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, GraphError> {
        let mut request = GraphRequest::new(method, path);
        if let Some(body) = body {
            request = request.with_body(GraphBody::Json(body));
        }
        self.send(&request).await?.json()
    }
}
