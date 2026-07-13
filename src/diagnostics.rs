//! Redacted, copyable support diagnostics.

use crate::config::Config;
use crate::source::SourceController;
use std::fmt::Write;
use std::time::UNIX_EPOCH;

pub fn build(cfg: &Config, source: Option<&SourceController>) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "memo_words.version={}", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(output, "runtime.os={}", std::env::consts::OS);
    let _ = writeln!(output, "runtime.arch={}", std::env::consts::ARCH);
    let _ = writeln!(output, "config.interval_secs={}", cfg.timing.interval_secs);
    let _ = writeln!(output, "config.theme={:?}", cfg.appearance.theme);
    let _ = writeln!(output, "config.speak={}", cfg.learning.speak);
    let _ = writeln!(
        output,
        "config.reduce_motion={}",
        cfg.accessibility.reduce_motion
    );

    let Some(source) = source else {
        let _ = writeln!(output, "source.health=normal");
        let _ = writeln!(output, "source.active=static");
        return output;
    };

    let status = source.status();
    let _ = writeln!(output, "source.health={}", status.health);
    let _ = writeln!(output, "source.active={}", status.active.kind);
    let _ = writeln!(output, "source.active_words={}", status.active.words);
    let _ = writeln!(output, "source.attempt={}", status.attempt_id);
    if let Some(pending) = status.pending {
        let _ = writeln!(output, "source.pending={}", pending.kind);
        let _ = writeln!(output, "source.pending_words={}", pending.words);
    }

    let Some(report) = source.latest_report() else {
        let _ = writeln!(output, "source.report=none");
        return output;
    };
    let _ = writeln!(
        output,
        "source.report_attempt={}",
        report
            .attempt
            .as_ref()
            .map(|attempt| attempt.id.to_string())
            .unwrap_or_else(|| "none".to_string())
    );
    let _ = writeln!(output, "source.outcome={}", report.outcome);
    let _ = writeln!(output, "source.requested={}", report.requested);
    let _ = writeln!(
        output,
        "source.reported_active={}",
        report
            .active
            .map(|kind| kind.to_string())
            .unwrap_or_else(|| "none".to_string())
    );
    let _ = writeln!(output, "source.loaded={}", report.loaded);
    let _ = writeln!(output, "source.skipped={}", report.skipped);
    let _ = writeln!(output, "source.issue_count={}", report.issues.len());
    let issue_kinds = report
        .issues
        .iter()
        .map(|issue| issue.kind.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let _ = writeln!(output, "source.issue_kinds={issue_kinds}");
    if let Some(attempt) = &report.attempt {
        let completed_ms = attempt
            .completed_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let _ = writeln!(output, "source.elapsed_ms={}", attempt.elapsed.as_millis());
        let _ = writeln!(output, "source.completed_unix_ms={completed_ms}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Word;
    use crate::source::{LoadIssue, LoadIssueKind, LoadReport, SourceKind};
    use std::sync::mpsc;

    #[test]
    fn diagnostics_include_state_but_never_raw_issue_messages() {
        let failed = LoadReport::failed(
            SourceKind::Mongo,
            LoadIssue::new(
                LoadIssueKind::Connection,
                "mongodb://user:secret@private-host:27017",
            ),
        );
        let mut report = LoadReport::with_fallback(
            failed,
            vec![Word {
                word: "fallback".into(),
                transcription: String::new(),
                translation: String::new(),
                frequency: 1,
                example: String::new(),
            }],
        );
        report.complete_attempt(
            1,
            std::time::Duration::from_millis(25),
            UNIX_EPOCH + std::time::Duration::from_secs(10),
        );
        let mut report = Some(report);
        let mut source = SourceController::new(
            40,
            Box::new(move |_| {
                let (tx, rx) = mpsc::channel();
                tx.send(report.take().unwrap()).unwrap();
                rx
            }),
        );
        source.poll().unwrap();

        let output = build(&Config::default(), Some(&source));

        assert!(output.contains("source.health=degraded"));
        assert!(output.contains("source.report_attempt=1"));
        assert!(output.contains("source.issue_kinds=connection"));
        assert!(output.contains("source.elapsed_ms=25"));
        assert!(!output.contains("secret"));
        assert!(!output.contains("private-host"));
    }
}
