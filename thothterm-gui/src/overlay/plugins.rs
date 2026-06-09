use mux::termwiztermtab::TermWizTerminal;
use termwiz::cell::{AttributeChange, Intensity};
use termwiz::color::{AnsiColor, ColorAttribute};
use termwiz::input::{InputEvent, KeyCode, KeyEvent, Modifiers};
use termwiz::surface::Change;
use termwiz::terminal::Terminal;
use thothterm_plugins::{PluginListEntry, PluginManager};

struct PluginsState {
    manager: PluginManager,
    plugins: Vec<PluginListEntry>,
    selected: usize,
    message: Option<String>,
}

impl PluginsState {
    fn load() -> Self {
        let plugin_dir = PluginManager::default_dir();

        let mut manager = PluginManager::new(plugin_dir);
        let _ = manager.load_all();
        let plugins = manager.list();
        Self { manager, plugins, selected: 0, message: None }
    }

    fn selected_clamp(&mut self) {
        if !self.plugins.is_empty() && self.selected >= self.plugins.len() {
            self.selected = self.plugins.len() - 1;
        }
    }

    fn refresh(&mut self) {
        self.plugins = self.manager.list();
    }

    fn toggle_selected(&mut self) {
        if self.plugins.is_empty() {
            return;
        }
        let name = self.plugins[self.selected].name.clone();
        let enabled = self.plugins[self.selected].enabled;
        let result = if enabled {
            self.manager.disable(&name)
        } else {
            self.manager.enable(&name)
        };
        match result {
            Ok(()) => {
                self.message = Some(format!(
                    "{} {}",
                    name,
                    if !enabled { "enabled" } else { "disabled" }
                ));
                self.refresh();
            }
            Err(e) => {
                self.message = Some(format!("Error: {}", e));
            }
        }
    }

    fn remove_selected(&mut self) {
        if self.plugins.is_empty() {
            return;
        }
        let name = self.plugins[self.selected].name.clone();
        match self.manager.remove(&name) {
            Ok(()) => {
                self.message = Some(format!("{} removed", name));
                self.refresh();
                self.selected_clamp();
            }
            Err(e) => {
                self.message = Some(format!("Error: {}", e));
            }
        }
    }
}

fn render(term: &mut TermWizTerminal, state: &PluginsState) -> anyhow::Result<()> {
    let mut changes = vec![];
    changes.push(Change::ClearScreen(ColorAttribute::Default));

    let header = " 𓆣 ThothTerm Plugins  [↑↓] Navigate  [Space] Toggle  [D] Remove  [q] Close ";
    changes.push(Change::Attribute(AttributeChange::Intensity(Intensity::Bold)));
    changes.push(Change::Attribute(AttributeChange::Foreground(AnsiColor::Yellow.into())));
    changes.push(Change::Text(header.to_string()));
    changes.push(Change::Attribute(AttributeChange::Intensity(Intensity::Normal)));
    changes.push(Change::Attribute(AttributeChange::Foreground(ColorAttribute::Default)));
    changes.push(Change::Text("\r\n".to_string()));
    changes.push(Change::Text("─".repeat(header.len()) + "\r\n"));

    if state.plugins.is_empty() {
        changes.push(Change::Text("\r\n  No plugins installed.\r\n".to_string()));
        changes.push(Change::Text(
            "  Install: copy plugin folder into ~/.local/share/thothterm/plugins/\r\n".to_string(),
        ));
    } else {
        for (i, plugin) in state.plugins.iter().enumerate() {
            let is_selected = i == state.selected;
            if is_selected {
                changes.push(Change::Attribute(AttributeChange::Background(AnsiColor::Navy.into())));
                changes.push(Change::Attribute(AttributeChange::Foreground(AnsiColor::White.into())));
            } else {
                let status_color: ColorAttribute = if plugin.enabled {
                    AnsiColor::Green.into()
                } else {
                    AnsiColor::Grey.into()
                };
                changes.push(Change::Attribute(AttributeChange::Foreground(status_color)));
            }
            let status_icon = if plugin.enabled { "●" } else { "○" };
            let line = format!(
                "  {}  {:<30}  v{:<8}  {}\r\n",
                status_icon,
                plugin.name,
                plugin.version,
                plugin.description
            );
            changes.push(Change::Text(line));
            if is_selected {
                changes.push(Change::Attribute(AttributeChange::Background(ColorAttribute::Default)));
            }
            changes.push(Change::Attribute(AttributeChange::Foreground(ColorAttribute::Default)));
        }
    }

    if let Some(msg) = &state.message {
        changes.push(Change::Text("\r\n".to_string()));
        changes.push(Change::Attribute(AttributeChange::Foreground(AnsiColor::Aqua.into())));
        changes.push(Change::Text(format!("  {}\r\n", msg)));
        changes.push(Change::Attribute(AttributeChange::Foreground(ColorAttribute::Default)));
    }

    term.render(&changes)?;
    Ok(())
}

pub fn show_plugins_overlay(mut term: TermWizTerminal) -> anyhow::Result<()> {
    let mut state = PluginsState::load();

    term.set_raw_mode()?;
    render(&mut term, &state)?;

    loop {
        match term.poll_input(None) {
            Ok(Some(InputEvent::Key(KeyEvent { key, modifiers }))) => {
                match (key, modifiers) {
                    (KeyCode::Char('q'), _)
                    | (KeyCode::Escape, _)
                    | (KeyCode::Char('c'), Modifiers::CTRL) => break,
                    (KeyCode::UpArrow, _) | (KeyCode::Char('k'), _) => {
                        if state.selected > 0 {
                            state.selected -= 1;
                            state.message = None;
                        }
                    }
                    (KeyCode::DownArrow, _) | (KeyCode::Char('j'), _) => {
                        if !state.plugins.is_empty() && state.selected + 1 < state.plugins.len() {
                            state.selected += 1;
                            state.message = None;
                        }
                    }
                    (KeyCode::Char(' '), _) => state.toggle_selected(),
                    (KeyCode::Char('d'), _) | (KeyCode::Char('D'), _) => state.remove_selected(),
                    _ => {}
                }
                render(&mut term, &state)?;
            }
            Ok(Some(_)) => {}
            Ok(None) => {}
            Err(e) => {
                log::error!("plugin manager input error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
