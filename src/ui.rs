//! Render-shell facade.
//!
//! The UI is intentionally split into small pieces: platform font setup, text
//! primitives, card-surface effects, and the card composition itself.

mod card;
mod foundation;
mod surface;
mod text;

pub use card::{CardContent, CardStyle, CardTimeline, CardView, MIN_WIDTH};
pub use foundation::{load_fonts, setup_visuals};
