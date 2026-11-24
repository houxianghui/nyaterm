//! WCAG contrast gate for the built-in theme catalog.
//!
//! Every pair is evaluated with pure sRGB math after compositing translucent
//! UI text onto the surface it is actually painted on. Known legacy palettes
//! are listed in [`KNOWN_FAILURES`]; anything else must meet its threshold, and
//! an allowlisted pair that starts passing makes the test fail so the list
//! cannot silently outlive its reason.

use nyaterm_core::themes;
use nyaterm_core::themes::ThemeConfig;

use crate::theme::{ThemePalette, palette_from_theme, solid_button_color};

/// Activity-bar separators use 65% opacity over their menu surface.
const ACTIVITY_BAR_SEPARATOR_ALPHA: f32 = 0.65;
/// Settings-sidebar borders use 70% alpha over a 20% elevated surface.
const SETTINGS_SIDEBAR_BORDER_ALPHA: f32 = 0.70;
const SETTINGS_SIDEBAR_SURFACE_ALPHA: f32 = 0x33 as f32 / 255.;

const TEXT_MIN: f32 = 4.5;
const GRAPHIC_MIN: f32 = 3.0;
const BORDER_MIN: f32 = 1.5;

struct ContrastPair<'a> {
    theme: &'a str,
    role: &'static str,
    foreground: u32,
    background: u32,
    minimum: f32,
}

fn channel(value: u32) -> f32 {
    let v = (value as f32) / 255.0;
    if v <= 0.03928 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance(rgb: u32) -> f32 {
    let r = channel((rgb >> 16) & 0xff);
    let g = channel((rgb >> 8) & 0xff);
    let b = channel(rgb & 0xff);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast_ratio(a: u32, b: u32) -> f32 {
    let (lighter, darker) = {
        let la = relative_luminance(a);
        let lb = relative_luminance(b);
        if la > lb { (la, lb) } else { (lb, la) }
    };
    (lighter + 0.05) / (darker + 0.05)
}

/// Composite `foreground` at `alpha` over `background`, returning opaque RGB.
fn composite(foreground: u32, background: u32, alpha: f32) -> u32 {
    let mix = |shift: u32| {
        let fg = ((foreground >> shift) & 0xff) as f32;
        let bg = ((background >> shift) & 0xff) as f32;
        (fg * alpha + bg * (1. - alpha)).round() as u32
    };
    (mix(16) << 16) | (mix(8) << 8) | mix(0)
}

fn push<'a>(
    pairs: &mut Vec<ContrastPair<'a>>,
    theme: &'a str,
    role: &'static str,
    foreground: u32,
    background: u32,
    minimum: f32,
) {
    pairs.push(ContrastPair {
        theme,
        role,
        foreground,
        background,
        minimum,
    });
}

fn pairs_for<'a>(theme: &'a ThemeConfig) -> Vec<ContrastPair<'a>> {
    let palette = palette_from_theme(theme);
    let ThemePalette {
        bg,
        surface,
        surface_elevated,
        text,
        text_muted,
        primary,
        danger,
        warning,
        success,
        link,
        focus_ring,
        on_primary,
        border,
        ..
    } = palette;
    let id = theme.id.as_str();
    let mut pairs = Vec::new();

    for (place, background) in [
        ("bg", bg),
        ("surface", surface),
        ("surface_elevated", surface_elevated),
    ] {
        push(
            &mut pairs,
            id,
            match place {
                "bg" => "text/bg",
                "surface" => "text/surface",
                _ => "text/surface_elevated",
            },
            text,
            background,
            TEXT_MIN,
        );
    }
    for (place, background) in [("bg", bg), ("surface", surface)] {
        push(
            &mut pairs,
            id,
            match place {
                "bg" => "text_muted/bg",
                _ => "text_muted/surface",
            },
            text_muted,
            background,
            TEXT_MIN,
        );
    }
    push(
        &mut pairs,
        id,
        "activity_bar_separator/surface@0.65",
        composite(border, surface, ACTIVITY_BAR_SEPARATOR_ALPHA),
        surface,
        BORDER_MIN,
    );
    let settings_sidebar_surface = composite(surface_elevated, bg, SETTINGS_SIDEBAR_SURFACE_ALPHA);
    push(
        &mut pairs,
        id,
        "settings_sidebar_border/surface@0.70",
        composite(
            border,
            settings_sidebar_surface,
            SETTINGS_SIDEBAR_BORDER_ALPHA,
        ),
        settings_sidebar_surface,
        BORDER_MIN,
    );
    for (role, color) in [
        ("primary/bg", primary),
        ("danger/bg", danger),
        ("warning/bg", warning),
        ("success/bg", success),
        ("link/bg", link),
        ("focus_ring/bg", focus_ring),
    ] {
        push(&mut pairs, id, role, color, bg, GRAPHIC_MIN);
    }
    for (role, color) in [
        ("on_primary/primary_fill", primary),
        ("on_primary/danger_fill", danger),
        ("on_primary/success_fill", success),
        ("on_primary/warning_fill", warning),
    ] {
        push(
            &mut pairs,
            id,
            role,
            solid_button_color(color, on_primary),
            on_primary,
            TEXT_MIN,
        );
    }
    push(&mut pairs, id, "border/bg", border, bg, BORDER_MIN);
    push(
        &mut pairs,
        id,
        "border/surface",
        border,
        surface,
        BORDER_MIN,
    );
    pairs
}

/// Built-in palettes whose pairs are known to miss the thresholds today.
/// Each entry must still be failing when the test runs.
const KNOWN_FAILURES: &[(&str, &str)] = &[
    // Legacy palettes kept as-is by the P0 contrast pass. They are recorded
    // here rather than retuned because PR-4 deliberately does not redesign the
    // other built-ins.
    ("nya-high-contrast", "activity_bar_separator/surface@0.65"),
    ("nya-high-contrast", "settings_sidebar_border/surface@0.70"),
    ("dracula", "activity_bar_separator/surface@0.65"),
    ("dracula", "settings_sidebar_border/surface@0.70"),
    ("nord", "text_muted/surface"),
    ("monokai-pro", "settings_sidebar_border/surface@0.70"),
    ("solarized-light", "text/surface"),
    ("solarized-light", "text/surface_elevated"),
    ("solarized-light", "text_muted/bg"),
    ("solarized-light", "text_muted/surface"),
    ("solarized-light", "activity_bar_separator/surface@0.65"),
    ("solarized-light", "warning/bg"),
    ("solarized-light", "success/bg"),
    ("catppuccin-mocha", "activity_bar_separator/surface@0.65"),
    ("catppuccin-mocha", "settings_sidebar_border/surface@0.70"),
    ("tokyo-night", "text_muted/bg"),
    ("tokyo-night", "text_muted/surface"),
    ("tokyo-night", "activity_bar_separator/surface@0.65"),
    ("tokyo-night", "settings_sidebar_border/surface@0.70"),
    ("rose-pine", "activity_bar_separator/surface@0.65"),
    ("one-dark-pro", "text_muted/bg"),
    ("one-dark-pro", "text_muted/surface"),
    ("gruvbox-dark", "settings_sidebar_border/surface@0.70"),
    ("catppuccin-latte", "text_muted/bg"),
    ("catppuccin-latte", "text_muted/surface"),
    ("catppuccin-latte", "activity_bar_separator/surface@0.65"),
    ("catppuccin-latte", "settings_sidebar_border/surface@0.70"),
    ("catppuccin-latte", "warning/bg"),
    ("catppuccin-latte", "success/bg"),
    ("rose-pine-dawn", "text_muted/bg"),
    ("rose-pine-dawn", "text_muted/surface"),
    ("rose-pine-dawn", "activity_bar_separator/surface@0.65"),
    ("rose-pine-dawn", "settings_sidebar_border/surface@0.70"),
    ("rose-pine-dawn", "warning/bg"),
    ("nord-light", "border/surface"),
    ("nord-light", "activity_bar_separator/surface@0.65"),
    ("nord-light", "settings_sidebar_border/surface@0.70"),
    ("one-light", "text_muted/surface"),
    ("one-light", "border/surface"),
    ("one-light", "activity_bar_separator/surface@0.65"),
    ("one-light", "settings_sidebar_border/surface@0.70"),
    ("dracula-light", "border/bg"),
    ("dracula-light", "border/surface"),
    ("dracula-light", "activity_bar_separator/surface@0.65"),
    ("dracula-light", "settings_sidebar_border/surface@0.70"),
    ("windterm-dark", "activity_bar_separator/surface@0.65"),
    ("windterm-dark", "settings_sidebar_border/surface@0.70"),
    ("windterm-light", "activity_bar_separator/surface@0.65"),
    ("windterm-light", "settings_sidebar_border/surface@0.70"),
    ("windterm-light", "success/bg"),
];

#[test]
fn builtin_theme_contrast_gate() {
    let mut failures = Vec::new();
    for theme in themes::builtins() {
        for pair in pairs_for(theme) {
            let ratio = contrast_ratio(pair.foreground, pair.background);
            if ratio + 1e-6 < pair.minimum {
                failures.push((pair.theme.to_string(), pair.role));
            }
        }
    }

    let known = |theme: &str, role: &str| {
        KNOWN_FAILURES
            .iter()
            .any(|(known_theme, known_role)| *known_theme == theme && *known_role == role)
    };

    let unexpected: Vec<_> = failures
        .iter()
        .filter(|(theme, role)| !known(theme, role))
        .collect();
    let stale: Vec<_> = KNOWN_FAILURES
        .iter()
        .filter(|(theme, role)| known_theme_passes(theme, role))
        .collect();
    assert!(
        unexpected.is_empty() && stale.is_empty(),
        "contrast failures not covered by KNOWN_FAILURES: {unexpected:#?}\n\
         stale KNOWN_FAILURES entries that now pass or no longer exist: {stale:#?}"
    );
}

/// Whether an allowlisted pair currently clears its threshold.
fn known_theme_passes(theme_id: &str, role: &str) -> bool {
    let Some(theme) = themes::builtins().iter().find(|theme| theme.id == theme_id) else {
        // A stale allowlist entry for a deleted theme is itself a failure.
        return true;
    };
    pairs_for(theme)
        .into_iter()
        .filter(|pair| pair.role == role)
        .all(|pair| contrast_ratio(pair.foreground, pair.background) >= pair.minimum)
}

#[test]
fn separators_composite_their_border_alpha_not_the_surface() {
    let theme = themes::builtins()
        .iter()
        .find(|theme| theme.id == "github-dark")
        .expect("default dark theme");
    let palette = palette_from_theme(theme);
    let pairs = pairs_for(theme);
    let activity = pairs
        .iter()
        .find(|pair| pair.role == "activity_bar_separator/surface@0.65")
        .expect("the activity-bar separator must be tested with its real alpha");
    assert_eq!(
        activity.foreground,
        composite(palette.border, palette.surface, 0.65)
    );
    assert_eq!(activity.background, palette.surface);

    let settings = pairs
        .iter()
        .find(|pair| pair.role == "settings_sidebar_border/surface@0.70")
        .expect("the settings-sidebar border must be tested with its real alpha");
    let sidebar_surface = composite(palette.surface_elevated, palette.bg, 0x33 as f32 / 255.);
    assert_eq!(
        settings.foreground,
        composite(palette.border, sidebar_surface, 0.70)
    );
    assert_eq!(settings.background, sidebar_surface);
}

#[test]
fn light_default_danger_hover_is_darker_than_resting_danger() {
    let theme = themes::builtins()
        .iter()
        .find(|theme| theme.id == "github-light")
        .expect("default light theme");
    let palette = palette_from_theme(theme);
    assert!(
        relative_luminance(palette.danger_hover) < relative_luminance(palette.danger),
        "a light-theme danger hover must darken, not lighten, the resting danger"
    );
}

#[test]
fn default_light_and_dark_palettes_meet_functional_text_contrast() {
    // The two defaults are held to the full bar even before the allowlist is
    // consulted, so a regression in either of them is never waved through.
    for theme in themes::builtins()
        .iter()
        .filter(|theme| matches!(theme.id.as_str(), "github-dark" | "github-light"))
    {
        let palette = palette_from_theme(theme);
        for (role, fg, bg) in [
            ("text/bg", palette.text, palette.bg),
            ("text/surface", palette.text, palette.surface),
            ("text_muted/bg", palette.text_muted, palette.bg),
            ("text_muted/surface", palette.text_muted, palette.surface),
        ] {
            assert!(
                contrast_ratio(fg, bg) >= TEXT_MIN,
                "{} {role} contrast {:.2}",
                theme.id,
                contrast_ratio(fg, bg)
            );
        }
        for (role, color) in [
            ("primary", palette.primary),
            ("danger", palette.danger),
            ("warning", palette.warning),
            ("success", palette.success),
            ("focus_ring", palette.focus_ring),
        ] {
            for (place, background) in [("bg", palette.bg), ("surface", palette.surface)] {
                let ratio = contrast_ratio(color, background);
                assert!(
                    ratio >= GRAPHIC_MIN,
                    "{} {role}/{place} contrast {ratio:.2}",
                    theme.id
                );
            }
            let fill = solid_button_color(color, palette.on_primary);
            let ratio = contrast_ratio(fill, palette.on_primary);
            assert!(
                ratio >= TEXT_MIN,
                "{} on_primary/{role}_fill contrast {ratio:.2}",
                theme.id
            );
        }
        // In the light default, warning and danger carry functional copy, so
        // they must clear the body-text bar rather than the graphic one.
        if theme.id == "github-light" {
            for (role, fg) in [("warning", palette.warning), ("danger", palette.danger)] {
                assert!(
                    contrast_ratio(fg, palette.surface) >= TEXT_MIN,
                    "{role} on surface"
                );
            }
        }
    }
}
