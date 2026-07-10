//! Host filesystem lookup for the optional config file.

pub(super) fn read_config_file() -> Option<String> {
    if let Ok(path) = std::env::var("MEMO_CONFIG") {
        return std::fs::read_to_string(path).ok();
    }
    let home = std::env::var("HOME").ok()?;
    std::fs::read_to_string(format!("{home}/.config/memo-words/config.conf")).ok()
}
