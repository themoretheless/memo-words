use crate::command::AppCommand;
use crate::source::{SourceHealth, SourceStatus};
use muda::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use std::sync::mpsc::{self, Receiver};
use tray_icon::Icon as TrayIcon;

#[derive(Clone)]
pub struct TrayMenu {
    status: MenuItem,
    pause: MenuItem,
    reload: MenuItem,
    command_ids: CommandIds,
}

pub struct TrayMenuBuild {
    pub controls: TrayMenu,
    pub menu: Menu,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayFeedback {
    DiagnosticsCopied,
}

#[derive(Clone, Copy)]
enum TraySource<'a> {
    Benchmark,
    Runtime {
        status: &'a SourceStatus,
        can_reload: bool,
        retry_recommended: bool,
    },
}

#[derive(Clone, Copy)]
pub struct TrayState<'a> {
    paused: bool,
    source: TraySource<'a>,
    feedback: Option<TrayFeedback>,
}

impl<'a> TrayState<'a> {
    pub fn benchmark(paused: bool, feedback: Option<TrayFeedback>) -> Self {
        Self {
            paused,
            source: TraySource::Benchmark,
            feedback,
        }
    }

    pub fn runtime(
        paused: bool,
        status: &'a SourceStatus,
        can_reload: bool,
        retry_recommended: bool,
        feedback: Option<TrayFeedback>,
    ) -> Self {
        Self {
            paused,
            source: TraySource::Runtime {
                status,
                can_reload,
                retry_recommended,
            },
            feedback,
        }
    }
}

#[derive(Clone)]
struct CommandIds {
    next: MenuId,
    pause: MenuId,
    reload: MenuId,
    diagnostics: MenuId,
    quit: MenuId,
}

impl CommandIds {
    fn resolve(&self, id: &MenuId) -> Option<AppCommand> {
        if id == &self.next {
            Some(AppCommand::NextWord)
        } else if id == &self.pause {
            Some(AppCommand::TogglePause)
        } else if id == &self.reload {
            Some(AppCommand::ReloadSource)
        } else if id == &self.diagnostics {
            Some(AppCommand::CopyDiagnostics)
        } else if id == &self.quit {
            Some(AppCommand::Quit)
        } else {
            None
        }
    }
}

impl TrayMenu {
    pub fn build(benchmark: bool) -> TrayMenuBuild {
        let status = MenuItem::new(
            if benchmark {
                "Source: Benchmark (1 word)"
            } else {
                "Source: Loading..."
            },
            false,
            None,
        );
        let next = MenuItem::new("Next word", true, None);
        let pause = MenuItem::new("Pause", true, None);
        let reload = MenuItem::new("Reload words", false, None);
        let diagnostics = MenuItem::new("Copy diagnostics", true, None);
        let quit = MenuItem::new("Quit", true, None);
        let separator_1 = PredefinedMenuItem::separator();
        let separator_2 = PredefinedMenuItem::separator();
        let separator_3 = PredefinedMenuItem::separator();
        let menu = Menu::new();
        let warning = menu
            .append_items(&[
                &status,
                &separator_1,
                &next,
                &pause,
                &separator_2,
                &reload,
                &diagnostics,
                &separator_3,
                &quit,
            ])
            .err()
            .map(|error| error.to_string());
        let command_ids = CommandIds {
            next: next.id().clone(),
            pause: pause.id().clone(),
            reload: reload.id().clone(),
            diagnostics: diagnostics.id().clone(),
            quit: quit.id().clone(),
        };

        TrayMenuBuild {
            controls: Self {
                status,
                pause,
                reload,
                command_ids,
            },
            menu,
            warning,
        }
    }

    pub fn command_receiver<F>(&self, mut wake: F) -> Receiver<AppCommand>
    where
        F: FnMut() + Send + 'static,
    {
        let command_ids = self.command_ids.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let events = MenuEvent::receiver();
            while let Ok(event) = events.recv() {
                let Some(command) = command_ids.resolve(event.id()) else {
                    continue;
                };
                if tx.send(command).is_err() {
                    break;
                }
                wake();
            }
        });
        rx
    }

    pub fn sync(&self, state: TrayState<'_>) {
        self.status.set_text(status_label(state));
        self.pause.set_text(pause_label(state.paused));
        match state.source {
            TraySource::Benchmark => {
                self.reload.set_text("Reload words");
                self.reload.set_enabled(false);
            }
            TraySource::Runtime {
                can_reload,
                retry_recommended,
                ..
            } => {
                self.reload.set_text(reload_label(retry_recommended));
                self.reload.set_enabled(can_reload);
            }
        }
    }
}

fn status_label(state: TrayState<'_>) -> String {
    if state.feedback == Some(TrayFeedback::DiagnosticsCopied) {
        return "Diagnostics copied".to_string();
    }
    match state.source {
        TraySource::Benchmark => "Source: Benchmark (1 word)".to_string(),
        TraySource::Runtime { status, .. } => source_label(status),
    }
}

fn pause_label(paused: bool) -> &'static str {
    if paused { "Resume" } else { "Pause" }
}

fn reload_label(retry_recommended: bool) -> &'static str {
    if retry_recommended {
        "Retry source"
    } else {
        "Reload words"
    }
}

fn source_label(status: &SourceStatus) -> String {
    if let Some(pending) = status.pending {
        return match status.health {
            SourceHealth::Normal => format!(
                "Source: {} ready ({})",
                short_source(pending.kind),
                word_count(pending.words)
            ),
            SourceHealth::Degraded => format!(
                "Source: {} partial ({})",
                short_source(pending.kind),
                word_count(pending.words)
            ),
            _ => status_health_label(status),
        };
    }
    status_health_label(status)
}

fn status_health_label(status: &SourceStatus) -> String {
    match status.health {
        SourceHealth::Loading => "Source: Loading...".to_string(),
        SourceHealth::Retrying => "Source: Retrying...".to_string(),
        SourceHealth::Normal => format!(
            "Source: {} ({})",
            short_source(status.active.kind),
            word_count(status.active.words)
        ),
        SourceHealth::Degraded => {
            if status.active.kind == crate::source::SourceKind::Fallback {
                format!("Source: Fallback ({})", word_count(status.active.words))
            } else {
                format!(
                    "Source: {} ({}, degraded)",
                    short_source(status.active.kind),
                    word_count(status.active.words)
                )
            }
        }
        SourceHealth::Failed => {
            format!("Source: {} (load failed)", short_source(status.active.kind))
        }
    }
}

fn short_source(source: crate::source::SourceKind) -> &'static str {
    match source {
        crate::source::SourceKind::Mongo => "MongoDB",
        crate::source::SourceKind::Static => "Static",
        crate::source::SourceKind::Fallback => "Fallback",
    }
}

fn word_count(count: usize) -> String {
    if count == 1 {
        "1 word".to_string()
    } else {
        format!("{count} words")
    }
}

/// Build the procedural "W" tray icon. Returns `None` if the icon resource can't
/// be created, so the caller can run without it rather than crash (the tray is a
/// convenience, not a requirement).
pub fn create_icon() -> Option<TrayIcon> {
    let (w, h) = (22u32, 22u32);
    let mut rgba = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let (fx, fy) = (x as f32 / w as f32, y as f32 / h as f32);

            // "W" letter shape
            let visible = (0.2..0.8).contains(&fy)
                && ((fx - 0.15).abs() < 0.08
                    || (fx - 0.85).abs() < 0.08
                    || ((fx - 0.35).abs() < 0.08 && fy > 0.5)
                    || ((fx - 0.65).abs() < 0.08 && fy > 0.5)
                    || (fy > 0.7 && (0.15..0.35).contains(&fx))
                    || (fy > 0.7 && (0.65..0.85).contains(&fx))
                    || ((0.6..0.7).contains(&fy) && (0.35..0.65).contains(&fx)));

            if visible {
                rgba[i..i + 4].copy_from_slice(&[30, 130, 230, 255]);
            }
        }
    }

    TrayIcon::from_rgba(rgba, w, h).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{DeckSource, SourceKind};

    fn status(health: SourceHealth) -> SourceStatus {
        SourceStatus {
            health,
            active: DeckSource {
                kind: SourceKind::Fallback,
                words: 40,
            },
            pending: None,
            attempt_id: 1,
        }
    }

    #[test]
    fn source_labels_are_compact_and_describe_pending_activation() {
        assert_eq!(
            source_label(&status(SourceHealth::Loading)),
            "Source: Loading..."
        );
        assert_eq!(
            source_label(&status(SourceHealth::Degraded)),
            "Source: Fallback (40 words)"
        );
        let mut ready = status(SourceHealth::Normal);
        ready.pending = Some(DeckSource {
            kind: SourceKind::Mongo,
            words: 2,
        });
        assert_eq!(source_label(&ready), "Source: MongoDB ready (2 words)");
    }

    #[test]
    fn failed_label_keeps_the_effective_source_visible() {
        let mut failed = status(SourceHealth::Failed);
        failed.active = DeckSource {
            kind: SourceKind::Mongo,
            words: 2,
        };
        assert_eq!(source_label(&failed), "Source: MongoDB (load failed)");
    }

    #[test]
    fn action_labels_always_name_the_next_effect() {
        assert_eq!(pause_label(false), "Pause");
        assert_eq!(pause_label(true), "Resume");
        assert_eq!(reload_label(false), "Reload words");
        assert_eq!(reload_label(true), "Retry source");
    }

    #[test]
    fn external_menu_ids_resolve_to_domain_commands() {
        let ids = CommandIds {
            next: MenuId::from("next"),
            pause: MenuId::from("pause"),
            reload: MenuId::from("reload"),
            diagnostics: MenuId::from("diagnostics"),
            quit: MenuId::from("quit"),
        };

        assert_eq!(
            ids.resolve(&MenuId::from("next")),
            Some(AppCommand::NextWord)
        );
        assert_eq!(
            ids.resolve(&MenuId::from("pause")),
            Some(AppCommand::TogglePause)
        );
        assert_eq!(
            ids.resolve(&MenuId::from("reload")),
            Some(AppCommand::ReloadSource)
        );
        assert_eq!(
            ids.resolve(&MenuId::from("diagnostics")),
            Some(AppCommand::CopyDiagnostics)
        );
        assert_eq!(ids.resolve(&MenuId::from("quit")), Some(AppCommand::Quit));
        assert_eq!(ids.resolve(&MenuId::from("unknown")), None);
    }
}
