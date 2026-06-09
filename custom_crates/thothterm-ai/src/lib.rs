pub mod error;
pub mod types;

use error::{AiError, AiResult};
use thothterm_config::AiProvider;
use types::*;
use reqwest::Client;
use std::time::Duration;
use tracing::debug;

const REQUEST_TIMEOUT_SECS: u64 = 30;
const MAX_CONTEXT_TOKENS: usize = 4096;

// ── Main AI client ────────────────────────────────────────────────────────────

pub struct AiClient {
    config: thothterm_config::AiConfig,
    http: Client,
}

impl AiClient {
    pub fn new(config: thothterm_config::AiConfig) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .expect("Failed to build HTTP client");

        Self { config, http }
    }

    /// Send a request to the configured AI provider.
    pub async fn ask(&self, request: AiRequest) -> AiResult<AiResponse> {
        if !self.config.enabled {
            return Err(AiError::Disabled);
        }
        if self.config.privacy_mode {
            return Err(AiError::PrivacyMode);
        }

        let system_prompt = build_system_prompt(&request.request_type, &request.context);
        let full_prompt = build_full_prompt(&request.prompt, &request.context);

        debug!("AI request type: {:?}", request.request_type);

        match self.config.provider {
            AiProvider::Ollama => {
                self.ask_ollama(&full_prompt, &system_prompt, &self.config.model)
                    .await
            }
            AiProvider::OpenAi | AiProvider::Custom => {
                if self.config.api_key.is_empty() {
                    return Err(AiError::MissingApiKey {
                        provider: "openai".into(),
                    });
                }
                self.ask_openai(&full_prompt, &system_prompt, &self.config.model)
                    .await
            }
            AiProvider::Claude => {
                if self.config.api_key.is_empty() {
                    return Err(AiError::MissingApiKey {
                        provider: "claude".into(),
                    });
                }
                self.ask_claude(&full_prompt, &system_prompt, &self.config.model)
                    .await
            }
        }
    }

    /// Check if the AI provider is reachable.
    pub async fn health_check(&self) -> AiResult<bool> {
        if self.config.privacy_mode {
            return Err(AiError::PrivacyMode);
        }

        let url = match self.config.provider {
            AiProvider::Ollama => format!("{}/api/tags", self.config.base_url),
            _ => format!("{}/v1/models", self.config.base_url),
        };

        let response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| AiError::ConnectionFailed {
                url: url.clone(),
                source: e,
            })?;

        Ok(response.status().is_success())
    }

    // ── Provider implementations ──────────────────────────────────────────────

    async fn ask_ollama(
        &self,
        prompt: &str,
        system: &str,
        model: &str,
    ) -> AiResult<AiResponse> {
        let url = format!("{}/api/generate", self.config.base_url);

        let body = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            system: Some(system.to_string()),
            options: OllamaOptions {
                num_predict: 512,
                temperature: 0.3,
            },
        };

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::ConnectionFailed {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(AiError::ProviderError {
                status,
                body: body_text,
            });
        }

        let parsed: OllamaResponse = resp.json().await?;

        if parsed.response.is_empty() {
            return Err(AiError::EmptyResponse);
        }

        Ok(AiResponse {
            suggested_command: extract_command(&parsed.response),
            text: parsed.response,
            model: model.to_string(),
            tokens_used: parsed.eval_count,
        })
    }

    async fn ask_openai(
        &self,
        prompt: &str,
        system: &str,
        model: &str,
    ) -> AiResult<AiResponse> {
        let url = format!("{}/v1/chat/completions", self.config.base_url);

        let body = OpenAiRequest {
            model: model.to_string(),
            messages: vec![
                OpenAiMessage {
                    role: "system".into(),
                    content: system.to_string(),
                },
                OpenAiMessage {
                    role: "user".into(),
                    content: prompt.to_string(),
                },
            ],
            max_tokens: 512,
            temperature: 0.3,
        };

        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::ConnectionFailed {
                url: url.clone(),
                source: e,
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(AiError::ProviderError {
                status,
                body: body_text,
            });
        }

        let parsed: OpenAiResponse = resp.json().await?;

        let text = parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .filter(|s| !s.is_empty())
            .ok_or(AiError::EmptyResponse)?;

        Ok(AiResponse {
            suggested_command: extract_command(&text),
            text,
            model: model.to_string(),
            tokens_used: parsed.usage.map(|u| u.total_tokens),
        })
    }

    async fn ask_claude(
        &self,
        prompt: &str,
        system: &str,
        model: &str,
    ) -> AiResult<AiResponse> {
        let base = if self.config.base_url.is_empty() {
            "https://api.anthropic.com"
        } else {
            &self.config.base_url
        };
        let url = format!("{}/v1/messages", base);

        let body = AnthropicRequest {
            model: model.to_string(),
            max_tokens: 512,
            system: system.to_string(),
            messages: vec![AnthropicMessage {
                role: "user".into(),
                content: prompt.to_string(),
            }],
        };

        let resp = self
            .http
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::ConnectionFailed { url: url.clone(), source: e })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(AiError::ProviderError { status, body: body_text });
        }

        let parsed: AnthropicResponse = resp.json().await?;
        let text = parsed
            .content
            .into_iter()
            .find(|c| c.content_type == "text")
            .and_then(|c| c.text)
            .filter(|s| !s.is_empty())
            .ok_or(AiError::EmptyResponse)?;

        Ok(AiResponse {
            suggested_command: extract_command(&text),
            text,
            model: model.to_string(),
            tokens_used: parsed.usage.map(|u| u.input_tokens + u.output_tokens),
        })
    }
}

// ── Prompt builders ───────────────────────────────────────────────────────────

fn build_system_prompt(request_type: &RequestType, ctx: &AiContext) -> String {
    let base = format!(
        "You are an AI assistant embedded in ThothTerm, a terminal emulator.\
        \nOS: {}, Shell: {}, Directory: {}",
        ctx.os, ctx.shell, ctx.cwd
    );

    let task = match request_type {
        RequestType::ExplainError => {
            "The user encountered a terminal error. Explain what went wrong in 2-3 sentences \
            and provide the exact command to fix it. Format: Explanation first, then \
            'Fix: `command here`'."
        }
        RequestType::NaturalLanguageToShell => {
            "Translate the user's natural language request into a shell command. \
            Reply with ONLY the command, no explanation unless asked. \
            Wrap the command in backticks."
        }
        RequestType::CommandSuggestion => {
            "Complete the partial command the user is typing. \
            Reply with only the most likely completion. No explanation."
        }
        RequestType::SummarizeOutput => {
            "Summarize the terminal output. Be concise (3-5 bullet points max). \
            Highlight errors, warnings, and key results."
        }
        RequestType::FreeForm => {
            "Answer the user's question concisely. \
            If your answer includes shell commands, wrap them in backticks."
        }
    };

    format!("{}\nTask: {}", base, task)
}

fn build_full_prompt(user_prompt: &str, ctx: &AiContext) -> String {
    let mut parts = Vec::new();

    if !ctx.recent_commands.is_empty() {
        parts.push(format!(
            "Recent commands:\n{}",
            ctx.recent_commands.join("\n")
        ));
    }

    if let Some(err) = &ctx.last_error {
        parts.push(format!("Last error output:\n{}", err));
    }

    if let Some(code) = ctx.last_exit_code {
        if code != 0 {
            parts.push(format!("Exit code: {}", code));
        }
    }

    if !parts.is_empty() {
        format!("{}\n\nUser: {}", parts.join("\n\n"), user_prompt)
    } else {
        user_prompt.to_string()
    }
}

/// Extract a shell command from backtick-wrapped text like `command here`.
fn extract_command(text: &str) -> Option<String> {
    let start = text.find('`')?;
    let after = &text[start + 1..];
    let end = after.find('`')?;
    let cmd = after[..end].trim().to_string();
    if cmd.is_empty() {
        None
    } else {
        Some(cmd)
    }
}
