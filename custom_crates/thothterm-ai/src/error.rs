use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("AI is disabled in config")]
    Disabled,

    #[error("Privacy mode is enabled — AI network calls blocked")]
    PrivacyMode,

    #[error("Cannot reach AI provider at {url}: {source}")]
    ConnectionFailed {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("AI provider returned error {status}: {body}")]
    ProviderError { status: u16, body: String },

    #[error("AI response was empty")]
    EmptyResponse,

    #[error("Failed to parse AI response: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Request timed out after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("API key is missing for provider {provider}")]
    MissingApiKey { provider: String },

    #[error("Context too long: {tokens} tokens (max {max})")]
    ContextTooLong { tokens: usize, max: usize },

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

pub type AiResult<T> = Result<T, AiError>;
