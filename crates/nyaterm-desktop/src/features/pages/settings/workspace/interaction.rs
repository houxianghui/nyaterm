use rust_i18n::t;

use std::borrow::Cow;

use gpui::{Context, FontWeight, IntoElement, SharedString, div, prelude::*, px, rgb};
use nyaterm_core::TerminalRightClickAction;
use nyaterm_ui::NyaSelectOption;

use crate::features::{
    pages::settings::panel::SettingsPanel,
    shell::{TAB_MOUSE_ACTIONS, TabMouseActionTarget},
};
use crate::theme::ThemePalette;

use super::super::{
    settings_form_row, settings_form_section, settings_segmented_control,
    settings_segmented_selected_index, settings_switch,
};

const TERMINAL_RIGHT_CLICK_ACTIONS: [TerminalRightClickAction; 3] = [
    TerminalRightClickAction::None,
    TerminalRightClickAction::Menu,
    TerminalRightClickAction::Paste,
];

impl SettingsPanel {
    pub(in crate::features) fn interaction_settings_section(
        &mut self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let palette = self.theme_palette();
        // Built before the form, which reads `self` throughout: creating the
        // box needs it mutably.
        let word_separators_input =
            self.existing_text_input_box("settings.interaction.word-separators", false);
        let double_action = self
            .settings
            .summary()
            .interaction_tab_double_click_action
            .clone();
        let middle_action = self
            .settings
            .summary()
            .interaction_tab_middle_click_action
            .clone();
        let right_action = self
            .settings
            .summary()
            .interaction_tab_right_click_action
            .clone();
        let _delay_ms = self
            .settings
            .summary()
            .interaction_duplicate_session_command_delay_ms;
        let _min_chars = self
            .settings
            .summary()
            .interaction_command_suggestion_min_chars;
        let _max_chars = self
            .settings
            .summary()
            .interaction_command_suggestion_max_chars;
        let suggestions_enabled = self
            .settings
            .summary()
            .interaction_command_suggestions_enabled;

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(settings_form_section(
                palette,
                Some(t!("settings.interactionClipboardMouse")),
                None,
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(settings_form_row(
                        palette,
                        t!("settings.copyOnSelect"),
                        None,
                        settings_switch(
                            palette,
                            "interaction-copy-select",
                            self.settings.summary().interaction_copy_on_select,
                            cx.listener(|this, _, _, cx| {
                                this.toggle_interaction_copy_on_select(cx);
                            }),
                        ),
                    ))
                    .child(settings_form_row(
                        palette,
                        t!("settings.allowOsc52ClipboardWrite"),
                        Some(SharedString::from(t!(
                            "settings.allowOsc52ClipboardWriteDesc"
                        ))),
                        settings_switch(
                            palette,
                            "interaction-osc52-clipboard",
                            self.settings
                                .summary()
                                .interaction_allow_osc52_clipboard_write,
                            cx.listener(|this, _, _, cx| {
                                this.toggle_osc52_clipboard_write(cx);
                            }),
                        ),
                    ))
                    .child(settings_form_row(
                        palette,
                        t!("settings.terminalRightClickAction"),
                        None,
                        settings_segmented_control(
                            "settings.interaction.terminal-right-click-action",
                            [
                                t!("settings.terminalRightClickNone"),
                                t!("settings.terminalRightClickMenu"),
                                t!("settings.terminalRightClickPaste"),
                            ],
                            terminal_right_click_selected_index(
                                self.settings
                                    .summary()
                                    .interaction_terminal_right_click_action,
                            ),
                            false,
                            cx.listener(|this, index, _, cx| {
                                let Some(action) = terminal_right_click_action(*index) else {
                                    return;
                                };
                                this.with_app(cx, |app, cx| {
                                    app.set_terminal_right_click_action(action, cx);
                                });
                            }),
                        ),
                    ))
                    .child(settings_form_row(
                        palette,
                        t!("settings.mouseEventsRequireAlt"),
                        Some(SharedString::from(t!("settings.mouseEventsRequireAltDesc"))),
                        settings_switch(
                            palette,
                            "interaction-mouse-requires-alt",
                            self.settings.summary().interaction_mouse_events_require_alt,
                            cx.listener(|this, _, _, cx| this.toggle_mouse_events_require_alt(cx)),
                        ),
                    )),
            ))
            .child(settings_form_section(
                palette,
                Some(t!("settings.interactionTerminalPage")),
                None,
                settings_form_row(
                    palette,
                    t!("settings.terminalZoomEnabled"),
                    Some(SharedString::from(t!("settings.terminalZoomEnabledDesc"))),
                    settings_switch(
                        palette,
                        "interaction-terminal-zoom",
                        self.settings.summary().interaction_terminal_zoom_enabled,
                        cx.listener(|this, _, _, cx| {
                            this.toggle_terminal_zoom_enabled(cx);
                        }),
                    ),
                ),
            ))
            .child(settings_form_section(
                palette,
                Some(t!("settings.interactionCommandInput")),
                None,
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(settings_form_row(
                        palette,
                        t!("settings.commandSuggestions"),
                        Some(SharedString::from(t!("settings.commandSuggestionsDesc"))),
                        settings_switch(
                            palette,
                            "interaction-cmd-suggestions",
                            suggestions_enabled,
                            cx.listener(|this, _, _, cx| {
                                this.toggle_command_suggestions(cx);
                            }),
                        ),
                    ))
                    .child(settings_form_row(
                        palette,
                        t!("settings.credentialAutofill"),
                        Some(SharedString::from(t!("settings.credentialAutofillDesc"))),
                        settings_switch(
                            palette,
                            "interaction-credential-autofill",
                            self.settings
                                .summary()
                                .interaction_credential_autofill_enabled,
                            cx.listener(|this, _, _, cx| {
                                this.toggle_credential_autofill(cx);
                            }),
                        ),
                    ))
                    .when(suggestions_enabled, |this| {
                        this.child(settings_form_row(
                            palette,
                            t!("settings.commandSuggestionsMinChars"),
                            None,
                            self.existing_number_input_box(
                                "settings.number.command-suggestion-min-chars",
                            ),
                        ))
                        .child(settings_form_row(
                            palette,
                            t!("settings.commandSuggestionsMaxChars"),
                            None,
                            self.existing_number_input_box(
                                "settings.number.command-suggestion-max-chars",
                            ),
                        ))
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(rgb(palette.text_dimmed))
                                .child(t!("settings.commandSuggestionsLengthDesc")),
                        )
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(settings_field_meta(
                                palette,
                                t!("settings.wordSeparators"),
                                t!("settings.wordSeparatorsDesc"),
                            ))
                            .child(div().w_full().max_w(px(640.)).child(word_separators_input)),
                    )
                    .child(settings_form_row(
                        palette,
                        t!("settings.duplicateSessionCommandDelay"),
                        Some(SharedString::from(t!(
                            "settings.duplicateSessionCommandDelayDesc"
                        ))),
                        self.existing_number_input_box(
                            "settings.number.duplicate-session-command-delay",
                        ),
                    ))
                    .child(settings_form_row(
                        palette,
                        t!("settings.altAsMeta"),
                        Some(SharedString::from(t!("settings.altAsMetaDesc"))),
                        settings_switch(
                            palette,
                            "interaction-alt-meta",
                            self.settings.summary().interaction_alt_as_meta,
                            cx.listener(|this, _, _, cx| {
                                this.toggle_alt_as_meta(cx);
                            }),
                        ),
                    )),
            ))
            .child(settings_form_section(
                palette,
                Some(t!("settings.tabMouseActions")),
                None,
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.tab_mouse_action_settings_field(
                        TabMouseActionPresentation {
                            label: t!("settings.tabDoubleClickAction"),
                            description: None,
                            id: "settings.interaction.tab-double",
                            target: TabMouseActionTarget::Double,
                            current: &double_action,
                        },
                        cx,
                    ))
                    .child(self.tab_mouse_action_settings_field(
                        TabMouseActionPresentation {
                            label: t!("settings.tabMiddleClickAction"),
                            description: None,
                            id: "settings.interaction.tab-middle",
                            target: TabMouseActionTarget::Middle,
                            current: &middle_action,
                        },
                        cx,
                    ))
                    .child(self.tab_mouse_action_settings_field(
                        TabMouseActionPresentation {
                            label: t!("settings.tabRightClickAction"),
                            description: Some(t!("settings.tabRightClickActionDesc")),
                            id: "settings.interaction.tab-right",
                            target: TabMouseActionTarget::Right,
                            current: &right_action,
                        },
                        cx,
                    )),
            ))
    }

    fn tab_mouse_action_settings_field(
        &mut self,
        presentation: TabMouseActionPresentation<'_>,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let TabMouseActionPresentation {
            label,
            description,
            id,
            target,
            current,
        } = presentation;
        let selected_index = settings_segmented_selected_index(&TAB_MOUSE_ACTIONS, &current);

        if selected_index.is_none() {
            let options = TAB_MOUSE_ACTIONS
                .iter()
                .map(|action| NyaSelectOption::new(*action, t!(tab_mouse_action_i18n_key(action))))
                .collect();
            return self
                .settings_select_field(
                    id,
                    label,
                    description.map(SharedString::from),
                    (options, current.to_string(), false),
                    cx,
                )
                .into_any_element();
        }

        let palette = self.theme_palette();
        settings_form_row(
            palette,
            label,
            description.map(SharedString::from),
            settings_segmented_control(
                id,
                TAB_MOUSE_ACTIONS
                    .iter()
                    .map(|action| t!(tab_mouse_action_i18n_key(action))),
                selected_index,
                false,
                cx.listener(move |this, index, _, cx| {
                    let Some(action) = TAB_MOUSE_ACTIONS.get(*index).copied() else {
                        return;
                    };
                    this.with_app(cx, |app, cx| {
                        app.set_tab_mouse_action(target, action, cx);
                    });
                }),
            ),
        )
        .into_any_element()
    }
}

struct TabMouseActionPresentation<'a> {
    label: Cow<'static, str>,
    description: Option<Cow<'static, str>>,
    id: &'static str,
    target: TabMouseActionTarget,
    current: &'a str,
}

fn settings_field_meta(
    palette: ThemePalette,
    label: impl Into<SharedString>,
    desc: impl Into<SharedString>,
) -> impl IntoElement {
    let label: SharedString = label.into();
    let desc: SharedString = desc.into();
    div()
        .min_w_0()
        .child(
            div()
                .text_size(px(13.))
                .font_weight(FontWeight(500.))
                .text_color(rgb(palette.text))
                .child(label),
        )
        .child(
            div()
                .mt_1()
                .text_size(px(11.))
                .text_color(rgb(palette.text_dimmed))
                .child(desc),
        )
}

fn terminal_right_click_selected_index(action: TerminalRightClickAction) -> Option<usize> {
    TERMINAL_RIGHT_CLICK_ACTIONS
        .iter()
        .position(|candidate| *candidate == action)
}

fn terminal_right_click_action(index: usize) -> Option<TerminalRightClickAction> {
    TERMINAL_RIGHT_CLICK_ACTIONS.get(index).copied()
}

fn tab_mouse_action_i18n_key(action: &str) -> &'static str {
    match action {
        "rename_tab" => "tabCtx.rename",
        "copy_tab_name" => "tabCtx.copyName",
        "copy_server_ip" => "tabCtx.copyIp",
        "duplicate_session" => "tabCtx.duplicate",
        "multiplex_ssh" => "tabCtx.multiplexSsh",
        "reconnect_session" => "tabCtx.reconnect",
        "disconnect_session" => "tabCtx.disconnect",
        "close_tab" => "tabCtx.close",
        _ => "settings.tabMouseActionNone",
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestAppContext;
    use nyaterm_core::{AppSettingsSummary, KeywordHighlightConfig, TerminalRightClickAction};

    use crate::features::settings::{
        SettingsFeatureFocus, SettingsFeatureInit, SettingsFeatureState,
    };

    use super::{terminal_right_click_action, terminal_right_click_selected_index};

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
    fn interaction_right_click_segmented_mapping_preserves_compatibility() {
        for (index, action) in [
            TerminalRightClickAction::None,
            TerminalRightClickAction::Menu,
            TerminalRightClickAction::Paste,
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(terminal_right_click_selected_index(action), Some(index));
            assert_eq!(terminal_right_click_action(index), Some(action));
        }
        assert_eq!(terminal_right_click_action(3), None);

        let mut state = settings_state();
        state.set_terminal_right_click_action(
            terminal_right_click_action(2).expect("paste should be selectable"),
        );
        assert_eq!(
            state.summary().interaction_terminal_right_click_action,
            TerminalRightClickAction::Paste
        );
        assert!(state.summary().interaction_right_click_paste);
    }
}
