//! OS adapters behind small ports, so the rest of the app stays platform-agnostic
//! and unit-testable. Today this is text-to-speech and the state-file location;
//! monitor sizing (`fill_screen`) and the config path are candidates to
//! consolidate here next.

use std::path::PathBuf;

/// Where durable learning state lives. `MEMO_STATE` overrides everything (for
/// tests and unusual setups); otherwise the platform's data-directory
/// convention: `~/Library/Application Support/memo-words/` on macOS,
/// `$XDG_DATA_HOME/memo-words/` (or `~/.local/share/memo-words/`) elsewhere.
/// `None` means no home directory could be resolved; the caller falls back to
/// a non-persisting store rather than failing startup.
pub fn state_file_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("MEMO_STATE") {
        return Some(PathBuf::from(path));
    }
    data_dir().map(|dir| dir.join("progress.json"))
}

fn data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join("Library/Application Support/memo-words"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            return Some(PathBuf::from(xdg).join("memo-words"));
        }
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join(".local/share/memo-words"))
    }
}

/// Pronounces a word out loud. A port (dependency inversion) so `App` depends on
/// the capability, not the concrete OS mechanism: the composition root picks the
/// real implementation, and tests inject a recording or no-op double.
pub trait Speaker {
    fn speak(&self, word: &str);
}

/// The real speaker: macOS `say`, fired and forgotten on a detached thread that
/// waits on the child so finished processes are reaped instead of piling up as
/// zombies over a session. A no-op on non-macOS targets.
pub struct SystemSpeaker;

impl Speaker for SystemSpeaker {
    #[cfg(target_os = "macos")]
    fn speak(&self, word: &str) {
        let word = word.to_string();
        std::thread::spawn(move || {
            let _ = std::process::Command::new("say").arg(word).status();
        });
    }

    #[cfg(not(target_os = "macos"))]
    fn speak(&self, _word: &str) {}
}

/// A speaker that does nothing. Used in tests and anywhere a non-speaking wiring
/// is wanted, so callers never need a real TTS process.
pub struct NullSpeaker;

impl Speaker for NullSpeaker {
    fn speak(&self, _word: &str) {}
}
