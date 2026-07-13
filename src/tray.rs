use crate::source::{SourceHealth, SourceStatus};
use muda::MenuItem;
use tray_icon::Icon as TrayIcon;

#[derive(Clone)]
pub struct SourceMenu {
    pub status: MenuItem,
    pub reload: MenuItem,
    pub diagnostics: MenuItem,
}

impl SourceMenu {
    pub fn new(benchmark: bool) -> Self {
        Self {
            status: MenuItem::new(
                if benchmark {
                    "Source: Benchmark (1 word)"
                } else {
                    "Source: Loading..."
                },
                false,
                None,
            ),
            reload: MenuItem::new("Reload words", false, None),
            diagnostics: MenuItem::new("Copy diagnostics", true, None),
        }
    }

    pub fn sync(&self, status: &SourceStatus, can_reload: bool) {
        self.status.set_text(source_label(status));
        self.reload.set_enabled(can_reload);
    }

    pub fn show_copied(&self) {
        self.status.set_text("Diagnostics copied");
    }

    pub fn sync_benchmark(&self) {
        self.status.set_text("Source: Benchmark (1 word)");
        self.reload.set_enabled(false);
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
}
