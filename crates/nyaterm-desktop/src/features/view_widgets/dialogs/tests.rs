use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use gpui::{
    AppContext as _, Context, Entity, IntoElement as _, Render, Styled as _, TestAppContext,
    VisualTestContext, Window, div,
};
use nyaterm_ui::{NyaDialogWindowExt as _, nya_root};

use super::FormSaveCompletion;
use crate::features::test_support::app_with_visible_local_session;
use crate::features::{AppLifecycleEvent, NyaTermApp};
use crate::test_support::{TestAppCleanup, TestConfigDir};

struct Host;
impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
    }
}

#[test]
fn async_form_waits_for_save_rejects_duplicates_and_closes_on_success() {
    let root = TestConfigDir::new("nyaterm-async-form");
    let mut cx = TestAppContext::single();
    let _cleanup = TestAppCleanup::new(&cx);
    let app = app_with_visible_local_session(&mut cx, root.path(), "async-form");
    app.update(&mut cx, |app, cx| app.sync_component_theme(cx));
    let host = cx.new(|_| Host);
    let (_, cx) = cx.add_window_view(move |window, cx| nya_root(host, window, cx));
    let completion = Rc::new(RefCell::new(None::<FormSaveCompletion>));
    let calls = Rc::new(Cell::new(0));
    let valid = Rc::new(Cell::new(false));
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            let completion = completion.clone();
            let calls = calls.clone();
            let valid = valid.clone();
            app.open_async_form_dialog(
                (
                    "Save".into(),
                    320.,
                    "Save".into(),
                    |_, _, _| div().into_any_element(),
                    move |_, done, _| {
                        calls.set(calls.get() + 1);
                        if !valid.get() {
                            return false;
                        }
                        *completion.borrow_mut() = Some(done);
                        true
                    },
                    |_, _| {},
                ),
                window,
                cx,
            );
        });
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    assert_eq!(calls.get(), 1);
    assert!(completion.borrow().is_none());
    valid.set(true);
    cx.simulate_keystrokes("enter");
    cx.simulate_keystrokes("enter");
    cx.simulate_keystrokes("escape");
    assert_eq!(calls.get(), 2, "pending saves must not be submitted twice");
    cx.update(|window, cx| assert!(window.has_active_nya_dialog(cx)));
    let failed = completion.borrow_mut().take().unwrap();
    app.update(cx, |_, cx| failed(false, cx));
    cx.run_until_parked();
    cx.update(|window, cx| assert!(window.has_active_nya_dialog(cx)));
    cx.simulate_keystrokes("enter");
    assert_eq!(calls.get(), 3, "failed saves must allow retry");
    let saved = completion.borrow_mut().take().unwrap();
    app.update(cx, |_, cx| saved(true, cx));
    cx.run_until_parked();
    cx.update(|window, cx| assert!(!window.has_active_nya_dialog(cx)));
}

struct QuitDialogHost;

impl Render for QuitDialogHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
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
    cx.update_entity(&app, |app, cx| app.sync_component_theme(cx));
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
