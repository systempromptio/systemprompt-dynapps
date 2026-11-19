//! What a model error tells the client: a stable kind and a fixed message,
//! never the provider's own text (it can echo the request or a credential).

use dynwork_chats::reply::model::failure;
use systemprompt::ai::error::{AiError, ProviderError, RequestError, RoutingError};
use systemprompt::identifiers::ProviderId;

const LEAK: &str = "upstream echoed the prompt 'my secret plan' and the key sk-ant-0123";

fn leaky_provider() -> ProviderId {
    ProviderId::new("sk-ant-0123")
}

fn errors() -> Vec<(AiError, &'static str)> {
    vec![
        (
            AiError::Provider(ProviderError::HttpStatus {
                provider: leaky_provider(),
                status: 400,
                retry_after: None,
                body: LEAK.to_owned(),
            }),
            "provider.error",
        ),
        (
            AiError::Provider(ProviderError::Timeout {
                provider: leaky_provider(),
                after_ms: 30_000,
            }),
            "provider.error",
        ),
        (
            AiError::Routing(RoutingError::ProviderNotFound {
                provider: leaky_provider(),
            }),
            "provider.unavailable",
        ),
        (
            AiError::Request(RequestError::PromptTooLong {
                provider: leaky_provider(),
                length: 9,
                max: 1,
            }),
            "provider.invalid-request",
        ),
        (AiError::Io(std::io::Error::other(LEAK)), "internal"),
    ]
}

#[test]
fn a_model_error_reaches_the_client_as_its_kind_and_a_fixed_message() {
    for (error, kind) in errors() {
        let reply = failure(&error);
        assert_eq!(reply.kind, kind, "{error}");
        for secret in ["sk-ant-0123", "secret plan", "upstream"] {
            assert!(!reply.message.contains(secret), "{}", reply.message);
        }
        assert!(!reply.message.is_empty());
    }
}

#[test]
fn every_error_of_one_kind_says_the_same() {
    let messages: Vec<(String, String)> = errors()
        .iter()
        .map(|(error, _)| {
            let reply = failure(error);
            (reply.kind, reply.message)
        })
        .collect();
    let (first, second) = (&messages[0], &messages[1]);
    assert_eq!(first, second, "two provider errors read alike");
}
