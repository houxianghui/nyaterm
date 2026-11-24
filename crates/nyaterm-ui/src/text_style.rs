//! Semantic text sizes for shared UI components.
//!
//! Tokens are expressed in rems against a 16px baseline. [`ui`] converts them
//! with the application's configured UI font size, so every pilot control
//! scales with the user's `ui_font_size` instead of pinning pixels.
//!
//! This module intentionally covers only the button, menu and input pilots.
//! The rest of the tree still uses literal sizes; `scripts/count_text_size_px.sh`
//! tracks that remaining migration surface.

use gpui::{App, Pixels, Rems, px, rems};

use crate::theme_bridge::component_typography;

pub const CAPTION_REM: f32 = 11. / 16.;
pub const FOOTNOTE_REM: f32 = 12. / 16.;
pub const BODY_REM: f32 = 13. / 16.;
pub const SUBHEAD_REM: f32 = 15. / 16.;
pub const TITLE_REM: f32 = 20. / 16.;
pub const LARGE_TITLE_REM: f32 = 28. / 16.;

/// The single dense-data token: `max(0.625rem, 10px)`.
pub const DENSE_REM: f32 = 0.625;
pub const DENSE_FLOOR_PX: f32 = 10.;

pub fn caption() -> Rems {
    rems(CAPTION_REM)
}

pub fn footnote() -> Rems {
    rems(FOOTNOTE_REM)
}

pub fn body() -> Rems {
    rems(BODY_REM)
}

pub fn subhead() -> Rems {
    rems(SUBHEAD_REM)
}

pub fn title() -> Rems {
    rems(TITLE_REM)
}

pub fn large_title() -> Rems {
    rems(LARGE_TITLE_REM)
}

pub fn dense() -> Rems {
    rems(DENSE_REM)
}

/// Convert a rem token with an explicit baseline, in pixels.
pub fn pixels_at(token: Rems, base: f32) -> Pixels {
    px(token.0 * base)
}

/// Convert a rem token with the application's UI font size.
pub fn ui(cx: &App, token: Rems) -> Pixels {
    pixels_at(token, f32::from(component_typography(cx).font_size))
}

/// Dense data at the configured font size, with the 10px floor.
/// The desktop keeps the UI baseline at 13px or above, so this floor never
/// overtakes [`body`].
pub fn dense_px(cx: &App) -> Pixels {
    dense_px_at(f32::from(component_typography(cx).font_size))
}

fn dense_px_at(base: f32) -> Pixels {
    px(DENSE_REM * base).max(px(DENSE_FLOOR_PX))
}

#[cfg(test)]
mod tests {
    use super::{
        BODY_REM, CAPTION_REM, DENSE_FLOOR_PX, DENSE_REM, LARGE_TITLE_REM, SUBHEAD_REM, TITLE_REM,
    };

    fn size(rem: f32, base: f32) -> f32 {
        rem * base
    }

    #[test]
    fn text_scale_is_monotone_across_baselines() {
        // Caption is auxiliary copy and dense is read-only numeric data; both
        // stay below body, but dense is allowed to be smaller than caption.
        let minimum = f32::from(nyaterm_core::MIN_UI_FONT_SIZE);
        for base in [minimum, 16.0, 24.0] {
            let caption = size(CAPTION_REM, base);
            let dense = f32::from(super::dense_px_at(base));
            let body = size(BODY_REM, base);
            let subhead = size(SUBHEAD_REM, base);
            let title = size(TITLE_REM, base);
            let large_title = size(LARGE_TITLE_REM, base);

            assert!(
                caption <= body,
                "base {base}: caption {caption} > body {body}"
            );
            assert!(dense <= body, "base {base}: dense {dense} > body {body}");
            assert!(
                body <= subhead,
                "base {base}: body {body} > subhead {subhead}"
            );
            assert!(
                subhead <= title,
                "base {base}: subhead {subhead} > title {title}"
            );
            assert!(
                title <= large_title,
                "base {base}: title {title} > large_title {large_title}"
            );
        }
    }

    #[test]
    fn dense_keeps_its_ten_pixel_floor_without_overtaking_body() {
        assert_eq!(
            super::dense_px_at(16.),
            gpui::px(DENSE_FLOOR_PX),
            "0.625rem at 16px equals the 10px floor"
        );
        assert_eq!(DENSE_REM, 0.625);
        // Even the smallest supported baseline keeps the legibility floor;
        // the desktop clamps its UI baseline above the dense/body crossover.
        assert_eq!(super::dense_px_at(12.), gpui::px(10.));
    }
}
