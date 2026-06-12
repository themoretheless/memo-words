//! Runtime configuration loaded from a simple `key = value` file.
//!
//! Lookup order: the path in `MEMO_CONFIG`, else `$HOME/.config/memo-words/config.conf`.
//! Unknown keys and unparseable values are ignored (the default is kept), so a
//! malformed file never prevents the app from starting. Every field defaults to
//! the original hard-coded behaviour, so no config file means no change.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().replace(['_', ' '], "-").as_str() {
            "top-left" => Some(Corner::TopLeft),
            "top-right" => Some(Corner::TopRight),
            "bottom-left" => Some(Corner::BottomLeft),
            "bottom-right" => Some(Corner::BottomRight),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub interval_secs: u64,
    pub transcription_delay: f32,
    pub translation_delay: f32,
    pub fade_duration: f32,
    pub corner: Corner,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interval_secs: 30,
            transcription_delay: 5.0,
            translation_delay: 10.0,
            fade_duration: 1.0,
            corner: Corner::BottomRight,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut cfg = Config::default();
        let Some(text) = read_config_file() else {
            return cfg;
        };

        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            match key {
                "interval_secs" => {
                    if let Ok(v) = value.parse::<u64>() {
                        cfg.interval_secs = v.max(1);
                    }
                }
                "transcription_delay" => {
                    if let Ok(v) = value.parse::<f32>() {
                        cfg.transcription_delay = v.max(0.0);
                    }
                }
                "translation_delay" => {
                    if let Ok(v) = value.parse::<f32>() {
                        cfg.translation_delay = v.max(0.0);
                    }
                }
                "fade_duration" => {
                    if let Ok(v) = value.parse::<f32>() {
                        cfg.fade_duration = v.max(0.01);
                    }
                }
                "corner" => {
                    if let Some(c) = Corner::parse(value) {
                        cfg.corner = c;
                    }
                }
                _ => {}
            }
        }
        cfg
    }
}

fn read_config_file() -> Option<String> {
    if let Ok(path) = std::env::var("MEMO_CONFIG") {
        return std::fs::read_to_string(path).ok();
    }
    let home = std::env::var("HOME").ok()?;
    let path = format!("{home}/.config/memo-words/config.conf");
    std::fs::read_to_string(path).ok()
}
