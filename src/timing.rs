//! Pure timing facade.
//!
//! The implementation is split by reason to change: easing curves, learning
//! pacing, reveal choreography, and repaint scheduling. Callers import this
//! facade, while each small module can be understood and tested independently.

mod easing;
mod pacing;
mod repaint;
mod timeline;

pub use easing::{exit_alpha, fade_factor, settle_offset, smoothstep};
pub use pacing::dwelled_base_secs;
pub use repaint::repaint_after;
pub use timeline::{anim_end, effective_translation_delay, example_delay, exit_window};
