//! Runtime configuration grouped by consumer responsibility.

mod parser;
mod path;

pub const DEFAULT_CARD_OPACITY: f32 = 77.0 / 255.0;
pub const DEFAULT_CORNER_RADIUS: f32 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    pub(super) fn parse(value: &str) -> Option<Self> {
        match normalized(value).as_str() {
            "top-left" => Some(Self::TopLeft),
            "top-right" => Some(Self::TopRight),
            "bottom-left" => Some(Self::BottomLeft),
            "bottom-right" => Some(Self::BottomRight),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemePreset {
    #[default]
    Graphite,
    Midnight,
    Paper,
    HighContrast,
}

impl ThemePreset {
    pub(super) fn parse(value: &str) -> Option<Self> {
        match normalized(value).as_str() {
            "graphite" | "default" => Some(Self::Graphite),
            "midnight" => Some(Self::Midnight),
            "paper" | "light" => Some(Self::Paper),
            "high-contrast" | "contrast" => Some(Self::HighContrast),
            _ => None,
        }
    }
}

fn normalized(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace(['_', ' '], "-")
}

#[derive(Debug, Clone, Copy)]
pub struct TimingConfig {
    pub interval_secs: u64,
    pub jitter_secs: u64,
    pub transcription_delay: f32,
    pub translation_delay: f32,
    pub fade_duration: f32,
    pub exit_duration: f32,
    pub rare_word_dwell: f32,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            interval_secs: 30,
            jitter_secs: 0,
            transcription_delay: 5.0,
            translation_delay: 10.0,
            fade_duration: 1.0,
            exit_duration: 0.0,
            rare_word_dwell: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AppearanceConfig {
    pub corner: Corner,
    pub card_opacity: f32,
    pub corner_radius: f32,
    pub settle_px: f32,
    pub accent_color: Option<[u8; 3]>,
    pub sheen: f32,
    pub theme: ThemePreset,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            corner: Corner::BottomRight,
            card_opacity: DEFAULT_CARD_OPACITY,
            corner_radius: DEFAULT_CORNER_RADIUS,
            settle_px: 0.0,
            accent_color: None,
            sheen: 0.0,
            theme: ThemePreset::Graphite,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LearningConfig {
    pub speak: bool,
    pub recall_mode: bool,
    pub recap_chance: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct AccessibilityConfig {
    pub font_scale: f32,
    pub enhanced_contrast: bool,
    pub reduce_motion: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            font_scale: 1.0,
            enhanced_contrast: false,
            reduce_motion: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Config {
    pub timing: TimingConfig,
    pub appearance: AppearanceConfig,
    pub learning: LearningConfig,
    pub accessibility: AccessibilityConfig,
}

impl Config {
    pub fn load() -> Self {
        path::read_config_file()
            .map(|text| parser::merge(Self::default(), &text))
            .unwrap_or_default()
    }

    #[cfg(test)]
    fn merge_str(self, text: &str) -> Self {
        parser::merge(self, text)
    }
}

#[cfg(test)]
mod tests;
