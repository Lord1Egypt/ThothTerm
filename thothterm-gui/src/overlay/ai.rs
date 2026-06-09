use mux::termwiztermtab::TermWizTerminal;
use std::sync::atomic::{AtomicBool, Ordering};
use termwiz::cell::{AttributeChange, Intensity};
use termwiz::color::{AnsiColor, ColorAttribute};
use termwiz::lineedit::*;
use termwiz::surface::Change;
use termwiz::terminal::Terminal;
use thothterm_ai::types::{AiContext, AiRequest, RequestType};
use thothterm_ai::AiClient;
use thothterm_config::{AiConfig, AiProvider, ThothConfig};

/// True while an AI inference request is in flight.
pub static AI_INFERRING: AtomicBool = AtomicBool::new(false);

pub fn is_ai_inferring() -> bool {
    AI_INFERRING.load(Ordering::Relaxed)
}

struct InferGuard;
impl InferGuard {
    fn start() -> Self {
        AI_INFERRING.store(true, Ordering::Relaxed);
        Self
    }
}
impl Drop for InferGuard {
    fn drop(&mut self) {
        AI_INFERRING.store(false, Ordering::Relaxed);
    }
}

struct AiHistory {
    entries: Vec<(String, String)>,
    host: NopLineEditorHost,
}

impl AiHistory {
    fn new() -> Self {
        Self { entries: Vec::new(), host: NopLineEditorHost::default() }
    }
}

fn render_conversation(term: &mut TermWizTerminal, ai: &AiHistory, status: &str) -> anyhow::Result<()> {
    let mut changes = vec![];
    changes.push(Change::ClearScreen(ColorAttribute::Default));

    let header = " 𓆣 ThothTerm AI  [Ctrl+C/Esc] Close ";
    changes.push(Change::Attribute(AttributeChange::Intensity(Intensity::Bold)));
    changes.push(Change::Attribute(AttributeChange::Foreground(AnsiColor::Yellow.into())));
    changes.push(Change::Text(header.to_string()));
    changes.push(Change::Attribute(AttributeChange::Intensity(Intensity::Normal)));
    changes.push(Change::Attribute(AttributeChange::Foreground(ColorAttribute::Default)));
    changes.push(Change::Text("\r\n".to_string()));
    changes.push(Change::Text("─".repeat(header.len()) + "\r\n\r\n"));

    for (prompt, response) in &ai.entries {
        changes.push(Change::Attribute(AttributeChange::Foreground(AnsiColor::Aqua.into())));
        changes.push(Change::Text(format!("You: {}\r\n", prompt)));
        changes.push(Change::Attribute(AttributeChange::Foreground(AnsiColor::White.into())));
        for line in response.lines() {
            changes.push(Change::Text(format!("  {}\r\n", line)));
        }
        changes.push(Change::Attribute(AttributeChange::Foreground(ColorAttribute::Default)));
        changes.push(Change::Text("\r\n".to_string()));
    }

    if !status.is_empty() {
        changes.push(Change::Attribute(AttributeChange::Foreground(AnsiColor::Grey.into())));
        changes.push(Change::Text(format!("  {}\r\n\r\n", status)));
        changes.push(Change::Attribute(AttributeChange::Foreground(ColorAttribute::Default)));
    }

    term.render(&changes)?;
    Ok(())
}

pub fn show_ai_overlay(mut term: TermWizTerminal) -> anyhow::Result<()> {
    let ai_config = ThothConfig::load()
        .ok()
        .map(|c| c.ai)
        .unwrap_or_else(|| AiConfig {
            enabled: true,
            provider: AiProvider::Ollama,
            model: "llama3".into(),
            base_url: "http://localhost:11434".into(),
            api_key: String::new(),
            privacy_mode: false,
            ..Default::default()
        });

    let ai_client = AiClient::new(ai_config);

    let mut ai_history = AiHistory::new();
    let mut status = String::new();

    term.set_raw_mode()?;
    render_conversation(&mut term, &ai_history, &status)?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    loop {
        status.clear();
        render_conversation(&mut term, &ai_history, &status)?;

        let input = {
            let mut editor = LineEditor::new(&mut term);
            editor.set_prompt("Ask: ");
            editor.read_line(&mut ai_history.host)
        };

        match input {
            Ok(Some(line)) if !line.trim().is_empty() => {
                let prompt_text = line.trim().to_string();

                status = "Thinking...".into();
                render_conversation(&mut term, &ai_history, &status)?;

                let request = AiRequest {
                    prompt: prompt_text.clone(),
                    context: AiContext {
                        cwd: std::env::current_dir()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default(),
                        shell: std::env::var("SHELL").unwrap_or_default(),
                        os: std::env::consts::OS.to_string(),
                        recent_commands: vec![],
                        last_error: None,
                        last_exit_code: None,
                    },
                    request_type: RequestType::FreeForm,
                };

                let _guard = InferGuard::start();
                let response = rt.block_on(ai_client.ask(request));
                drop(_guard);
                status.clear();

                match response {
                    Ok(resp) => {
                        ai_history.entries.push((prompt_text, resp.text));
                    }
                    Err(e) => {
                        ai_history.entries.push((prompt_text, format!("Error: {}", e)));
                    }
                }
            }
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => break,
        }
    }

    Ok(())
}

pub fn show_ai_error_overlay(mut term: TermWizTerminal, exit_code: i32) -> anyhow::Result<()> {
    let ai_config = ThothConfig::load()
        .ok()
        .map(|c| c.ai)
        .unwrap_or_else(|| AiConfig {
            enabled: true,
            provider: AiProvider::Ollama,
            model: "llama3".into(),
            base_url: "http://localhost:11434".into(),
            api_key: String::new(),
            privacy_mode: false,
            ..Default::default()
        });

    if !ai_config.enabled || ai_config.privacy_mode {
        return Ok(());
    }

    let ai_client = AiClient::new(ai_config);

    let initial_prompt = format!(
        "The last command failed with exit code {}. What might have gone wrong and how can I fix it?",
        exit_code
    );

    let request = AiRequest {
        prompt: initial_prompt.clone(),
        context: AiContext {
            cwd: std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            shell: std::env::var("SHELL").unwrap_or_default(),
            os: std::env::consts::OS.to_string(),
            recent_commands: vec![],
            last_error: None,
            last_exit_code: Some(exit_code),
        },
        request_type: RequestType::ExplainError,
    };

    let mut ai_history = AiHistory::new();
    term.set_raw_mode()?;

    let status = format!("Asking AI about exit code {}…", exit_code);
    render_conversation(&mut term, &ai_history, &status)?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let _guard = InferGuard::start();
    let response_text = match rt.block_on(ai_client.ask(request)) {
        Ok(resp) => resp.text,
        Err(e) => format!("AI unavailable: {}", e),
    };
    ai_history.entries.push((initial_prompt, response_text));

    loop {
        render_conversation(&mut term, &ai_history, "")?;

        let input = {
            let mut editor = LineEditor::new(&mut term);
            editor.set_prompt("Follow-up (Enter to close): ");
            editor.read_line(&mut ai_history.host)
        };

        match input {
            Ok(Some(line)) if !line.trim().is_empty() => {
                let prompt_text = line.trim().to_string();

                render_conversation(&mut term, &ai_history, "Thinking...")?;

                let followup = AiRequest {
                    prompt: prompt_text.clone(),
                    context: AiContext {
                        cwd: std::env::current_dir()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default(),
                        shell: std::env::var("SHELL").unwrap_or_default(),
                        os: std::env::consts::OS.to_string(),
                        recent_commands: vec![],
                        last_error: None,
                        last_exit_code: Some(exit_code),
                    },
                    request_type: RequestType::FreeForm,
                };

                let _guard2 = InferGuard::start();
                match rt.block_on(ai_client.ask(followup)) {
                    Ok(resp) => ai_history.entries.push((prompt_text, resp.text)),
                    Err(e) => ai_history.entries.push((prompt_text, format!("Error: {}", e))),
                }
            }
            Ok(Some(_)) | Ok(None) | Err(_) => break,
        }
    }

    Ok(())
}
