//! The API over its loopback socket, as a browser or the bridge reaches it:
//! a bearer per call, redirects reported rather than followed.

use reqwest::StatusCode;
use serde_json::Value;

pub(super) struct Rest {
    http: reqwest::Client,
    base: String,
}

pub(super) struct Reply {
    pub(super) status: StatusCode,
    pub(super) body: String,
}

impl Reply {
    pub(super) fn json(&self) -> Value {
        serde_json::from_str(&self.body)
            .unwrap_or_else(|e| panic!("{} is not JSON ({e}): {}", self.status, self.body))
    }
}

impl Rest {
    pub(super) fn new(base: &str) -> Self {
        Self {
            http: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("build the HTTP client"),
            base: base.to_owned(),
        }
    }

    async fn send(&self, request: reqwest::RequestBuilder) -> Reply {
        let response = request.send().await.expect("reach the API");
        let status = response.status();
        let body = response.text().await.expect("read the API response");
        Reply { status, body }
    }

    pub(super) async fn get(&self, path: &str, bearer: Option<&str>) -> Reply {
        let mut request = self.http.get(format!("{}{path}", self.base));
        if let Some(token) = bearer {
            request = request.bearer_auth(token);
        }
        self.send(request).await
    }

    pub(super) async fn bytes(&self, path: &str, bearer: &str) -> Vec<u8> {
        let response = self
            .http
            .get(format!("{}{path}", self.base))
            .bearer_auth(bearer)
            .send()
            .await
            .expect("reach the API");
        assert!(
            response.status().is_success(),
            "{path}: {}",
            response.status()
        );
        response.bytes().await.expect("read the content").to_vec()
    }

    pub(super) async fn json(&self, method: &str, path: &str, bearer: &str, body: &Value) -> Reply {
        let method = reqwest::Method::from_bytes(method.as_bytes()).expect("an HTTP method");
        self.send(
            self.http
                .request(method, format!("{}{path}", self.base))
                .bearer_auth(bearer)
                .json(body),
        )
        .await
    }

    pub(super) async fn upload(
        &self,
        path: &str,
        bearer: &str,
        headers: &[(&str, &str)],
        body: Vec<u8>,
    ) -> Reply {
        let mut request = self
            .http
            .post(format!("{}{path}", self.base))
            .bearer_auth(bearer)
            .body(body);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        self.send(request).await
    }
}
