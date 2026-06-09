use thothterm_ai::{AiClient, error::AiError, types::*};
use thothterm_config::AiConfig;
use mockito::{Server, ServerGuard};

fn make_config(server: &ServerGuard) -> AiConfig {
    AiConfig {
        enabled: true,
        provider: thothterm_config::AiProvider::Ollama,
        model: "llama3.2".into(),
        base_url: server.url(),
        api_key: String::new(),
        suggestions: true,
        error_analysis: true,
        privacy_mode: false,
    }
}

fn basic_request() -> AiRequest {
    AiRequest {
        prompt: "why did my command fail?".into(),
        context: AiContext {
            cwd: "/home/user".into(),
            shell: "bash".into(),
            os: "Linux".into(),
            recent_commands: vec!["git push origin main".into()],
            last_error: Some("error: failed to push some refs".into()),
            last_exit_code: Some(1),
        },
        request_type: RequestType::ExplainError,
    }
}

#[tokio::test]
async fn disabled_ai_returns_error() {
    let mut config = AiConfig::default();
    config.enabled = false;
    let client = AiClient::new(config);
    let result = client.ask(basic_request()).await;
    assert!(matches!(result, Err(AiError::Disabled)));
}

#[tokio::test]
async fn privacy_mode_blocks_requests() {
    let mut config = AiConfig::default();
    config.enabled = true;
    config.privacy_mode = true;
    let client = AiClient::new(config);
    let result = client.ask(basic_request()).await;
    assert!(matches!(result, Err(AiError::PrivacyMode)));
}

#[tokio::test]
async fn missing_api_key_for_openai_returns_error() {
    let mut config = AiConfig::default();
    config.enabled = true;
    config.provider = thothterm_config::AiProvider::OpenAi;
    config.api_key = String::new();
    let client = AiClient::new(config);
    let result = client.ask(basic_request()).await;
    assert!(matches!(result, Err(AiError::MissingApiKey { .. })));
}

#[tokio::test]
async fn successful_ollama_response() {
    let mut server = Server::new_async().await;
    let mock = server.mock("POST", "/api/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "response": "The push failed because the remote has new commits. Run: `git pull --rebase`",
            "done": true,
            "eval_count": 42
        }"#)
        .create_async()
        .await;

    let client = AiClient::new(make_config(&server));
    let response = client.ask(basic_request()).await.unwrap();

    assert!(!response.text.is_empty());
    assert_eq!(response.tokens_used, Some(42));
    // Command extracted from backticks
    assert_eq!(response.suggested_command, Some("git pull --rebase".into()));

    mock.assert_async().await;
}

#[tokio::test]
async fn ollama_server_error_returns_provider_error() {
    let mut server = Server::new_async().await;
    let mock = server.mock("POST", "/api/generate")
        .with_status(500)
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let client = AiClient::new(make_config(&server));
    let result = client.ask(basic_request()).await;
    assert!(matches!(result, Err(AiError::ProviderError { status: 500, .. })));

    mock.assert_async().await;
}

#[tokio::test]
async fn empty_ollama_response_returns_error() {
    let mut server = Server::new_async().await;
    let mock = server.mock("POST", "/api/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"response": "", "done": true}"#)
        .create_async()
        .await;

    let client = AiClient::new(make_config(&server));
    let result = client.ask(basic_request()).await;
    assert!(matches!(result, Err(AiError::EmptyResponse)));

    mock.assert_async().await;
}

#[tokio::test]
async fn health_check_success() {
    let mut server = Server::new_async().await;
    let mock = server.mock("GET", "/api/tags")
        .with_status(200)
        .with_body(r#"{"models": []}"#)
        .create_async()
        .await;

    let client = AiClient::new(make_config(&server));
    assert!(client.health_check().await.unwrap());

    mock.assert_async().await;
}

#[tokio::test]
async fn health_check_privacy_mode_blocked() {
    let mut config = AiConfig::default();
    config.enabled = true;
    config.privacy_mode = true;
    let client = AiClient::new(config);
    let result = client.health_check().await;
    assert!(matches!(result, Err(AiError::PrivacyMode)));
}

#[test]
fn natural_language_request_type() {
    let req = AiRequest {
        prompt: "list all rust files modified in the last hour".into(),
        context: AiContext::default(),
        request_type: RequestType::NaturalLanguageToShell,
    };
    // Just verify the struct can be created — actual translation is tested via mock
    assert_eq!(req.prompt, "list all rust files modified in the last hour");
}
