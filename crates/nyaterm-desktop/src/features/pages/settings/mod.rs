use std::borrow::Cow;

use gpui::{
    App, ClickEvent, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px, rgb, rgba,
};

use crate::theme::ThemePalette;
use nyaterm_ui::radius_xl;
use nyaterm_ui::{NyaSegmentedControl, NyaSwitch};

use super::super::NyaTermApp;

mod ai;
mod flush;
mod inputs;
pub(in crate::features) mod panel;
mod security;
mod sync_backup;
mod terminal;
mod transfer;
mod translation;
mod workspace;

// Row controls share one width regardless of their component type. Stacked
// fields and toolbar controls take their width from their own containers.
const SETTINGS_ROW_CONTROL_WIDTH_PX: f32 = 260.;

impl NyaTermApp {
    pub(in crate::features) fn settings_view(
        &mut self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.request_settings_panel_refresh(cx);
        self.settings_panel
            .clone()
            .cached(crate::features::layout::cached_panel_style())
    }
}

pub(super) fn settings_form_section(
    palette: ThemePalette,
    title: Option<Cow<'static, str>>,
    desc: Option<Cow<'static, str>>,
    content: impl IntoElement,
) -> impl IntoElement {
    div()
        .rounded(radius_xl())
        .bg(rgba((palette.surface << 8) | 0x99))
        .overflow_hidden()
        .when(title.is_some() || desc.is_some(), |this| {
            this.child(
                div()
                    .px_4()
                    .py_4()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .when_some(title, |this, title| {
                        this.child(
                            div()
                                .text_size(px(14.))
                                .font_weight(FontWeight(600.))
                                .text_color(rgb(palette.text))
                                .child(title),
                        )
                    })
                    .when_some(desc, |this, desc| {
                        this.child(
                            div()
                                .text_size(px(12.))
                                .text_color(rgb(palette.text_dimmed))
                                .child(desc),
                        )
                    }),
            )
        })
        .child(div().px_4().py_4().flex().flex_col().gap_4().child(content))
}

/// Tauri SettingRow: label/desc left, control right.
pub(super) fn settings_form_row(
    palette: ThemePalette,
    label: impl Into<SharedString>,
    desc: Option<SharedString>,
    control: impl IntoElement,
) -> impl IntoElement {
    let label = label.into();
    div()
        .flex()
        .flex_wrap()
        .items_start()
        .justify_between()
        .gap_4()
        .child(
            div()
                .min_w_0()
                .min_w(px(200.))
                .flex_1()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(FontWeight(500.))
                        .text_color(rgb(palette.text))
                        .child(label),
                )
                .when_some(desc, |this, desc| {
                    this.child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(palette.text_dimmed))
                            .child(desc),
                    )
                }),
        )
        .child(
            div()
                .w(px(SETTINGS_ROW_CONTROL_WIDTH_PX))
                .flex_shrink_0()
                .min_w_0()
                .max_w_full()
                .flex()
                .items_center()
                .justify_end()
                .gap_2()
                .child(control),
        )
}

/// Gives an input a definite width inside a content-sized settings row control slot.
pub(super) fn settings_input_control(input: impl IntoElement) -> gpui::Div {
    div()
        .w(px(SETTINGS_ROW_CONTROL_WIDTH_PX))
        .max_w_full()
        .min_w_0()
        .flex()
        .child(div().min_w_0().flex_1().child(input))
}

/// Keeps an input and its trailing action on one line in a settings row.
pub(super) fn settings_input_action_control(
    width: f32,
    input: impl IntoElement,
    action: impl IntoElement,
) -> impl IntoElement {
    div()
        .w(px(width))
        .max_w_full()
        .min_w_0()
        .flex()
        .items_center()
        .gap_2()
        .child(div().min_w_0().flex_1().child(input))
        .child(div().flex_none().child(action))
}

/// Maps a persisted value to its segmented option without inventing a selection.
pub(super) fn settings_segmented_selected_index<T: PartialEq>(
    options: &[T],
    selected: &T,
) -> Option<usize> {
    options.iter().position(|option| option == selected)
}

pub(super) fn settings_segmented_control(
    id: impl Into<String>,
    labels: impl IntoIterator<Item = impl Into<SharedString>>,
    selected_index: Option<usize>,
    disabled: bool,
    on_select: impl Fn(&usize, &mut Window, &mut App) + 'static,
) -> NyaSegmentedControl {
    NyaSegmentedControl::new(id.into())
        .items(labels)
        .selected_index(selected_index)
        .disabled(disabled)
        .on_select(on_select)
}

/// Compact on/off switch control (Tauri SettingSwitch look).
pub(in crate::features::pages) fn settings_switch(
    palette: ThemePalette,
    id: impl Into<String>,
    checked: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    settings_switch_with_enabled(palette, id, checked, true, on_click)
}

pub(super) fn settings_switch_with_enabled(
    _palette: ThemePalette,
    id: impl Into<String>,
    checked: bool,
    enabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    NyaSwitch::new(id.into())
        .checked(checked)
        .disabled(!enabled)
        .on_click(move |_, window, cx| {
            if enabled {
                on_click(&ClickEvent::default(), window, cx);
            }
        })
}

#[cfg(test)]
mod tests {
    use gpui::TestAppContext;
    use nyaterm_core::{AppSettingsSummary, KeywordHighlightConfig};

    use crate::features::settings::{
        SettingsFeatureFocus, SettingsFeatureInit, SettingsFeatureState,
    };

    use super::settings_segmented_selected_index;

    fn settings_state() -> SettingsFeatureState {
        let cx = TestAppContext::single();
        let _test_app_cleanup = crate::test_support::TestAppCleanup::new(&cx);
        let focus = || cx.update(|cx| cx.focus_handle());
        SettingsFeatureState::new(
            SettingsFeatureInit {
                summary: AppSettingsSummary::default(),
                keyword_config: KeywordHighlightConfig::default(),
                store_path: String::new(),
                store_message: String::new(),
                store_ready: true,
                ui_font_options: vec!["Inter".to_string()],
                terminal_font_options: vec!["JetBrains Mono".to_string()],
            },
            SettingsFeatureFocus {
                search_engine: focus(),
                keyword_highlight: focus(),
                keybindings: focus(),
            },
        )
    }

    #[test]
    fn settings_segmented_selected_index_matches_only_known_values() {
        let levels = ["warn", "info", "debug"];
        assert_eq!(settings_segmented_selected_index(&levels, &"info"), Some(1));
        assert_eq!(settings_segmented_selected_index(&levels, &"trace"), None);

        let retention_days = [3, 7, 14, 30];
        assert_eq!(
            settings_segmented_selected_index(&retention_days, &14),
            Some(2)
        );
        assert_eq!(
            settings_segmented_selected_index(&retention_days, &99),
            None
        );
    }

    #[test]
    fn settings_segmented_diagnostics_updates_only_the_selected_draft_field() {
        let mut state = settings_state();
        let mut expected = state.summary().clone();
        expected.diagnostics_level = "debug".to_string();
        assert!(state.set_diagnostics_level("debug"));
        assert_eq!(state.summary(), &expected);

        expected.diagnostics_retention_days = 14;
        assert!(state.set_diagnostics_retention_days(14));
        assert_eq!(state.summary(), &expected);
    }
}
