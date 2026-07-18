//! Domain-neutral commands accepted by the application shell.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppCommand {
    NextWord,
    TogglePause,
    ReloadSource,
    CopyDiagnostics,
    Quit,
}
