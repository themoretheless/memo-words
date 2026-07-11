//! Semantic design tokens for the ambient card.

use crate::config::{Config, ThemePreset};
use eframe::egui;

#[derive(Clone, Copy)]
pub struct TypeScale {
    pub word: f32,
    pub translation: f32,
    pub transcription: f32,
    pub example: f32,
}

#[derive(Clone, Copy)]
pub struct TextPalette {
    pub word: egui::Color32,
    pub translation: egui::Color32,
    pub transcription: egui::Color32,
    pub example: egui::Color32,
}

#[derive(Clone, Copy)]
pub struct CardMetrics {
    pub screen_margin: f32,
    pub inner_margin: f32,
    pub min_width: f32,
    pub max_width: f32,
    pub height: f32,
    pub accent_width: f32,
    pub accent_thickness: f32,
    pub accent_gap: f32,
    pub width_transition: f32,
}

#[derive(Clone, Copy)]
pub struct Theme {
    surface_rgb: [u8; 3],
    min_surface_opacity: f32,
    pub border: egui::Color32,
    pub shadow: egui::epaint::Shadow,
    pub text: TextPalette,
    pub type_scale: TypeScale,
    pub metrics: CardMetrics,
    pub sheen_max_alpha: u8,
    pub sheen_height_fraction: f32,
}

impl Theme {
    pub fn from_config(cfg: &Config) -> Self {
        let mut theme = match cfg.appearance.theme {
            ThemePreset::Graphite => graphite(),
            ThemePreset::Midnight => midnight(),
            ThemePreset::Paper => paper(),
            ThemePreset::HighContrast => high_contrast(),
        };

        let scale = cfg.accessibility.font_scale.clamp(0.8, 1.5);
        theme.type_scale = TypeScale {
            word: 32.0 * scale,
            translation: 18.0 * scale,
            transcription: 14.0 * scale,
            example: 13.0 * scale,
        };
        theme.metrics = CardMetrics {
            screen_margin: 40.0,
            inner_margin: 20.0 * scale.clamp(0.9, 1.25),
            min_width: 150.0 * scale.clamp(0.9, 1.2),
            max_width: 600.0 * scale.max(1.0),
            height: 160.0 * scale.max(0.9),
            accent_width: 28.0 * scale,
            accent_thickness: (2.0 * scale).max(1.0),
            accent_gap: 4.0 * scale,
            width_transition: 0.5,
        };

        if cfg.accessibility.enhanced_contrast {
            theme.min_surface_opacity = theme.min_surface_opacity.max(0.82);
            theme.border = with_alpha(theme.border, theme.border.a().max(72));
            theme.text = TextPalette {
                word: with_alpha(theme.text.word, 255),
                translation: with_alpha(
                    theme.text.translation,
                    theme.text.translation.a().max(245),
                ),
                transcription: with_alpha(
                    theme.text.transcription,
                    theme.text.transcription.a().max(215),
                ),
                example: with_alpha(theme.text.example, theme.text.example.a().max(195)),
            };
        }

        theme
    }

    pub fn card_background(self, requested_opacity: f32) -> egui::Color32 {
        let [r, g, b] = self.surface_rgb;
        let alpha = (requested_opacity.clamp(self.min_surface_opacity, 1.0) * 255.0).round() as u8;
        egui::Color32::from_rgba_unmultiplied(r, g, b, alpha)
    }
}

fn graphite() -> Theme {
    base_theme(
        [30, 30, 30],
        0.0,
        egui::Color32::from_rgba_premultiplied(18, 18, 18, 18),
        TextPalette {
            word: egui::Color32::WHITE,
            translation: tone(215),
            transcription: tone(145),
            example: tone(120),
        },
    )
}

fn midnight() -> Theme {
    base_theme(
        [20, 28, 38],
        0.42,
        egui::Color32::from_rgba_unmultiplied(137, 180, 224, 42),
        TextPalette {
            word: egui::Color32::from_rgb(245, 248, 252),
            translation: egui::Color32::from_rgba_unmultiplied(222, 232, 242, 225),
            transcription: egui::Color32::from_rgba_unmultiplied(162, 190, 214, 175),
            example: egui::Color32::from_rgba_unmultiplied(168, 182, 198, 150),
        },
    )
}

fn paper() -> Theme {
    let mut theme = base_theme(
        [242, 244, 246],
        0.9,
        egui::Color32::from_rgba_unmultiplied(20, 28, 34, 38),
        TextPalette {
            word: egui::Color32::from_rgb(18, 22, 26),
            translation: egui::Color32::from_rgba_unmultiplied(30, 38, 44, 235),
            transcription: egui::Color32::from_rgba_unmultiplied(66, 78, 88, 195),
            example: egui::Color32::from_rgba_unmultiplied(78, 88, 96, 175),
        },
    );
    theme.shadow.color = egui::Color32::from_black_alpha(45);
    theme.sheen_max_alpha = 14;
    theme
}

fn high_contrast() -> Theme {
    let mut theme = base_theme(
        [0, 0, 0],
        0.9,
        egui::Color32::from_white_alpha(110),
        TextPalette {
            word: egui::Color32::WHITE,
            translation: egui::Color32::from_white_alpha(255),
            transcription: egui::Color32::from_white_alpha(225),
            example: egui::Color32::from_white_alpha(205),
        },
    );
    theme.shadow.color = egui::Color32::from_black_alpha(100);
    theme
}

fn base_theme(
    surface_rgb: [u8; 3],
    min_surface_opacity: f32,
    border: egui::Color32,
    text: TextPalette,
) -> Theme {
    Theme {
        surface_rgb,
        min_surface_opacity,
        border,
        shadow: egui::epaint::Shadow {
            offset: [0, 6],
            blur: 24,
            spread: 0,
            color: egui::Color32::from_black_alpha(60),
        },
        text,
        type_scale: TypeScale {
            word: 32.0,
            translation: 18.0,
            transcription: 14.0,
            example: 13.0,
        },
        metrics: CardMetrics {
            screen_margin: 40.0,
            inner_margin: 20.0,
            min_width: 150.0,
            max_width: 600.0,
            height: 160.0,
            accent_width: 28.0,
            accent_thickness: 2.0,
            accent_gap: 4.0,
            width_transition: 0.5,
        },
        sheen_max_alpha: 30,
        sheen_height_fraction: 0.55,
    }
}

fn tone(intensity: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(intensity, intensity, intensity, intensity)
}

fn with_alpha(color: egui::Color32, alpha: u8) -> egui::Color32 {
    let [r, g, b, _] = color.to_srgba_unmultiplied();
    egui::Color32::from_rgba_unmultiplied(r, g, b, alpha)
}

pub fn faded(color: egui::Color32, ease: f32) -> egui::Color32 {
    dim(color, ease)
}

pub fn dim(color: egui::Color32, factor: f32) -> egui::Color32 {
    let factor = factor.clamp(0.0, 1.0);
    let scale = |channel: u8| (channel as f32 * factor) as u8;
    egui::Color32::from_rgba_premultiplied(
        scale(color.r()),
        scale(color.g()),
        scale(color.b()),
        scale(color.a()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_CARD_OPACITY;

    #[test]
    fn graphite_default_preserves_original_surface() {
        let theme = Theme::from_config(&Config::default());
        let original = egui::Color32::from_rgba_premultiplied(9, 9, 9, 77);
        assert_eq!(theme.card_background(DEFAULT_CARD_OPACITY), original);
    }

    #[test]
    fn every_scale_keeps_answer_first_hierarchy() {
        for scale in [0.8, 1.0, 1.25, 1.5] {
            let mut cfg = Config::default();
            cfg.accessibility.font_scale = scale;
            let sizes = Theme::from_config(&cfg).type_scale;
            assert!(sizes.word > sizes.translation);
            assert!(sizes.translation > sizes.transcription);
            assert!(sizes.transcription > sizes.example);
        }
    }

    #[test]
    fn readable_presets_enforce_opaque_surfaces() {
        for preset in [ThemePreset::Paper, ThemePreset::HighContrast] {
            let mut cfg = Config::default();
            cfg.appearance.theme = preset;
            assert!(Theme::from_config(&cfg).card_background(0.0).a() >= 229);
        }
    }

    #[test]
    fn enhanced_contrast_raises_surface_and_secondary_text() {
        let mut cfg = Config::default();
        cfg.accessibility.enhanced_contrast = true;
        let theme = Theme::from_config(&cfg);
        assert!(theme.card_background(0.1).a() >= 209);
        assert!(theme.text.transcription.a() >= 215);
    }

    #[test]
    fn enhanced_contrast_never_dims_an_already_stronger_preset() {
        let mut cfg = Config::default();
        cfg.appearance.theme = ThemePreset::HighContrast;
        let normal = Theme::from_config(&cfg);
        cfg.accessibility.enhanced_contrast = true;
        let enhanced = Theme::from_config(&cfg);
        assert!(enhanced.text.translation.a() >= normal.text.translation.a());
        assert!(enhanced.text.transcription.a() >= normal.text.transcription.a());
        assert!(enhanced.text.example.a() >= normal.text.example.a());
    }

    #[test]
    fn dim_has_identity_and_transparent_endpoints() {
        let color = egui::Color32::from_rgb(200, 100, 50);
        assert_eq!(dim(color, 1.0), color);
        assert_eq!(dim(color, 0.0), egui::Color32::TRANSPARENT);
    }
}
