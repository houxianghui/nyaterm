use rust_i18n::t;

use std::borrow::Cow;

use gpui::{Context, FontWeight, IntoElement, SharedString, div, prelude::*, px, rgb};
use nyaterm_core::TerminalRightClickAction;
use nyaterm_ui::NyaSelectOption;

use crate::features::{pages::settings::panel::SettingsPanel, shell::TAB_MOUSE_ACTIONS};
use crate::theme::ThemePalette;

use super::super::{
    settings_form_row, settings_form_section, settings_segmented_control, settings_switch,
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
                            current: &double_action,
                        },
                        cx,
                    ))
                    .child(self.tab_mouse_action_settings_field(
                        TabMouseActionPresentation {
                            label: t!("settings.tabMiddleClickAction"),
                            description: None,
                            id: "settings.interaction.tab-middle",
                            current: &middle_action,
                        },
                        cx,
                    ))
                    .child(self.tab_mouse_action_settings_field(
                        TabMouseActionPresentation {
                            label: t!("settings.tabRightClickAction"),
                            description: Some(t!("settings.tabRightClickActionDesc")),
                            id: "settings.interaction.tab-right",
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
            current,
        } = presentation;
        let options = TAB_MOUSE_ACTIONS
            .iter()
            .map(|action| NyaSelectOption::new(*action, t!(tab_mouse_action_i18n_key(action))))
            .collect();

        self.settings_select_field(
            id,
            label,
            description.map(SharedString::from),
            (options, current.to_string(), false),
            cx,
        )
        .into_any_element()
    }
}

struct TabMouseActionPresentation<'a> {
    label: Cow<'static, str>,
    description: Option<Cow<'static, str>>,
    id: &'static str,
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
    use std::path::Path;

    use gpui::{
        AppContext as _, Entity, IntoElement, Modifiers, ParentElement as _, Render, Styled as _,
        TestAppContext, VisualTestContext, div, px,
    };
    use nyaterm_core::{
        AppRuntime, AppSettingsSummary, KeywordHighlightConfig, RuntimeMode,
        TerminalRightClickAction,
    };

    use crate::entities::{OverlayStore, StartupRestoreStore, UiStoreHandles};
    use crate::features::pages::settings::panel::SettingsPanel;
    use crate::features::settings::{
        SettingsFeatureFocus, SettingsFeatureInit, SettingsFeatureState,
    };
    use crate::features::{
        NyaTermApp,
        shell::{TAB_MOUSE_ACTIONS, TabMouseActionTarget},
    };
    use crate::models::{NavItem, SettingsTab};
    use crate::test_support::TestConfigDir;

    use super::{
        TabMouseActionPresentation, terminal_right_click_action,
        terminal_right_click_selected_index,
    };

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

    struct TabMouseActionsHost {
        panel: Entity<SettingsPanel>,
    }

    impl Render for TabMouseActionsHost {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            let fields = self.panel.update(cx, |panel, cx| {
                let double_action = panel
                    .settings
                    .summary()
                    .interaction_tab_double_click_action
                    .clone();
                let middle_action = panel
                    .settings
                    .summary()
                    .interaction_tab_middle_click_action
                    .clone();
                let right_action = panel
                    .settings
                    .summary()
                    .interaction_tab_right_click_action
                    .clone();
                div()
                    .flex()
                    .flex_col()
                    .child(panel.tab_mouse_action_settings_field(
                        TabMouseActionPresentation {
                            label: "Double click".into(),
                            description: None,
                            id: "settings.interaction.tab-double",
                            current: &double_action,
                        },
                        cx,
                    ))
                    .child(panel.tab_mouse_action_settings_field(
                        TabMouseActionPresentation {
                            label: "Middle click".into(),
                            description: None,
                            id: "settings.interaction.tab-middle",
                            current: &middle_action,
                        },
                        cx,
                    ))
                    .child(panel.tab_mouse_action_settings_field(
                        TabMouseActionPresentation {
                            label: "Right click".into(),
                            description: None,
                            id: "settings.interaction.tab-right",
                            current: &right_action,
                        },
                        cx,
                    ))
                    .into_any_element()
            });
            div().w(px(720.)).h(px(560.)).child(fields)
        }
    }

    fn interaction_app(cx: &mut TestAppContext, root: &Path) -> Entity<NyaTermApp> {
        let runtime = AppRuntime::from_parts_for_test(
            RuntimeMode::Portable,
            root.to_path_buf(),
            root.join("config"),
            root.join("logs"),
            root.join("cache"),
            None,
        );
        let stores = UiStoreHandles {
            startup_restore: cx.new(|_| StartupRestoreStore::default()),
            overlays: cx.new(|_| OverlayStore::default()),
        };
        let app = cx.new(|cx| NyaTermApp::new(runtime, stores, cx));
        cx.update_entity(&app, |app, cx| {
            app.sync_component_theme(cx);
            app.open_page(NavItem::Settings, cx);
            app.set_tab_mouse_action(TabMouseActionTarget::Double, TAB_MOUSE_ACTIONS[0], cx);
            app.set_tab_mouse_action(TabMouseActionTarget::Middle, TAB_MOUSE_ACTIONS[0], cx);
            app.settings
                .set_tab_right_click_action("legacy-unknown".to_string());
            // Render looks fields up rather than creating them, so the host has to
            // activate the tab the way the tab strip does and build its inputs.
            app.focus_settings_tab(SettingsTab::Interaction, cx);
            app.flush_settings_panel_snapshots(cx);
        });
        app
    }

    fn draw(vcx: &mut VisualTestContext) {
        vcx.run_until_parked();
        vcx.update(|window, cx| {
            _ = window.draw(cx);
        });
        vcx.run_until_parked();
    }

    /// Picks an option of the tab mouse-action select named by `field_id` the way a
    /// keyboard user does: focus the trigger, open it, walk the list, confirm.
    ///
    /// The three tab fields are select controls, not segmented controls: they have
    /// no in-place arrow navigation, but they do own a real open-list keyboard
    /// path. What must hold is that path — a confirmed option reaches
    /// `set_tab_mouse_action`, which normalizes and stores it.
    fn choose_select_option(vcx: &mut VisualTestContext, field_id: &'static str, navigation: &str) {
        let trigger = vcx
            .debug_bounds(field_id)
            .unwrap_or_else(|| panic!("{field_id} trigger must be laid out"));
        vcx.simulate_click(trigger.center(), Modifiers::default());
        draw(vcx);
        vcx.simulate_keystrokes(navigation);
        draw(vcx);
        vcx.simulate_keystrokes("enter");
        draw(vcx);
    }

    #[gpui::test]
    fn tab_mouse_actions_use_keyboard_selects_and_preserve_unknown_values(cx: &mut TestAppContext) {
        let temp = TestConfigDir::new("nyaterm-tab-mouse-action-selects");
        let _cleanup = crate::test_support::TestAppCleanup::new(cx);
        let app = interaction_app(cx, temp.path());
        cx.update_entity(&app, |app, _| {
            assert_eq!(
                app.settings.summary().interaction_tab_right_click_action,
                "legacy-unknown"
            );
        });
        let panel = cx.update_entity(&app, |app, _| app.settings_panel.clone());
        let (_, vcx) = cx.add_window_view(move |_, _| TabMouseActionsHost {
            panel: panel.clone(),
        });
        let vcx: &mut VisualTestContext = vcx;
        draw(vcx);

        for (target, field_id) in [
            (
                TabMouseActionTarget::Double,
                "settings.interaction.tab-double",
            ),
            (
                TabMouseActionTarget::Middle,
                "settings.interaction.tab-middle",
            ),
        ] {
            // Both fields hold "none" (row 0), so the next row is a real change.
            choose_select_option(vcx, field_id, "down");
            vcx.update(|_, cx| {
                let summary = app.read(cx).settings.summary();
                let actual = match target {
                    TabMouseActionTarget::Double => &summary.interaction_tab_double_click_action,
                    TabMouseActionTarget::Middle => &summary.interaction_tab_middle_click_action,
                    TabMouseActionTarget::Right => unreachable!(),
                };
                assert_eq!(actual, TAB_MOUSE_ACTIONS[1], "{field_id} must move one row");
            });
        }

        // The stored value was not one of the options, so the field opens with the
        // first row highlighted; confirming it replaces the stored value and
        // normalizes it.
        choose_select_option(vcx, "settings.interaction.tab-right", "");
        vcx.update(|_, cx| {
            let stored = app
                .read(cx)
                .settings
                .summary()
                .interaction_tab_right_click_action
                .clone();
            assert_ne!(stored, "legacy-unknown", "the pick must reach the store");
            assert_eq!(stored, TAB_MOUSE_ACTIONS[0]);
        });
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
