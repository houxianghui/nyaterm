use std::rc::Rc;

use gpui::{
    App, Axis, FocusHandle, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, Styled as _, Window, div, prelude::FluentBuilder as _, relative,
};
use gpui_base::{Toggle, ToggleGroup, h_flex};
use gpui_kit::component::ActiveTheme;

type NyaIndexSelectHandler = Box<dyn Fn(&usize, &mut Window, &mut App)>;
type NyaSharedIndexSelectHandler = Rc<dyn Fn(&usize, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct NyaSegmentedControl {
    id: SharedString,
    items: Vec<SharedString>,
    selected_index: Option<usize>,
    disabled: bool,
    on_select: Option<NyaIndexSelectHandler>,
}

impl NyaSegmentedControl {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            selected_index: None,
            disabled: false,
            on_select: None,
        }
    }

    pub fn items(mut self, items: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.items = items.into_iter().map(Into::into).collect();
        self
    }

    pub fn selected_index(mut self, selected_index: Option<usize>) -> Self {
        self.selected_index = selected_index;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(mut self, handler: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for NyaSegmentedControl {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let previous = self.selected_index;
        let disabled = self.disabled;
        let item_count = self.items.len();
        let id = self.id;
        let item_focuses = (0..item_count)
            .map(|index| {
                window
                    .use_keyed_state((id.clone(), index), cx, |_, cx| cx.focus_handle())
                    .read(cx)
                    .clone()
            })
            .collect::<Vec<FocusHandle>>();
        let on_select: Option<NyaSharedIndexSelectHandler> = self.on_select.map(Rc::from);
        let keyboard_on_select = on_select.clone();
        let rounding = cx.theme().radius;
        let border = cx.theme().border;
        let background = cx.theme().tokens.background;
        let pressed_background = cx.theme().tokens.accent;
        let pressed_foreground = cx.theme().accent_foreground;
        let items = self
            .items
            .into_iter()
            .enumerate()
            .map(|(index, label)| {
                let checked = previous == Some(index);
                let focus_handle = item_focuses[index].clone();
                let on_select = on_select.clone();
                let item = Toggle::new(index)
                    .pressed(checked)
                    .disabled(disabled)
                    .track_focus(&focus_handle)
                    .on_change(move |next, _, window, cx| {
                        if next
                            && Some(index) != previous
                            && let Some(handler) = &on_select
                        {
                            handler(&index, window, cx);
                        }
                    })
                    .child(label)
                    .flex()
                    .flex_row()
                    .line_height(relative(1.25))
                    .items_center()
                    .justify_center()
                    .min_w_8()
                    .h_8()
                    .px_2()
                    .text_sm()
                    .border_color(border)
                    .bg(background)
                    .when(!disabled && !checked, |this| {
                        this.hover(|this| {
                            this.bg(pressed_background).text_color(pressed_foreground)
                        })
                    })
                    .styles(|styles| {
                        styles.pressed(|style| {
                            style.bg(pressed_background).text_color(pressed_foreground)
                        })
                    });
                if item_count == 1 {
                    item.border_1().rounded(rounding)
                } else if index == 0 {
                    item.border_1().rounded_tl(rounding).rounded_bl(rounding)
                } else if index == item_count - 1 {
                    item.border_t_1()
                        .border_r_1()
                        .border_b_1()
                        .rounded_tr(rounding)
                        .rounded_br(rounding)
                } else {
                    item.border_t_1().border_r_1().border_b_1()
                }
            })
            .collect::<Vec<_>>();
        let group = ToggleGroup::new(id)
            .axis(Axis::Horizontal)
            .child(h_flex().items_center().children(items));

        div().child(group).on_key_down(move |event, window, cx| {
            if disabled || item_count == 0 {
                return;
            }
            let Some(current) = item_focuses
                .iter()
                .position(|focus| focus.is_focused(window))
            else {
                return;
            };
            let next = match event.keystroke.key.as_str() {
                "left" => current.saturating_sub(1),
                "right" => current.saturating_add(1).min(item_count - 1),
                "home" => 0,
                "end" => item_count - 1,
                _ => return,
            };

            window.prevent_default();
            cx.stop_propagation();
            item_focuses[next].focus(window, cx);
            if Some(next) != previous
                && let Some(handler) = &keyboard_on_select
            {
                handler(&next, window, cx);
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{
        Context, FocusHandle, InteractiveElement as _, IntoElement, Modifiers, ParentElement as _,
        Render, TestAppContext, VisualTestContext, Window, div, point, px,
    };

    use super::NyaSegmentedControl;

    struct SegmentedHarness {
        selected_index: Option<usize>,
        disabled: bool,
        leading_focus: Option<FocusHandle>,
        clicks: Rc<RefCell<Vec<usize>>>,
    }

    impl Render for SegmentedHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            let mut root = div();
            if let Some(leading_focus) = &self.leading_focus {
                root = root.child(div().track_focus(leading_focus));
            }
            root.child(
                NyaSegmentedControl::new("log-level")
                    .items(["Warn", "Info", "Debug"])
                    .selected_index(self.selected_index)
                    .disabled(self.disabled)
                    .on_select(cx.listener(move |this, index, _, cx| {
                        this.selected_index = Some(*index);
                        clicks.borrow_mut().push(*index);
                        cx.notify();
                    })),
            )
        }
    }

    fn harness(
        cx: &mut TestAppContext,
        selected_index: Option<usize>,
        disabled: bool,
    ) -> (&mut VisualTestContext, Rc<RefCell<Vec<usize>>>) {
        let (cx, clicks, _) = harness_with_leading_focus(cx, selected_index, disabled, false);
        (cx, clicks)
    }

    fn harness_with_leading_focus(
        cx: &mut TestAppContext,
        selected_index: Option<usize>,
        disabled: bool,
        with_leading_focus: bool,
    ) -> (
        &mut VisualTestContext,
        Rc<RefCell<Vec<usize>>>,
        Option<FocusHandle>,
    ) {
        cx.update(gpui_kit::init);
        let clicks = Rc::new(RefCell::new(Vec::new()));
        let (host, cx) = cx.add_window_view({
            let clicks = clicks.clone();
            move |_, cx| SegmentedHarness {
                selected_index,
                disabled,
                leading_focus: with_leading_focus.then(|| cx.focus_handle().tab_stop(true)),
                clicks,
            }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let leading_focus = host.read_with(cx, |host, _| host.leading_focus.clone());
        (cx, clicks, leading_focus)
    }

    fn click_second(cx: &mut VisualTestContext) {
        cx.simulate_click(point(px(60.), px(16.)), Modifiers::default());
    }

    #[gpui::test]
    fn clicking_another_item_selects_only_that_index(cx: &mut TestAppContext) {
        let (cx, clicks) = harness(cx, Some(0), false);

        click_second(cx);

        assert_eq!(clicks.borrow().as_slice(), &[1]);
    }

    #[gpui::test]
    fn disabled_control_does_not_select_on_click(cx: &mut TestAppContext) {
        let (cx, clicks) = harness(cx, Some(0), true);

        click_second(cx);

        assert!(clicks.borrow().is_empty());
    }

    #[gpui::test]
    fn focused_control_supports_right_home_and_end(cx: &mut TestAppContext) {
        let (cx, clicks) = harness(cx, Some(0), false);
        cx.update(|window, cx| window.focus_next(cx));

        cx.simulate_keystrokes("right");
        assert_eq!(clicks.borrow().as_slice(), &[1]);

        cx.simulate_keystrokes("end");
        assert_eq!(clicks.borrow().as_slice(), &[1, 2]);

        cx.simulate_keystrokes("home");
        assert_eq!(clicks.borrow().as_slice(), &[1, 2, 0]);
    }

    #[gpui::test]
    fn right_follows_focus_instead_of_a_different_selected_item(cx: &mut TestAppContext) {
        let (cx, clicks) = harness(cx, Some(2), false);
        cx.update(|window, cx| window.focus_next(cx));

        cx.simulate_keystrokes("right");

        assert_eq!(clicks.borrow().as_slice(), &[1]);
    }

    #[gpui::test]
    fn left_follows_actual_focus_for_middle_and_last_items(cx: &mut TestAppContext) {
        let (cx, clicks) = harness(cx, Some(2), false);
        cx.update(|window, cx| {
            window.focus_next(cx);
            window.focus_next(cx);
        });

        cx.simulate_keystrokes("left");
        assert_eq!(clicks.borrow().as_slice(), &[0]);

        cx.simulate_keystrokes("end left");
        assert_eq!(clicks.borrow().as_slice(), &[0, 2, 1]);
    }

    #[gpui::test]
    fn left_at_the_first_item_stays_at_the_boundary(cx: &mut TestAppContext) {
        let (cx, clicks) = harness(cx, Some(1), false);
        cx.update(|window, cx| window.focus_next(cx));

        cx.simulate_keystrokes("left left");
        assert_eq!(clicks.borrow().as_slice(), &[0]);

        cx.simulate_keystrokes("right");
        assert_eq!(clicks.borrow().as_slice(), &[0, 1]);
    }

    #[gpui::test]
    fn home_keeps_focus_on_an_already_focused_first_item(cx: &mut TestAppContext) {
        let (cx, clicks, leading_focus) = harness_with_leading_focus(cx, Some(2), false, true);
        let leading_focus = leading_focus.unwrap();
        cx.update(|window, cx| {
            window.focus_next(cx);
            window.focus_next(cx);
            assert!(!leading_focus.is_focused(window));
        });

        cx.simulate_keystrokes("home");
        assert_eq!(clicks.borrow().as_slice(), &[0]);

        cx.update(|window, cx| {
            window.focus_prev(cx);
            assert!(leading_focus.is_focused(window));
        });
    }

    #[gpui::test]
    fn empty_and_out_of_bounds_selection_render_without_panicking(cx: &mut TestAppContext) {
        struct EmptyHarness;

        impl Render for EmptyHarness {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                NyaSegmentedControl::new("empty")
            }
        }

        struct OutOfBoundsHarness;

        impl Render for OutOfBoundsHarness {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                NyaSegmentedControl::new("out-of-bounds")
                    .items(["Warn", "Info", "Debug"])
                    .selected_index(Some(9))
            }
        }

        cx.update(gpui_kit::init);
        let (_, empty_cx) = cx.add_window_view(|_, _| EmptyHarness);
        empty_cx.update(|window, cx| window.draw(cx).clear(cx));
        let (_, out_of_bounds_cx) = cx.add_window_view(|_, _| OutOfBoundsHarness);
        out_of_bounds_cx.update(|window, cx| window.draw(cx).clear(cx));
    }
}
