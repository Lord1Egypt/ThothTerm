use serde::{Deserialize, Serialize};

/// A request to the AI with full context.
#[derive(Debug, Clone)]
pub struct AiRequest {
    /// The user's question or the context prompt.
    pub prompt: String,
    /// Recent terminal history (last N commands + output).
    pub context: AiContext,
    /// Type of request — affects system prompt.
    pub request_type: RequestType,
}

#[derive(Debug, Clone, Default)]
pub struct AiContext {
    /// Current working directory.
    pub cwd: String,
    /// Current shell (bash, zsh, fish, pwsh).
    pub shell: String,
    /// OS type.
    pub os: String,
    /// Last N commands typed by the user.
    pub recent_commands: Vec<String>,
    /// Last error output (if any).
    pub last_error: Option<String>,
    /// Last command exit code.
    pub last_exit_code: Option<i32>,
}

#[derive(Debug, Clone)]
pub enum RequestType {
    /// Explain an error and suggest a fix.
    ExplainError,
    /// Translate natural language to a shell command.
    NaturalLanguageToShell,
    /// Suggest completion for a partial command.
    CommandSuggestion,
    /// Summarize long terminal output.
    SummarizeOutput,
    /// Free-form question.
    FreeForm,
}

/// The AI's response.
#[derive(Debug, Clone)]
pub struct AiResponse {
    pub text: String,
    /// If the AI suggested a command, it's extracted here.
    pub suggested_command: Option<String>,
    /// Model that produced this response.
    pub model: String,
    /// Approximate tokens used.
    pub tokens_used: Option<usize>,
}

// ── Ollama API types ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub(crate) struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub options: OllamaOptions,
}

#[derive(Debug, Serialize)]
pub(crate) struct OllamaOptions {
    pub num_predict: i32,
    pub temperature: f32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OllamaResponse {
    pub response: String,
    pub done: bool,
    pub eval_count: Option<usize>,
}

// ── OpenAI API types ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub(crate) struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct OpenAiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OpenAiResponse {
    pub choices: Vec<OpenAiChoice>,
    pub usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OpenAiChoice {
    pub message: OpenAiMessage,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OpenAiUsage {
    pub total_tokens: usize,
}

// ── Anthropic Messages API types ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub(crate) struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    pub system: String,
    pub messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AnthropicResponse {
    pub content: Vec<AnthropicContent>,
    pub usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AnthropicContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AnthropicUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}
