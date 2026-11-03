use std::rc::Rc;

use gpui::{
    App, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, SharedString,
    Window, div,
};
use gpui_kit::component::{
    Disableable,
    button::{Toggle, ToggleGroup, ToggleVariant, ToggleVariants},
};

type NyaIndexSelectHandler = Box<dyn Fn(&usize, &mut Window, &mut App)>;

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
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let previous = self.selected_index;
        let disabled = self.disabled;
        let item_count = self.items.len();
        let on_select: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>> =
            self.on_select.map(Rc::from);
        let keyboard_on_select = on_select.clone();
        let group = ToggleGroup::new(self.id)
            .segmented()
            .with_variant(ToggleVariant::Outline)
            .disabled(disabled)
            .children(self.items.into_iter().enumerate().map(|(index, label)| {
                Toggle::new(index)
                    .label(label)
                    .checked(previous == Some(index))
            }))
            .on_click(move |checks, window, cx| {
                let next = checks.iter().enumerate().find_map(|(index, checked)| {
                    (*checked && Some(index) != previous).then_some(index)
                });
                if let (Some(index), Some(handler)) = (next, &on_select) {
                    handler(&index, window, cx);
                }
            });

        div().child(group).on_key_down(move |event, window, cx| {
            if disabled || item_count == 0 {
                return;
            }
            let current = previous.filter(|index| *index < item_count).unwrap_or(0);
            let next = match event.keystroke.key.as_str() {
                "right" => current.saturating_add(1).min(item_count - 1),
                "home" => 0,
                "end" => item_count - 1,
                _ => return,
            };

            window.prevent_default();
            cx.stop_propagation();
            match next.cmp(&current) {
                std::cmp::Ordering::Less => {
                    for _ in next..current {
                        window.focus_prev(cx);
                    }
                }
                std::cmp::Ordering::Greater => {
                    for _ in current..next {
                        window.focus_next(cx);
                    }
                }
                std::cmp::Ordering::Equal => {}
            }
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
        Context, IntoElement, Modifiers, ParentElement as _, Render, TestAppContext,
        VisualTestContext, Window, div, point, px,
    };

    use super::NyaSegmentedControl;

    struct SegmentedHarness {
        selected_index: Option<usize>,
        disabled: bool,
        clicks: Rc<RefCell<Vec<usize>>>,
    }

    impl Render for SegmentedHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            div().child(
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
        cx.update(gpui_kit::init);
        let clicks = Rc::new(RefCell::new(Vec::new()));
        let (_, cx) = cx.add_window_view({
            let clicks = clicks.clone();
            move |_, _| SegmentedHarness {
                selected_index,
                disabled,
                clicks,
            }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        (cx, clicks)
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
