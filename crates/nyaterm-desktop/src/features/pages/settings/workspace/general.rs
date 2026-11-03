use rust_i18n::t;

use gpui::{Context, IntoElement, SharedString, div, prelude::*};
use nyaterm_ui::NyaSelectOption;

use crate::features::pages::settings::panel::SettingsPanel;
use crate::models::HeaderStatusMode;
use crate::widgets::small_button;

use super::super::{settings_form_row, settings_form_section, settings_switch};

fn startup_restore_nested_row() -> gpui::Div {
    div().pl_3().ml_1()
}

impl SettingsPanel {
    pub(in crate::features) fn general_settings_section(
        &mut self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let palette = self.theme_palette();
        // Tauri GeneralTab: language, nested startup layout, tray, confirm, diagnostics.
        let language_value = crate::i18n::normalize_locale(&self.settings.summary().language);
        let diagnostics_level = self.settings.summary().diagnostics_level.clone();
        let retention = self.settings.summary().diagnostics_retention_days;
        let days_unit = t!("common.days");
        let header_status_mode =
            HeaderStatusMode::from_setting(&self.settings.summary().ui_header_status_mode);
        let header_status_visible = self.settings.summary().ui_header_status_visible;
        let header_status_value = if header_status_visible {
            header_status_mode.persistence_id()
        } else {
            "hidden"
        };

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(settings_form_section(
                palette,
                None,
                None,
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(settings_form_row(
                        palette,
                        t!("settings.language"),
                        None,
                        self.settings_select_control(
                            "settings.general.language",
                            crate::i18n::language_options(),
                            language_value,
                            false,
                            cx,
                        ),
                    ))
                    .child(settings_form_row(
                        palette,
                        t!("settings.headerStatus"),
                        Some(SharedString::from(t!("settings.headerStatusDesc"))),
                        self.settings_select_control(
                            "settings.general.header-status",
                            std::iter::once(NyaSelectOption::new(
                                "hidden",
                                t!("headerStatus.hidden"),
                            ))
                            .chain(HeaderStatusMode::ALL.into_iter().map(|mode| {
                                NyaSelectOption::new(mode.persistence_id(), t!(mode.i18n_key()))
                            }))
                            .collect(),
                            header_status_value,
                            false,
                            cx,
                        ),
                    )),
            ))
            .child(settings_form_section(
                palette,
                None,
                None,
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(settings_form_row(
                        palette,
                        t!("settings.startupRestore"),
                        Some(SharedString::from(t!("settings.startupRestoreDesc"))),
                        settings_switch(
                            palette,
                            "general-startup-restore",
                            self.settings.summary().startup_restore,
                            cx.listener(|this, _, _, cx| {
                                this.toggle_startup_restore(cx);
                            }),
                        ),
                    ))
                    .when(self.settings.summary().startup_restore, |this| {
                        this.child(startup_restore_nested_row().child(settings_form_row(
                            palette,
                            t!("settings.startupRestoreWindowLayout"),
                            Some(SharedString::from(t!(
                                "settings.startupRestoreWindowLayoutDesc"
                            ))),
                            settings_switch(
                                palette,
                                "general-startup-restore-window-layout",
                                self.settings.summary().startup_restore_window_layout,
                                cx.listener(|this, _, _, cx| {
                                    this.toggle_startup_restore_window_layout(cx);
                                }),
                            ),
                        )))
                    })
                    .child(settings_form_row(
                        palette,
                        t!("settings.minimizeToTray"),
                        Some(SharedString::from(t!("settings.minimizeToTrayDesc"))),
                        settings_switch(
                            palette,
                            "general-minimize-to-tray",
                            self.settings.summary().minimize_to_tray,
                            cx.listener(|this, _, _, cx| {
                                this.toggle_minimize_to_tray(cx);
                            }),
                        ),
                    ))
                    .child(settings_form_row(
                        palette,
                        t!("settings.confirmOnClose"),
                        Some(SharedString::from(t!("settings.confirmOnCloseDesc"))),
                        settings_switch(
                            palette,
                            "general-confirm-close",
                            self.settings.summary().confirm_on_close,
                            cx.listener(|this, _, _, cx| {
                                this.toggle_confirm_on_close(cx);
                            }),
                        ),
                    )),
            ))
            .child(settings_form_section(
                palette,
                Some(t!("settings.diagnostics")),
                Some(t!("settings.diagnosticsDesc")),
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(settings_form_row(
                        palette,
                        t!("settings.logLevel"),
                        Some(SharedString::from(t!("settings.logLevelDesc"))),
                        self.settings_select_control(
                            "settings.general.diagnostics-level",
                            vec![
                                NyaSelectOption::new("warn", t!("settings.logLevelWarn")),
                                NyaSelectOption::new("info", t!("settings.logLevelInfo")),
                                NyaSelectOption::new("debug", t!("settings.logLevelDebug")),
                            ],
                            diagnostics_level,
                            false,
                            cx,
                        ),
                    ))
                    .child(settings_form_row(
                        palette,
                        t!("settings.logRetention"),
                        Some(SharedString::from(t!("settings.logRetentionDesc"))),
                        self.settings_select_control(
                            "settings.general.diagnostics-retention",
                            [3_u32, 7, 14, 30]
                                .into_iter()
                                .map(|days| {
                                    NyaSelectOption::new(
                                        days.to_string(),
                                        format!("{days} {days_unit}"),
                                    )
                                })
                                .collect(),
                            retention.to_string(),
                            false,
                            cx,
                        ),
                    ))
                    .child(div().flex().justify_end().child(small_button(
                        palette,
                        "general-open-logs",
                        t!("settings.openLogs"),
                        cx.listener(|this, _, _, cx| {
                            this.reveal_log_dir(cx);
                        }),
                    )))
                    .child(settings_form_row(
                        palette,
                        t!("settings.exportDiagnostics"),
                        Some(SharedString::from(t!("settings.exportDiagnosticsDesc"))),
                        small_button(
                            palette,
                            "general-export-diagnostics",
                            t!("settings.exportDiagnostics"),
                            cx.listener(|this, _, _, cx| {
                                this.prompt_diagnostics_export(cx);
                            }),
                        ),
                    )),
            ))
    }
}

#[cfg(test)]
mod tests {
    use gpui::{
        Background, IntoElement, ParentElement as _, Render, Styled as _, TestAppContext,
        VisualTestContext, div, px, rgb,
    };

    struct NestedRowFixture;

    impl Render for NestedRowFixture {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            div().child(
                super::startup_restore_nested_row()
                    .size(px(40.))
                    .bg(rgb(0xffffff)),
            )
        }
    }

    #[gpui::test]
    fn startup_restore_nested_row_paints_without_a_vertical_border(cx: &mut TestAppContext) {
        let _cleanup = crate::test_support::TestAppCleanup::new(cx);
        let (_, cx) = cx.add_window_view(|_, _| NestedRowFixture);
        let cx: &mut VisualTestContext = cx;
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
            let quad = window
                .painted_quads()
                .into_iter()
                .find(|quad| quad.background == Background::from(rgb(0xffffff)))
                .expect("test background should be painted");
            assert_eq!(quad.border_widths.left, px(0.).scale(window.scale_factor()));
        });
    }
}
