use config::ConfigHandle;
use mux::termwiztermtab::TermWizTerminal;
use std::io::Write;
use termwiz::cell::{AttributeChange, CellAttributes, Intensity};
use termwiz::color::{AnsiColor, ColorAttribute};
use termwiz::input::{InputEvent, KeyCode, KeyEvent, Modifiers};
use termwiz::surface::Change;
use termwiz::terminal::Terminal;

struct SettingsState {
    selected: usize,
    sections: Vec<(&'static str, Vec<SettingEntry>)>,
    dirty: bool,
}

struct SettingEntry {
    label: &'static str,
    value: String,
    editable: bool,
}

impl SettingsState {
    fn build(config: &ConfigHandle) -> Self {
        let font_size = format!("{:.1}", config.font_size);
        let opacity = format!("{:.2}", config.window_background_opacity);
        let color_scheme = config.color_scheme.as_deref().unwrap_or("(default)").to_string();
        let default_prog = config
            .default_prog
            .as_ref()
            .map(|v| v.join(" "))
            .unwrap_or_else(|| "(system shell)".into());
        let window_close_confirmation = format!("{:?}", config.window_close_confirmation);

        Self {
            selected: 0,
            dirty: false,
            sections: vec![
                (
                    "Appearance",
                    vec![
                        SettingEntry { label: "Font Size", value: font_size, editable: false },
                        SettingEntry { label: "Window Opacity", value: opacity, editable: false },
                        SettingEntry { label: "Color Scheme", value: color_scheme, editable: false },
                    ],
                ),
                (
                    "Behavior",
                    vec![
                        SettingEntry {
                            label: "Default Shell",
                            value: default_prog,
                            editable: false,
                        },
                        SettingEntry {
                            label: "Close Confirmation",
                            value: window_close_confirmation,
                            editable: false,
                        },
                    ],
                ),
                (
                    "About",
                    vec![
                        SettingEntry {
                            label: "Version",
                            value: env!("CARGO_PKG_VERSION").to_string(),
                            editable: false,
                        },
                        SettingEntry {
                            label: "Config file",
                            value: config::CONFIG_FILE_OVERRIDE.lock().unwrap()
                                .as_ref()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "~/.config/thothterm/thothterm.lua".into()),
                            editable: false,
                        },
                    ],
                ),
            ],
        }
    }

    fn total_rows(&self) -> usize {
        self.sections.iter().map(|(_, entries)| entries.len() + 1).sum()
    }

    fn selected_clamp(&mut self) {
        let max = self.total_rows().saturating_sub(1);
        if self.selected > max {
            self.selected = max;
        }
    }
}

fn render(term: &mut TermWizTerminal, state: &SettingsState) -> anyhow::Result<()> {
    let mut changes = vec![];

    changes.push(Change::ClearScreen(ColorAttribute::Default));

    let header = " 𓆣 ThothTerm Settings  [↑↓] Navigate  [q/Esc] Close ";
    changes.push(Change::Attribute(AttributeChange::Intensity(Intensity::Bold)));
    changes.push(Change::Attribute(AttributeChange::Foreground(
        AnsiColor::Yellow.into(),
    )));
    changes.push(Change::Text(header.to_string()));
    changes.push(Change::Attribute(AttributeChange::Intensity(Intensity::Normal)));
    changes.push(Change::Attribute(AttributeChange::Foreground(
        ColorAttribute::Default,
    )));
    changes.push(Change::Text("\r\n".to_string()));
    changes.push(Change::Text(
        "─".repeat(header.len()).to_string() + "\r\n",
    ));

    let mut row_idx = 0usize;
    for (section_name, entries) in &state.sections {
        // Section header
        changes.push(Change::Text("\r\n".to_string()));
        changes.push(Change::Attribute(AttributeChange::Intensity(Intensity::Bold)));
        changes.push(Change::Attribute(AttributeChange::Foreground(
            AnsiColor::Cyan.into(),
        )));
        changes.push(Change::Text(format!("  {}\r\n", section_name)));
        changes.push(Change::Attribute(AttributeChange::Intensity(Intensity::Normal)));
        changes.push(Change::Attribute(AttributeChange::Foreground(
            ColorAttribute::Default,
        )));
        row_idx += 1;

        for entry in entries {
            let is_selected = row_idx == state.selected + 1; // +1 to skip section headers
            if is_selected {
                changes.push(Change::Attribute(AttributeChange::Background(
                    AnsiColor::Navy.into(),
                )));
                changes.push(Change::Attribute(AttributeChange::Foreground(
                    AnsiColor::White.into(),
                )));
            }
            let line = format!("    {:<30}  {}\r\n", entry.label, entry.value);
            changes.push(Change::Text(line));
            if is_selected {
                changes.push(Change::Attribute(AttributeChange::Background(
                    ColorAttribute::Default,
                )));
                changes.push(Change::Attribute(AttributeChange::Foreground(
                    ColorAttribute::Default,
                )));
            }
            row_idx += 1;
        }
    }

    changes.push(Change::Text("\r\n".to_string()));
    changes.push(Change::Attribute(AttributeChange::Foreground(
        AnsiColor::Grey.into(),
    )));
    changes.push(Change::Text(
        "  Edit settings in your config file and restart to apply changes.\r\n".to_string(),
    ));
    changes.push(Change::Attribute(AttributeChange::Foreground(
        ColorAttribute::Default,
    )));

    term.render(&changes)?;
    Ok(())
}

pub fn show_settings_overlay(
    mut term: TermWizTerminal,
    config: ConfigHandle,
) -> anyhow::Result<()> {
    let mut state = SettingsState::build(&config);

    term.set_raw_mode()?;
    render(&mut term, &state)?;

    loop {
        match term.poll_input(None) {
            Ok(Some(InputEvent::Key(KeyEvent { key, modifiers }))) => {
                match (key, modifiers) {
                    (KeyCode::Char('q'), _)
                    | (KeyCode::Escape, _)
                    | (KeyCode::Char('c'), Modifiers::CTRL) => {
                        break;
                    }
                    (KeyCode::UpArrow, _) | (KeyCode::Char('k'), _) => {
                        if state.selected > 0 {
                            state.selected -= 1;
                        }
                    }
                    (KeyCode::DownArrow, _) | (KeyCode::Char('j'), _) => {
                        let max = state.total_rows().saturating_sub(1);
                        if state.selected < max {
                            state.selected += 1;
                        }
                    }
                    _ => {}
                }
                state.selected_clamp();
                render(&mut term, &state)?;
            }
            Ok(Some(_)) => {}
            Ok(None) => {}
            Err(e) => {
                log::error!("settings overlay input error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
