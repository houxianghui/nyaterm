use rust_i18n::t;

use std::rc::Rc;

use gpui::{
    AnyElement, AppContext as _, Context, Entity, ParentElement as _, Render, Subscription, Window,
    div,
};
use nyaterm_ui::{
    NyaConfirmDialog, NyaDialog, NyaDialogFooter, NyaDialogHandle, NyaDialogWindowExt,
};

use crate::features::NyaTermApp;

type FormRenderer =
    dyn Fn(&mut NyaTermApp, &mut Window, &mut Context<NyaTermApp>) -> AnyElement + 'static;
type FormSubmitHandler =
    dyn Fn(&mut NyaTermApp, &mut Window, &mut Context<NyaTermApp>) -> bool + 'static;
type AppDialogHandler = dyn Fn(&mut NyaTermApp, &mut Context<NyaTermApp>) + 'static;
type FormBusyPredicate = dyn Fn(&NyaTermApp) -> bool + 'static;
type ConfirmHandler =
    dyn Fn(&mut NyaTermApp, &mut Window, &mut Context<NyaTermApp>) -> bool + 'static;

struct AppDialogContent {
    app: Entity<NyaTermApp>,
    renderer: Rc<FormRenderer>,
    _app_subscription: Subscription,
    _release_subscription: Option<Subscription>,
}

impl AppDialogContent {
    fn new(app: Entity<NyaTermApp>, renderer: Rc<FormRenderer>, cx: &mut Context<Self>) -> Self {
        let app_subscription = cx.observe(&app, |_, _, cx| cx.notify());
        Self {
            app,
            renderer,
            _app_subscription: app_subscription,
            _release_subscription: None,
        }
    }
}

impl Render for AppDialogContent {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.app
            .update(cx, |app, cx| (self.renderer)(app, window, cx))
    }
}

struct AppDialogFooter {
    app: Entity<NyaTermApp>,
    cancel_label: String,
    action_label: String,
    busy: Rc<FormBusyPredicate>,
    cancel_while_busy: bool,
    _app_subscription: Subscription,
}

impl AppDialogFooter {
    fn new(
        app: Entity<NyaTermApp>,
        cancel_label: String,
        action_label: String,
        busy: Rc<FormBusyPredicate>,
        cx: &mut Context<Self>,
    ) -> Self {
        let app_subscription = cx.observe(&app, |_, _, cx| cx.notify());
        Self {
            app,
            cancel_label,
            action_label,
            busy,
            cancel_while_busy: false,
            _app_subscription: app_subscription,
        }
    }
}

impl Render for AppDialogFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let busy = (self.busy)(self.app.read(cx));
        NyaDialogFooter::new(self.cancel_label.clone(), self.action_label.clone())
            .disabled(busy && !self.cancel_while_busy)
            .action_disabled(busy)
            .loading(busy)
            .into_element()
    }
}

impl NyaTermApp {
    pub(in crate::features) fn open_content_dialog(
        &mut self,
        title: String,
        width: f32,
        render: impl Fn(&mut NyaTermApp, &mut Window, &mut Context<NyaTermApp>) -> AnyElement + 'static,
        on_close: impl Fn(&mut NyaTermApp, &mut Context<NyaTermApp>) + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let app = cx.entity();
        let content = cx.new(|cx| AppDialogContent::new(app.clone(), Rc::new(render), cx));
        let on_close = Rc::new(on_close);

        window.open_nya_dialog(cx, move |dialog, _, _| {
            let close_app = app.clone();
            let on_close = on_close.clone();
            let dialog = if title.is_empty() {
                dialog
            } else {
                dialog.title(title.clone())
            };
            dialog
                .width(width)
                .content(content.clone())
                .on_close(move |_, _, cx| {
                    close_app.update(cx, |app, cx| on_close(app, cx));
                })
        });
    }

    pub(in crate::features) fn open_form_dialog<R, S, C>(
        &mut self,
        spec: (String, f32, String, R, S, C),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) where
        R: Fn(&mut NyaTermApp, &mut Window, &mut Context<NyaTermApp>) -> AnyElement + 'static,
        S: Fn(&mut NyaTermApp, &mut Window, &mut Context<NyaTermApp>) -> bool + 'static,
        C: Fn(&mut NyaTermApp, &mut Context<NyaTermApp>) + 'static,
    {
        let (title, width, action_label, render, on_submit, on_cancel) = spec;
        let cancel_label = t!("common.cancel").to_string();
        let app = cx.entity();
        let content = cx
            .new(|cx| AppDialogContent::new(app.clone(), Rc::new(render) as Rc<FormRenderer>, cx));
        let on_submit = Rc::new(on_submit) as Rc<FormSubmitHandler>;
        let on_cancel = Rc::new(on_cancel) as Rc<AppDialogHandler>;

        window.open_nya_dialog(cx, move |dialog, _, _| {
            let submit_app = app.clone();
            let cancel_app = app.clone();
            let on_submit = on_submit.clone();
            let on_cancel = on_cancel.clone();
            NyaConfirmDialog::new(
                dialog.title(title.clone()).width(width),
                NyaDialogFooter::new(cancel_label.clone(), action_label.clone()),
            )
            .content(content.clone())
            .on_confirm(move |_, window, cx| {
                submit_app.update(cx, |app, cx| on_submit(app, window, cx))
            })
            .on_cancel(move |_, _, cx| {
                cancel_app.update(cx, |app, cx| on_cancel(app, cx));
                true
            })
            .into_dialog()
        });
    }

    pub(in crate::features) fn open_guarded_form_dialog<R, S, C, B>(
        &mut self,
        spec: (String, f32, String, R, S, C, B),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) where
        R: Fn(&mut NyaTermApp, &mut Window, &mut Context<NyaTermApp>) -> AnyElement + 'static,
        S: Fn(&mut NyaTermApp, &mut Window, &mut Context<NyaTermApp>) -> bool + 'static,
        C: Fn(&mut NyaTermApp, &mut Context<NyaTermApp>) + 'static,
        B: Fn(&NyaTermApp) -> bool + 'static,
    {
        self.open_guarded_form_dialog_impl(spec, false, window, cx);
    }

    /// A modal asynchronous form whose request can be invalidated by cancelling.
    pub(in crate::features) fn open_cancellable_form_dialog<R, S, C, B>(
        &mut self,
        spec: (String, f32, String, R, S, C, B),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> NyaDialogHandle
    where
        R: Fn(&mut NyaTermApp, &mut Window, &mut Context<NyaTermApp>) -> AnyElement + 'static,
        S: Fn(&mut NyaTermApp, &mut Window, &mut Context<NyaTermApp>) -> bool + 'static,
        C: Fn(&mut NyaTermApp, &mut Context<NyaTermApp>) + 'static,
        B: Fn(&NyaTermApp) -> bool + 'static,
    {
        self.open_guarded_form_dialog_impl(spec, true, window, cx)
    }

    fn open_guarded_form_dialog_impl<R, S, C, B>(
        &mut self,
        spec: (String, f32, String, R, S, C, B),
        cancel_while_busy: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> NyaDialogHandle
    where
        R: Fn(&mut NyaTermApp, &mut Window, &mut Context<NyaTermApp>) -> AnyElement + 'static,
        S: Fn(&mut NyaTermApp, &mut Window, &mut Context<NyaTermApp>) -> bool + 'static,
        C: Fn(&mut NyaTermApp, &mut Context<NyaTermApp>) + 'static,
        B: Fn(&NyaTermApp) -> bool + 'static,
    {
        let (title, width, action_label, render, on_submit, on_cancel, busy) = spec;
        let cancel_label = t!("common.cancel").to_string();
        let app = cx.entity();
        let content = cx
            .new(|cx| AppDialogContent::new(app.clone(), Rc::new(render) as Rc<FormRenderer>, cx));
        let busy = Rc::new(busy) as Rc<FormBusyPredicate>;
        let footer = cx.new(|cx| {
            let mut footer = AppDialogFooter::new(
                app.clone(),
                cancel_label.clone(),
                action_label.clone(),
                busy.clone(),
                cx,
            );
            footer.cancel_while_busy = cancel_while_busy;
            footer
        });
        let on_submit = Rc::new(on_submit) as Rc<FormSubmitHandler>;
        let on_cancel = Rc::new(on_cancel) as Rc<AppDialogHandler>;

        if cancel_while_busy {
            let weak_app = app.downgrade();
            let on_cancel = on_cancel.clone();
            content.update(cx, |content, cx| {
                content._release_subscription = Some(cx.on_release(move |_, cx| {
                    let _ = weak_app.update(cx, |app, cx| on_cancel(app, cx));
                }));
            });
        }

        window.open_tracked_nya_dialog(cx, move |dialog, window, _| {
            let submit_app = app.clone();
            let cancel_app = app.clone();
            let submit_busy = busy.clone();
            let cancel_busy = busy.clone();
            let on_submit = on_submit.clone();
            let on_cancel = on_cancel.clone();
            let button_props = NyaDialogFooter::new(cancel_label.clone(), action_label.clone());
            let dialog_width = if cancel_while_busy {
                width
                    .min(f32::from(window.viewport_size().width) - 24.)
                    .max(1.)
            } else {
                width
            };
            NyaDialog::confirm_with_footer(
                dialog
                    .title(title.clone())
                    .width(dialog_width)
                    .overlay_closable(!cancel_while_busy)
                    .close_button(false),
                button_props,
                footer.clone(),
            )
            .content(content.clone())
            .on_ok(move |_, window, cx| {
                submit_app.update(cx, |app, cx| {
                    if submit_busy(app) {
                        false
                    } else {
                        on_submit(app, window, cx)
                    }
                })
            })
            .on_cancel(move |_, _, cx| {
                cancel_app.update(cx, |app, cx| {
                    if cancel_busy(app) && !cancel_while_busy {
                        false
                    } else {
                        on_cancel(app, cx);
                        true
                    }
                })
            })
        })
    }

    pub(in crate::features) fn open_confirm_dialog<F>(
        &mut self,
        spec: (String, String, String, bool, F),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) where
        F: Fn(&mut NyaTermApp, &mut Window, &mut Context<NyaTermApp>) -> bool + 'static,
    {
        let (title, message, action_label, danger, on_confirm) = spec;
        self.open_confirm_dialog_with_cancel(
            (title, message, action_label, danger, on_confirm, |_, _| {}),
            window,
            cx,
        );
    }

    pub(in crate::features) fn open_confirm_dialog_with_cancel<F, C>(
        &mut self,
        spec: (String, String, String, bool, F, C),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) where
        F: Fn(&mut NyaTermApp, &mut Window, &mut Context<NyaTermApp>) -> bool + 'static,
        C: Fn(&mut NyaTermApp, &mut Context<NyaTermApp>) + 'static,
    {
        let (title, message, action_label, danger, on_confirm, on_cancel) = spec;
        let cancel_label = t!("common.cancel").to_string();
        let app = cx.weak_entity();
        let on_confirm = Rc::new(on_confirm) as Rc<ConfirmHandler>;
        let on_cancel = Rc::new(on_cancel) as Rc<AppDialogHandler>;
        window.open_nya_dialog(cx, move |dialog, _, _| {
            let confirm_app = app.clone();
            let cancel_app = app.clone();
            let on_confirm = on_confirm.clone();
            let on_cancel = on_cancel.clone();
            let footer = NyaDialogFooter::new(cancel_label.clone(), action_label.clone());
            let footer = if danger { footer.danger() } else { footer };
            NyaConfirmDialog::new(dialog.title(title.clone()).width(384.), footer)
                .content(div().child(message.clone()))
                .on_confirm(move |_, window, cx| {
                    confirm_app
                        .update(cx, |app, cx| on_confirm(app, window, cx))
                        .unwrap_or(false)
                })
                .on_cancel(move |_, _, cx| {
                    let _ = cancel_app.update(cx, |app, cx| on_cancel(app, cx));
                    true
                })
                .into_dialog()
        });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use gpui::{
        AppContext as _, Context, Entity, IntoElement, Render, Styled as _, TestAppContext,
        VisualTestContext, Window, div,
    };
    use nyaterm_ui::{NyaDialogWindowExt as _, nya_root};

    use crate::features::{AppLifecycleEvent, NyaTermApp};
    use crate::test_support::TestConfigDir;

    struct QuitDialogHost;

    impl Render for QuitDialogHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full()
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });
        cx.run_until_parked();
    }

    fn request_quit(cx: &mut VisualTestContext, app: &Entity<NyaTermApp>) {
        cx.update(|window, cx| {
            app.update(cx, |app, cx| {
                app.handle_window_close_request_with_count(1, window, cx);
            });
        });
        draw(cx);
        // The test host has no platform frame loop. Run the same one-shot
        // next-frame focus callback that the native window runs after drawing.
        cx.update(|window, cx| window.simulate_next_frame(cx));
        draw(cx);
    }

    #[test]
    fn quit_confirmation_cancels_and_reopens_on_cancel() {
        let test_dir = TestConfigDir::new("nyaterm-quit-confirm-focus");
        let mut cx = TestAppContext::single();
        let _cleanup = crate::test_support::TestAppCleanup::new(&cx);
        let app = crate::features::test_support::app_with_visible_local_session(
            &mut cx,
            test_dir.path(),
            "local",
        );
        let shutdown_requests = Arc::new(AtomicUsize::new(0));
        let shutdown_cancellations = Arc::new(AtomicUsize::new(0));
        let observed_requests = shutdown_requests.clone();
        let observed_cancellations = shutdown_cancellations.clone();
        let _subscription = cx.update(|cx| {
            cx.subscribe(&app, move |_, event, _| match event {
                AppLifecycleEvent::ShutdownRequested => {
                    observed_requests.fetch_add(1, Ordering::SeqCst);
                }
                AppLifecycleEvent::ShutdownCancelled => {
                    observed_cancellations.fetch_add(1, Ordering::SeqCst);
                }
                AppLifecycleEvent::NewWindowRequested => {}
            })
        });
        let (_, cx) = cx.add_window_view(|window, cx| {
            let host = cx.new(|_| QuitDialogHost);
            nya_root(host, window, cx)
        });
        let cx: &mut VisualTestContext = cx;

        request_quit(cx, &app);
        cx.update(|window, cx| {
            assert!(window.has_active_nya_dialog(cx));
            assert!(app.read(cx).session.dialog_should_quit_after_close_all());
            let initial_focus = window.focused(cx);
            assert!(
                initial_focus.is_some(),
                "the quit dialog must own focus after its first completed frame"
            );
            window.focus_next(cx);
            assert_ne!(
                window.focused(cx),
                initial_focus,
                "Tab must move from the initial footer control"
            );
        });
        cx.simulate_keystrokes("escape");
        draw(cx);
        cx.update(|window, cx| {
            assert!(!window.has_active_nya_dialog(cx));
            assert!(!app.read(cx).session.dialog_should_quit_after_close_all());
        });
        assert_eq!(shutdown_cancellations.load(Ordering::SeqCst), 1);
        assert_eq!(shutdown_requests.load(Ordering::SeqCst), 0);

        request_quit(cx, &app);
        // Raw KeyDown/KeyUp covers Dialog binding precedence as well as the
        // focused button. A dangerous Confirm would emit ShutdownRequested.
        let key = gpui::Keystroke::parse("enter").unwrap();
        cx.simulate_event(gpui::KeyDownEvent {
            keystroke: key.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(gpui::KeyUpEvent { keystroke: key });
        cx.run_until_parked();
        draw(cx);
        cx.update(|window, cx| {
            assert!(!window.has_active_nya_dialog(cx));
            assert!(!app.read(cx).session.dialog_should_quit_after_close_all());
        });
        assert_eq!(
            shutdown_cancellations.load(Ordering::SeqCst),
            2,
            "Enter on the reopened dialog must cancel again"
        );
        assert_eq!(
            shutdown_requests.load(Ordering::SeqCst),
            0,
            "Enter on Cancel must never request application shutdown"
        );
    }
}
