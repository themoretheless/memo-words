//! OS adapters behind small ports, so the rest of the app stays platform-agnostic
//! and unit-testable. Today this is just text-to-speech; monitor sizing
//! (`fill_screen`) and the config path are candidates to consolidate here next.

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
