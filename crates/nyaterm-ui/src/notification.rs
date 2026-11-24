use std::rc::Rc;

use gpui::{App, ClickEvent, InteractiveElement as _, SharedString, Window};
use gpui_kit::component::{
    Sizable as _, WindowExt as _,
    button::Button,
    notification::{Notification, NotificationType},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NyaNotificationKind {
    Success,
    Warning,
    Error,
}

type NyaNotificationActionHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// An optional action shown on an operation notification, such as "Retry" or
/// "View Logs". Installing an action also keeps the toast open (the component
/// disables autohide when an action is present).
pub struct NyaNotificationAction {
    label: SharedString,
    /// Runs on click before the notification dismisses itself.
    on_click: NyaNotificationActionHandler,
}

impl NyaNotificationAction {
    pub fn new(
        label: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            on_click: Rc::new(on_click),
        }
    }
}

fn notification_type(kind: NyaNotificationKind) -> NotificationType {
    match kind {
        NyaNotificationKind::Success => NotificationType::Success,
        NyaNotificationKind::Warning => NotificationType::Warning,
        NyaNotificationKind::Error => NotificationType::Error,
    }
}

/// Success toasts fade out on their own; everything else stays until the user
/// closes it. An action is persistent by definition and suppresses autohide.
fn operation_autohide(kind: NyaNotificationKind, has_action: bool) -> bool {
    !has_action && kind == NyaNotificationKind::Success
}

fn operation_notification(
    key: impl Into<SharedString>,
    kind: NyaNotificationKind,
    message: impl Into<SharedString>,
    action: Option<NyaNotificationAction>,
) -> Notification {
    let has_action = action.is_some();
    let notification = Notification::new()
        .id1::<NyaNotificationKind>(key.into())
        .message(message)
        .with_type(notification_type(kind))
        .autohide(operation_autohide(kind, has_action));
    match action {
        Some(action) => {
            let NyaNotificationAction { label, on_click } = action;
            notification.action(move |_, _, cx| {
                let entity = cx.entity();
                let on_click = on_click.clone();
                Button::new("nya-notification-action")
                    .label(label.clone())
                    .small()
                    .debug_selector(|| "nya-notification-action".to_string())
                    .on_click(move |event, window, cx| {
                        on_click(event, window, cx);
                        entity.update(cx, |notification, cx| {
                            notification.dismiss(window, cx);
                        });
                    })
            })
        }
        None => notification,
    }
}

/// In-app operation feedback. Stable operation keys replace duplicate toasts.
pub trait NyaNotificationWindowExt {
    fn notify_operation(
        &mut self,
        key: impl Into<SharedString>,
        kind: NyaNotificationKind,
        message: impl Into<SharedString>,
        cx: &mut App,
    );

    /// Same contract as [`Self::notify_operation`], with an action button.
    ///
    /// The action runs first and then closes the toast, so callers keep the
    /// old API unchanged and opt into retry/detail affordances per call site.
    fn notify_operation_with_action(
        &mut self,
        key: impl Into<SharedString>,
        kind: NyaNotificationKind,
        message: impl Into<SharedString>,
        action: NyaNotificationAction,
        cx: &mut App,
    );
}

impl NyaNotificationWindowExt for Window {
    fn notify_operation(
        &mut self,
        key: impl Into<SharedString>,
        kind: NyaNotificationKind,
        message: impl Into<SharedString>,
        cx: &mut App,
    ) {
        let notification = operation_notification(key, kind, message, None);
        self.push_notification(notification, cx);
    }

    fn notify_operation_with_action(
        &mut self,
        key: impl Into<SharedString>,
        kind: NyaNotificationKind,
        message: impl Into<SharedString>,
        action: NyaNotificationAction,
        cx: &mut App,
    ) {
        let notification = operation_notification(key, kind, message, Some(action));
        self.push_notification(notification, cx);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{
        AppContext as _, ClickEvent, Context, IntoElement, Modifiers, Render, TestAppContext,
        VisualTestContext, Window, div,
    };

    use super::{
        NyaNotificationAction, NyaNotificationKind, notification_type, operation_autohide,
    };
    use gpui_kit::component::notification::NotificationType;

    struct NotificationHost;

    impl Render for NotificationHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
    }

    #[gpui::test]
    fn operation_action_button_invokes_once_and_dismisses(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        cx.update(|cx| cx.set_reduce_motion(true));
        let (_, cx) = cx.add_window_view(|window, cx| {
            crate::nya_root(cx.new(|_| NotificationHost), window, cx)
        });
        let invoked = Rc::new(Cell::new(0));
        let count = invoked.clone();
        cx.update(|window, cx| {
            use super::NyaNotificationWindowExt as _;
            window.notify_operation_with_action(
                "retry-test",
                NyaNotificationKind::Error,
                "Connection failed",
                NyaNotificationAction::new("Retry", move |_, _, _| {
                    count.set(count.get() + 1);
                }),
                cx,
            );
        });
        draw(cx);
        let action = cx
            .debug_bounds("nya-notification-action")
            .expect("the notification action button must render");
        cx.simulate_click(action.center(), Modifiers::default());
        cx.background_executor
            .advance_clock(std::time::Duration::from_millis(250));
        draw(cx);

        assert_eq!(invoked.get(), 1);
        assert!(
            cx.debug_bounds("nya-notification-action").is_none(),
            "activating the action must dismiss its notification"
        );
    }

    #[test]
    fn notification_kinds_map_to_component_types() {
        assert!(matches!(
            notification_type(NyaNotificationKind::Success),
            NotificationType::Success
        ));
        assert!(matches!(
            notification_type(NyaNotificationKind::Warning),
            NotificationType::Warning
        ));
        assert!(matches!(
            notification_type(NyaNotificationKind::Error),
            NotificationType::Error
        ));
    }

    #[test]
    fn actions_suppress_autohide_and_non_success_errors_persist() {
        assert!(operation_autohide(NyaNotificationKind::Success, false));
        assert!(!operation_autohide(NyaNotificationKind::Success, true));
        assert!(!operation_autohide(NyaNotificationKind::Error, false));
        assert!(!operation_autohide(NyaNotificationKind::Error, true));
    }

    #[test]
    fn action_keeps_its_label_and_invokes_the_handler() {
        let invoked = Rc::new(Cell::new(0));
        let count = invoked.clone();
        let action = NyaNotificationAction::new("Retry", move |_, _, _| {
            count.set(count.get() + 1);
        });
        assert_eq!(action.label.as_ref(), "Retry");

        let mut cx = TestAppContext::single();
        let window = cx.add_empty_window();
        window.update(|window, cx| {
            (action.on_click)(&ClickEvent::default(), window, cx);
        });
        assert_eq!(invoked.get(), 1);
    }
}
