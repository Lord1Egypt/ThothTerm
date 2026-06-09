use mux::termwiztermtab::TermWizTerminal;
use std::sync::{Arc, Mutex};
use termwiz::cell::{AttributeChange, Intensity};
use termwiz::color::{AnsiColor, ColorAttribute};
use termwiz::input::{InputEvent, KeyCode, KeyEvent, Modifiers};
use termwiz::lineedit::*;
use termwiz::surface::Change;
use termwiz::terminal::Terminal;
use thothterm_config::ThothConfig;
use thothterm_web3::Web3Client;
use thothterm_web3::detector::detect_project;
use thothterm_web3::wallet::WalletStore;

lazy_static::lazy_static! {
    pub static ref GAS_CACHE: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}

pub fn cached_gas_status() -> Option<String> {
    GAS_CACHE.lock().ok()?.clone()
}

struct Web3State {
    rpc_url: String,
    chain_id: Option<String>,
    block: Option<String>,
    gas: Option<String>,
    project: Option<String>,
    wallet: Option<String>,
    ens_input: String,
    ens_result: Option<String>,
    error: Option<String>,
    loading: bool,
}

impl Web3State {
    fn empty(rpc_url: String) -> Self {
        Self {
            rpc_url,
            chain_id: None,
            block: None,
            gas: None,
            project: None,
            wallet: None,
            ens_input: String::new(),
            ens_result: None,
            error: None,
            loading: true,
        }
    }
}

fn fetch_web3_state(rpc_url: &str) -> Web3State {
    let client = Web3Client::new(rpc_url);

    let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(e) => {
            return Web3State {
                rpc_url: rpc_url.to_string(),
                error: Some(format!("Runtime error: {}", e)),
                loading: false,
                ..Web3State::empty(rpc_url.to_string())
            };
        }
    };

    let chain_id = rt.block_on(client.chain_id()).ok().map(|id| format!("{}", id));
    let block = rt.block_on(client.block_number()).ok().map(|n| format!("{}", n));
    let gas = rt.block_on(client.gas_info()).ok().map(|g| {
        if let Ok(mut cache) = GAS_CACHE.lock() {
            *cache = Some(g.display_label());
        }
        format!("{:.2} Gwei (est. ${:.4}/transfer)", g.base_fee_gwei, g.simple_transfer_usd)
    });
    let project = detect_project(std::path::Path::new(".")).map(|info| {
        format!(
            "{:?} ({})",
            info.project_type,
            info.rpc_url.as_deref().unwrap_or("no local RPC")
        )
    });

    let wallet = WalletStore::load(&WalletStore::default_path())
        .ok()
        .and_then(|store| store.active().map(|w| w.display_address()));

    Web3State {
        rpc_url: rpc_url.to_string(),
        chain_id,
        block,
        gas,
        project,
        wallet,
        ens_input: String::new(),
        ens_result: None,
        error: None,
        loading: false,
    }
}

fn push_row(changes: &mut Vec<Change>, label: &str, value: &str, color: ColorAttribute) {
    changes.push(Change::Attribute(AttributeChange::Foreground(AnsiColor::Aqua.into())));
    changes.push(Change::Text(format!("  {:<22}", label)));
    changes.push(Change::Attribute(AttributeChange::Foreground(color)));
    changes.push(Change::Text(format!("{}\r\n", value)));
    changes.push(Change::Attribute(AttributeChange::Foreground(ColorAttribute::Default)));
}

fn render(term: &mut TermWizTerminal, state: &Web3State) -> anyhow::Result<()> {
    let mut changes = vec![];
    changes.push(Change::ClearScreen(ColorAttribute::Default));

    let header = " 𓆣 ThothTerm Web3  [R] Refresh  [E] ENS Lookup  [q/Esc] Close ";
    changes.push(Change::Attribute(AttributeChange::Intensity(Intensity::Bold)));
    changes.push(Change::Attribute(AttributeChange::Foreground(AnsiColor::Yellow.into())));
    changes.push(Change::Text(header.to_string()));
    changes.push(Change::Attribute(AttributeChange::Intensity(Intensity::Normal)));
    changes.push(Change::Attribute(AttributeChange::Foreground(ColorAttribute::Default)));
    changes.push(Change::Text("\r\n".to_string()));
    changes.push(Change::Text("─".repeat(header.len()) + "\r\n\r\n"));

    push_row(&mut changes, "RPC URL:", &state.rpc_url, ColorAttribute::Default);

    if state.loading {
        changes.push(Change::Attribute(AttributeChange::Foreground(AnsiColor::Grey.into())));
        changes.push(Change::Text("\r\n  Fetching Web3 data...\r\n".to_string()));
        changes.push(Change::Attribute(AttributeChange::Foreground(ColorAttribute::Default)));
    } else if let Some(err) = &state.error {
        push_row(&mut changes, "Error:", err, AnsiColor::Red.into());
    } else {
        push_row(&mut changes, "Chain ID:", state.chain_id.as_deref().unwrap_or("unknown"), AnsiColor::Green.into());
        push_row(&mut changes, "Latest Block:", state.block.as_deref().unwrap_or("unknown"), ColorAttribute::Default);
        push_row(&mut changes, "Gas Price:", state.gas.as_deref().unwrap_or("unavailable"), AnsiColor::Yellow.into());
        if let Some(proj) = &state.project {
            push_row(&mut changes, "Project:", proj, AnsiColor::Aqua.into());
        }
        if let Some(wallet) = &state.wallet {
            push_row(&mut changes, "Active Wallet:", wallet, AnsiColor::Fuchsia.into());
        }
        if !state.ens_input.is_empty() {
            changes.push(Change::Text("\r\n".to_string()));
            push_row(&mut changes, "ENS Query:", &state.ens_input, ColorAttribute::Default);
            if let Some(result) = &state.ens_result {
                push_row(&mut changes, "  \u{2192} Address:", result, AnsiColor::Green.into());
            }
        }
    }

    term.render(&changes)?;
    Ok(())
}

pub fn show_web3_overlay(mut term: TermWizTerminal) -> anyhow::Result<()> {
    let rpc_url = ThothConfig::load()
        .ok()
        .filter(|c| c.web3.enabled && !c.web3.rpc_url.is_empty())
        .map(|c| c.web3.rpc_url)
        .unwrap_or_else(|| "http://localhost:8545".into());

    let mut state = Web3State::empty(rpc_url.clone());

    term.set_raw_mode()?;
    render(&mut term, &state)?;

    state = fetch_web3_state(&rpc_url);
    render(&mut term, &state)?;

    let mut host = NopLineEditorHost::default();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    loop {
        match term.poll_input(None) {
            Ok(Some(InputEvent::Key(KeyEvent { key, modifiers }))) => {
                match (key, modifiers) {
                    (KeyCode::Char('q'), _)
                    | (KeyCode::Escape, _)
                    | (KeyCode::Char('c'), Modifiers::CTRL) => break,
                    (KeyCode::Char('r'), _) | (KeyCode::Char('R'), _) => {
                        state = fetch_web3_state(&rpc_url);
                        render(&mut term, &state)?;
                    }
                    (KeyCode::Char('e'), _) | (KeyCode::Char('E'), _) => {
                        let ens_name = {
                            let mut editor = LineEditor::new(&mut term);
                            editor.set_prompt("ENS name (e.g. vitalik.eth): ");
                            editor.read_line(&mut host).ok().flatten()
                        };
                        if let Some(name) = ens_name {
                            let name = name.trim().to_string();
                            if !name.is_empty() {
                                state.ens_input = name.clone();
                                state.ens_result = Some("Resolving...".into());
                                render(&mut term, &state)?;

                                let client = Web3Client::new(&rpc_url);
                                state.ens_result = Some(match rt.block_on(client.resolve_ens(&name)) {
                                    Ok(addr) => addr,
                                    Err(e) => format!("Error: {}", e),
                                });
                                render(&mut term, &state)?;
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Some(_)) => {}
            Ok(None) => {}
            Err(e) => {
                log::error!("web3 overlay error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
