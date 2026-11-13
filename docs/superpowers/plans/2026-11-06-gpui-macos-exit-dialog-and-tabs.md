# GPUI macOS 退出对话框与终端页签交互改进 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让退出确认表现得像 macOS 的 sheet——默认焦点落在“取消”、Enter 不会误确认、Escape 保持取消、危险动作用主题强调色而不是红色实心块——同时减轻终端页签的选中态，收敛状态颜色，并锁定现有页签快捷键与关闭按钮的键盘可达性。

**Architecture:** 对话框继续使用 gpui-kit 的 `Dialog` focus trap、`close_on_escape`、`Cancel`/`Confirm` 键位，只在 `open_tracked_nya_dialog` 构建完成后为确认框补一次延迟 `focus_next`。危险按钮直接复用 gpui-kit 的 Primary 样式，不引入新的颜色系统或扩散 `cx` 参数。页签改动集中在 `tabs.rs` 的纯样式函数与渲染；快捷键继续走现有 `ShortcutId::NextTab` / `PreviousTab` / `CloseTab` action 路径，不再添加原始键事件处理器。

**Tech Stack:** Rust 2024、GPUI（fork `nyakang/zed` rev `c054331`）、gpui-kit（rev `5aca244`）`Dialog`/`Button`、rust-i18n。

**Spec:** `docs/superpowers/specs/2026-11-03-gpui-macos-interaction-design.md`（该 spec 未逐项覆盖退出对话框与页签细节，本计划由 2026-11-05 的逐页交互审查结论补充；凡与 spec 冲突之处以 spec 为准）

## Global Constraints

- 不改动 `open_confirm_dialog_with_cancel` 的签名，也不改任何调用方的确认/取消语义；`confirm_on_close` 仍表示“关闭时仍有会话才弹确认”。
- 保留现有平台快捷键：Windows/Linux 为 `Ctrl+Tab`、`Ctrl+Shift+Tab`、`Ctrl+Shift+W`；macOS 为 `Cmd+Tab`、`Cmd+Shift+Tab`、`Cmd+W` / `Cmd+Shift+W`。不得另加 `on_key_down` 造成双重触发，也不得新增重复 i18n 键。
- 终端页签的编号、连接图标、自定义颜色圆点、同步角标、未读圆点、hover card、拖拽预览、拖拽插入指示、中键/右键/双击行为、锁定页签不可关闭的行为全部保留。
- 本机可运行 `nyaterm-ui` 测试；desktop 测试二进制若仍被系统拒绝执行（`os error 5`），必须如实记录“编译通过、测试未执行”，由 CI 补跑，不得声称本地测试通过。
- 本地打包与真实 UI 验收一律使用 release，不用 debug 程序验收。
- 提交信息使用英文类型前缀和中文正文，并附 `Co-Authored-By: Claude Code <noreply@anthropic.com>`。

## Review Focus

- 打开退出确认后立即按 Enter 必须取消而不是退出：Task 1 组件测试与 Task 3 应用层测试共同覆盖。
- 焦点已移到确认按钮时，Escape 仍取消；取消后再次打开时焦点重新回到取消：Task 3 覆盖。
- 带输入框的确认表单应聚焦第一个实际控件，不能因默认焦点策略而失效：Task 1 的输入框测试覆盖。
- 自定义页签颜色的 alpha 必须精确保留，断开状态优先于未读/激活状态：Task 4、Task 5 的纯函数测试覆盖。
- 关闭按钮在视觉上隐藏时仍是 Tab 停靠点，获得焦点后同时显示按钮与焦点环：Task 7 覆盖。

---

### Task 1: 确认对话框打开时聚焦第一个控件

**Files:**
- Modify: `crates/nyaterm-ui/src/dialog.rs`（`NyaDialog` 增加 opt-in 标志；`open_tracked_nya_dialog` 聚焦）
- Test: `crates/nyaterm-ui/src/root.rs` 内现有 `#[cfg(test)]` 模块（复用 `RootContentFixture` 与 `draw`）

**Interfaces:**
- Consumes: `gpui_kit::component::WindowExt::open_dialog`、`gpui::Window::focus_next`、现有 `nya_root` 测试宿主。
- Produces:
  - `NyaDialog::focus_first_control(self, focus: bool) -> Self`
  - `NyaConfirmDialog::new` 内部设置该标志为 `true`；直接构造的 `NyaDialog` 默认仍为 `false`。
  - 不改变 `NyaConfirmDialog::new`、`confirm`、`into_element` 的签名。

**已验证的实现事实：**
- gpui-kit 的 `Button` 没有 `autofocus`，其焦点句柄是私有 keyed state。
- `RootState::open_dialog` 打开对话框时把焦点放在对话框根；无输入框的确认框中，第一次 `focus_next` 到达页脚取消按钮。
- 带输入框时，第一次 `focus_next` 到达正文第一个输入框，这正是表单的预期行为。
- 对话框测试必须使用 `nya_root`，不能用没有 component root 的 `add_empty_window`。

- [x] **Step 1: 写退出确认与表单的失败测试**

在 `root.rs` 测试模块的 `crate::{...}` 导入中补 `NyaConfirmDialog`，然后加入：

```rust
#[gpui::test]
fn confirm_dialog_opens_with_focus_on_cancel(cx: &mut TestAppContext) {
    use std::cell::Cell;
    use std::rc::Rc;

    cx.update(gpui_kit::init);
    let (_, cx) = cx.add_window_view(|window, cx| {
        let content = cx.new(|_| RootContentFixture);
        nya_root(content, window, cx)
    });
    let cancelled = Rc::new(Cell::new(0));
    let cancelled_in_dialog = cancelled.clone();
    cx.update(|window, cx| {
        window.open_nya_dialog(cx, move |dialog, _, _| {
            let cancelled = cancelled_in_dialog.clone();
            NyaConfirmDialog::new(
                dialog.title("Quit").width(384.),
                NyaDialogFooter::new("Cancel", "Quit").danger(),
            )
            .content(div().child("Quit?"))
            .on_cancel(move |_, _, _| {
                cancelled.set(cancelled.get() + 1);
                true
            })
            .into_dialog()
        });
    });
    draw(cx);

    let tag = cx.update(|window, cx| {
        window.focused(cx).map(|handle| handle.id().tag())
    });
    assert_eq!(tag.as_deref(), Some("nya-dialog-cancel"));

    cx.simulate_keystrokes("enter");
    draw(cx);
    assert_eq!(cancelled.get(), 1, "Enter on Cancel must cancel");
    cx.update(|window, cx| assert!(!window.has_active_nya_dialog(cx)));
}

#[gpui::test]
fn confirm_form_focuses_its_first_input(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let field = cx.new(|cx| NyaInputState::new(cx, ""));
    let (_, cx) = cx.add_window_view(|window, cx| {
        let content = cx.new(|_| RootContentFixture);
        nya_root(content, window, cx)
    });
    let input = field.clone();
    cx.update(|window, cx| {
        window.open_nya_dialog(cx, move |dialog, _, _| {
            NyaConfirmDialog::new(
                dialog.title("Form"),
                NyaDialogFooter::new("Cancel", "Save"),
            )
            .content(NyaInputShell::new("confirm-first-input", &input))
            .into_dialog()
        });
    });
    draw(cx);

    cx.update(|window, cx| {
        assert_eq!(
            window.focused(cx),
            Some(field.read(cx).component_focus_handle(cx)),
            "the first real form control receives focus"
        );
    });
}
```

- [x] **Step 2: 运行确认失败**

Run: `cargo test -p nyaterm-ui --lib confirm_`
Expected: 两个测试均 FAIL；当前焦点仍在对话框根，不在取消按钮或第一个输入框。

- [x] **Step 3: 实现 opt-in 默认焦点**

在 `dialog.rs` 修改结构与构造：

```rust
pub struct NyaDialog {
    inner: Dialog,
    focus_first_control: bool,
}

impl NyaDialog {
    fn from_component(inner: Dialog) -> Self {
        Self {
            inner,
            focus_first_control: false,
        }
    }

    /// Focus the first Tab stop inside the dialog once it opens.
    pub fn focus_first_control(mut self, focus: bool) -> Self {
        self.focus_first_control = focus;
        self
    }
}
```

`NyaConfirmDialog::new` 改为：

```rust
    pub fn new(dialog: NyaDialog, footer: NyaDialogFooter) -> Self {
        Self {
            dialog: dialog.confirm(footer).focus_first_control(true),
        }
    }
```

`open_tracked_nya_dialog` 的构建闭包改为：

```rust
        self.open_dialog(cx, move |dialog, window, cx| {
            let _lifetime = &marker;
            let built = build(NyaDialog::from_component(dialog), window, cx);
            if built.focus_first_control {
                window.defer(cx, |window, cx| {
                    window.focus_next(cx);
                });
            }
            built
                .into_component()
                .shadow(gpui_base::Theme::global(cx).tokens.shadow.lg)
        });
```

- [x] **Step 4: 运行通过**

Run: `cargo test -p nyaterm-ui --lib confirm_`
Expected: PASS；随后运行 `cargo test -p nyaterm-ui --lib`，现有对话框与焦点测试不回归。

- [x] **Step 5: 提交**

```bash
cd F:/workspace/nyaterm
cargo fmt -p nyaterm-ui && cargo clippy -p nyaterm-ui --all-targets
git add crates/nyaterm-ui/src/dialog.rs crates/nyaterm-ui/src/root.rs
git commit -m "feat(ui): 确认对话框打开时聚焦首个控件" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"
```

---

### Task 2: 危险确认改用主题强调色按钮

**Files:**
- Modify: `crates/nyaterm-ui/src/dialog.rs`（`NyaDialogFooter::button_props` 与 `into_footer`）
- Test: `crates/nyaterm-ui/src/root.rs` 内现有测试模块

**Interfaces:**
- Consumes: gpui-kit 的 `ButtonVariants::primary`、当前主题的 `tokens.primary` / `tokens.destructive`。
- Produces: `NyaDialogFooter::danger()` 继续表达业务语义，但确认按钮与键盘 Confirm 都使用 Primary 强调色；公开签名不变。

- [x] **Step 1: 写绘制断言失败测试**

```rust
#[gpui::test]
fn dangerous_confirm_uses_primary_fill_without_danger_fill(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let (_, cx) = cx.add_window_view(|window, cx| {
        let content = cx.new(|_| RootContentFixture);
        nya_root(content, window, cx)
    });
    cx.update(|window, cx| {
        window.open_nya_dialog(cx, |dialog, _, _| {
            NyaConfirmDialog::new(
                dialog.title("Quit").width(384.),
                NyaDialogFooter::new("Cancel", "Quit").danger(),
            )
            .content(div().child("Active sessions will be disconnected. Quit?"))
            .into_dialog()
        });
    });
    draw(cx);

    let action = cx
        .debug_bounds("nya-dialog-action-button")
        .expect("the action button draws");
    let (primary, destructive) = cx.update(|cx| {
        let theme = gpui_kit::component::ActiveTheme::theme(cx);
        (theme.tokens.primary, theme.tokens.destructive)
    });
    cx.update(|window, _| {
        let quads = window.painted_quads();
        assert!(
            !quads.iter().any(|quad| {
                quad.bounds.intersects(&action)
                    && quad.background == gpui::Background::from(destructive)
            }),
            "the dangerous action must not paint a solid danger fill"
        );
        assert!(
            quads.iter().any(|quad| {
                quad.bounds.intersects(&action)
                    && quad.background == gpui::Background::from(primary)
            }),
            "the dangerous action keeps the theme's prominent fill"
        );
    });
}
```

- [x] **Step 2: 运行确认失败**

Run: `cargo test -p nyaterm-ui --lib dangerous_confirm_uses_primary_fill`
Expected: FAIL；当前危险按钮绘制 `destructive` 而不是 `primary`。

- [x] **Step 3: 复用 Primary 样式**

`button_props()` 的末尾固定使用 Primary，让键盘 Confirm 与页脚按钮一致：

```rust
    fn button_props(&self) -> DialogButtonProps {
        DialogButtonProps::default()
            .cancel_text(self.cancel_label.clone())
            .ok_text(self.action_label.clone())
            .show_cancel(true)
            .ok_variant(ButtonVariant::Primary)
    }
```

`into_footer()` 删除 `action_variant` 局部变量，动作按钮改为：

```rust
        let action = Button::new("nya-dialog-action")
            .label(self.action_label)
            .primary()
            .loading(self.loading)
            .disabled(self.disabled || self.action_disabled)
            .debug_selector(|| "nya-dialog-action-button".to_string());
```

不增加 `ButtonCustomVariant`，不给 `into_footer` / `into_element` 增加 `cx` 参数。

- [x] **Step 4: 运行通过**

Run: `cargo test -p nyaterm-ui --lib dangerous_confirm_uses_primary_fill && cargo test -p nyaterm-ui --lib`
Expected: PASS。

- [x] **Step 5: 提交**

```bash
cd F:/workspace/nyaterm
cargo fmt -p nyaterm-ui && cargo clippy -p nyaterm-ui --all-targets
git add crates/nyaterm-ui/src/dialog.rs crates/nyaterm-ui/src/root.rs
git commit -m "feat(ui): 危险确认使用主题强调色按钮" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"
```

---

### Task 3: 应用层锁定退出确认的取消与重开行为

**Files:**
- Test: `crates/nyaterm-desktop/src/features/view_widgets/dialogs.rs` 内新增 `#[cfg(test)]` 模块

**Interfaces:**
- Consumes: Task 1 的 `NyaConfirmDialog::new`、`app_with_visible_local_session`、`handle_window_close_request_with_count`。
- Produces: 无新生产 API；`open_confirm_dialog_with_cancel` 的签名与回调语义保持不变。

- [x] **Step 1: 确认退出路径已经经过统一组件**

Run: `rg -n "NyaConfirmDialog::new|open_confirm_dialog_with_cancel" crates/nyaterm-desktop/src/features/view_widgets/dialogs.rs crates/nyaterm-desktop/src/features/terminal/terminal_runtime/sessions.rs`
Expected: `open_close_all_sessions_confirm` 调用 `open_confirm_dialog_with_cancel`，后者使用 `NyaConfirmDialog::new`；无需修改生产调用点。

- [x] **Step 2: 添加应用层集成测试**

在 `dialogs.rs` 末尾添加：

```rust
#[cfg(test)]
mod tests {
    use gpui::{
        AppContext as _, Context, Entity, IntoElement, Render, Styled as _, TestAppContext,
        VisualTestContext, Window, div,
    };
    use nyaterm_ui::{NyaDialogWindowExt as _, nya_root};

    use crate::features::NyaTermApp;
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
    }

    fn focused_tag(cx: &mut VisualTestContext) -> Option<String> {
        cx.update(|window, cx| {
            window.focused(cx).map(|handle| handle.id().tag())
        })
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
        let (_, cx) = cx.add_window_view(|window, cx| {
            let host = cx.new(|_| QuitDialogHost);
            nya_root(host, window, cx)
        });
        let cx: &mut VisualTestContext = cx;

        request_quit(cx, &app);
        assert_eq!(focused_tag(cx).as_deref(), Some("nya-dialog-cancel"));

        cx.update(|window, cx| window.focus_next(cx));
        assert_eq!(focused_tag(cx).as_deref(), Some("nya-dialog-action"));
        cx.simulate_keystrokes("escape");
        draw(cx);
        cx.update(|window, cx| {
            assert!(!window.has_active_nya_dialog(cx));
            assert!(!app.read(cx).session.dialog_should_quit_after_close_all());
        });

        request_quit(cx, &app);
        assert_eq!(
            focused_tag(cx).as_deref(),
            Some("nya-dialog-cancel"),
            "a reopened quit sheet resets focus to Cancel"
        );
        cx.simulate_keystrokes("enter");
        draw(cx);
        cx.update(|window, cx| {
            assert!(!window.has_active_nya_dialog(cx));
            assert!(!app.read(cx).session.dialog_should_quit_after_close_all());
        });
    }
}
```

该测试使用 desktop 测试常见的 `#[test] + TestAppContext::single()` 形式；`nya_root` 是对话框所必需的宿主，不能换成空窗口。

- [ ] **Step 3: 运行集成测试**

Run: `cargo test -p nyaterm-desktop --lib quit_confirmation_cancels_and_reopens_on_cancel`
Expected: PASS（Task 1 已提供默认焦点；本测试是应用路径的集成锁定，不伪造一次 Task 1 完成前的 RED）。本机若仍为 `os error 5`，记录“编译通过、未执行”，由 CI 验证。

- [x] **Step 4: 提交**

```bash
cd F:/workspace/nyaterm
cargo fmt -p nyaterm-desktop && cargo check -p nyaterm-desktop
git add crates/nyaterm-desktop/src/features/view_widgets/dialogs.rs
git commit -m "test(dialogs): 锁定退出确认取消与重开焦点" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"
```

---

### Task 4: 页签胶囊减重

**Files:**
- Modify: `crates/nyaterm-desktop/src/features/layout/workspace/surface/tabs.rs`（`session_strip_tab` 的填充、边框、阴影与字重）
- Test: 同文件现有 `#[cfg(test)]` 模块

**Interfaces:**
- Consumes: `crate::theme::ThemePalette` 的 `bg` / `hover` 与 GPUI `Hsla`。
- Produces: `fn session_tab_background(palette: &ThemePalette, active: bool, custom: Option<u32>) -> Hsla`，供渲染与单元测试共同使用。

- [x] **Step 1: 写失败测试**

在测试导入中补 `Hsla`、`rgba` 与 `session_tab_background`：

```rust
#[test]
fn active_tab_is_a_subtle_fill_and_inactive_tabs_are_transparent() {
    let palette = crate::theme::theme_palette("github-dark");
    assert_eq!(session_tab_background(&palette, false, None).a, 0.);
    assert_eq!(
        session_tab_background(&palette, true, None),
        Hsla::from(rgb(palette.bg))
    );
    assert_ne!(
        session_tab_background(&palette, true, None),
        Hsla::from(rgb(palette.surface))
    );
}

#[test]
fn custom_color_tabs_keep_their_exact_tint_alpha() {
    let palette = crate::theme::theme_palette("github-dark");
    assert_eq!(
        session_tab_background(&palette, true, Some(0x336699)),
        rgba((0x336699 << 8) | 0x24)
    );
    assert_eq!(
        session_tab_background(&palette, false, Some(0x336699)),
        rgba((0x336699 << 8) | 0x14)
    );
}
```

- [x] **Step 2: 运行确认失败**

Run: `cargo test -p nyaterm-desktop --lib active_tab_is_a_subtle_fill`
Expected: 编译失败，`session_tab_background` 未定义。本机无法执行时保留编译失败证据，不声称已运行。

- [x] **Step 3: 实现并接入渲染**

```rust
/// Selected tabs use a quiet fill; unselected tabs leave the strip visible.
fn session_tab_background(
    palette: &crate::theme::ThemePalette,
    active: bool,
    custom: Option<u32>,
) -> Hsla {
    match (custom, active) {
        (Some(color), true) => rgba((color << 8) | 0x24),
        (Some(color), false) => rgba((color << 8) | 0x14),
        (None, true) => rgb(palette.bg),
        (None, false) => rgba(0x00000000),
    }
}
```

在文件顶部的 `gpui::{...}` 导入中补 `Hsla`。用该函数替换 `session_strip_tab` 的 `bg` 计算，并保留自定义颜色的现有 hover 分支：

```rust
let hover_bg = if let Some(custom_color) = custom_color {
    rgba((custom_color << 8) | if is_active { 0x32 } else { 0x22 })
} else {
    rgba((palette.hover << 8) | 0x1a)
};
```激活分支保留 `session-tab-active-*` 调试子元素与 1px 几何边界，但将边框改为透明并删除 `shadow_sm()`：

```rust
.when(is_active, |this| {
    this.border_1()
        .border_color(rgba(0x00000000))
        .child(
            div()
                .debug_selector(|| format!("session-tab-active-{session_id}"))
                .size(px(0.)),
        )
})
```

普通会话页签编号字重从 700 改为 600，激活标题从 700 改为 600；pending/failed 页签不在本任务改样式。

- [ ] **Step 4: 运行通过**

Run: `cargo test -p nyaterm-desktop --lib session_tab_background`
Expected: PASS；desktop 测试无法本机执行时记录并由 CI 补跑。

- [x] **Step 5: 提交**

```bash
cd F:/workspace/nyaterm
cargo fmt -p nyaterm-desktop && cargo check -p nyaterm-desktop
git add crates/nyaterm-desktop/src/features/layout/workspace/surface/tabs.rs
git commit -m "feat(tabs): 选中页签改用轻填充并降低字重" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"
```

---

### Task 5: 状态样式收敛（断开 / 未读 / 同步）

**Files:**
- Modify: `crates/nyaterm-desktop/src/features/layout/workspace/surface/tabs.rs`（accent、标题色、未读圆点）
- Test: 同文件现有测试模块

**Interfaces:**
- Consumes: Task 4 的 `ThemePalette`，不依赖 Task 4 的函数。
- Produces:
  - `fn session_tab_accent(palette: &ThemePalette, active: bool, disconnected: bool, unread: bool, custom: Option<u32>) -> u32`
  - `fn session_tab_title_color(palette: &ThemePalette, active: bool, disconnected: bool) -> u32`

- [x] **Step 1: 写失败测试**

```rust
#[test]
fn tab_accent_prioritises_disconnect_over_unread_and_active() {
    let palette = crate::theme::theme_palette("github-dark");
    assert_eq!(session_tab_accent(&palette, true, true, true, None), palette.danger);
    assert_eq!(session_tab_accent(&palette, false, false, true, None), palette.warning);
    assert_eq!(session_tab_accent(&palette, true, false, false, None), palette.primary);
    assert_eq!(session_tab_accent(&palette, false, false, false, None), palette.text_dimmed);
    assert_eq!(session_tab_accent(&palette, true, true, true, Some(0x336699)), 0x336699);
}

#[test]
fn tab_title_uses_one_text_scale_for_every_state() {
    let palette = crate::theme::theme_palette("github-dark");
    assert_eq!(session_tab_title_color(&palette, true, false), palette.text);
    assert_eq!(session_tab_title_color(&palette, false, false), palette.text_muted);
    assert_eq!(session_tab_title_color(&palette, false, true), palette.text_dimmed);
}
```

- [x] **Step 2: 运行确认失败**

Run: `cargo test -p nyaterm-desktop --lib tab_accent_prioritises`
Expected: 编译失败，两个辅助函数尚未定义。

- [x] **Step 3: 实现并替换内联表达式**

```rust
fn session_tab_accent(
    palette: &crate::theme::ThemePalette,
    active: bool,
    disconnected: bool,
    unread: bool,
    custom: Option<u32>,
) -> u32 {
    if let Some(custom) = custom {
        return custom;
    }
    if disconnected {
        palette.danger
    } else if active {
        palette.primary
    } else if unread {
        palette.warning
    } else {
        palette.text_dimmed
    }
}

fn session_tab_title_color(
    palette: &crate::theme::ThemePalette,
    active: bool,
    disconnected: bool,
) -> u32 {
    if disconnected {
        palette.text_dimmed
    } else if active {
        palette.text
    } else {
        palette.text_muted
    }
}
```

`accent_color` 与标题 `.text_color(...)` 改用这两个函数。未读圆点的颜色由 `palette.success` 改为 `palette.warning`，仍只在 `has_unread && !is_active` 时出现；同步角标颜色、暂停透明度和断开页签的 `opacity(0.78)` 不变。

- [ ] **Step 4: 运行通过**

Run: `cargo test -p nyaterm-desktop --lib tab_accent && cargo test -p nyaterm-desktop --lib tab_title`
Expected: PASS；desktop 测试无法本机执行时记录并由 CI 补跑。

- [x] **Step 5: 提交**

```bash
cd F:/workspace/nyaterm
cargo fmt -p nyaterm-desktop && cargo check -p nyaterm-desktop
git add crates/nyaterm-desktop/src/features/layout/workspace/surface/tabs.rs
git commit -m "refactor(tabs): 收敛断开未读与标题状态颜色" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"
```

---

### Task 6: 锁定现有页签快捷键，不新增第二套键处理

**Files:**
- Test: `crates/nyaterm-desktop/src/shortcuts/mod.rs` 内现有测试模块

**Interfaces:**
- Consumes: 现有 `SHORTCUT_REGISTRY`、`ShortcutId`、`ShortcutPlatform`、`select_relative_session`、`close_active_session`。
- Produces: 无生产代码变化；现有 action 路径继续唯一负责切换与关闭。

**现状结论：** `NextTab` / `PreviousTab` 已分别走 `select_relative_session(1/-1)`，`CloseTab` 已走 `close_active_session`；默认值也已存在。本任务只补平台默认值覆盖，禁止在 `tabs.rs` 添加 `Ctrl+Tab` / `Ctrl+W` 原始键处理。

- [x] **Step 1: 添加平台默认值表征测试**

在现有 tests 模块中复用已有导入，并补入 `ShortcutPlatform`：

```rust
#[test]
fn tab_shortcuts_keep_platform_defaults() {
    let cases = [
        (ShortcutId::NextTab, "ctrl+tab", "meta+tab"),
        (ShortcutId::PreviousTab, "ctrl+shift+tab", "meta+shift+tab"),
        (ShortcutId::CloseTab, "ctrl+shift+w", "meta+w,meta+shift+w"),
    ];
    for (id, windows, macos) in cases {
        let definition = SHORTCUT_REGISTRY
            .iter()
            .find(|definition| definition.id == id)
            .expect("tab shortcut stays registered");
        assert_eq!(
            definition
                .default_binding_for(ShortcutPlatform::Windows)
                .canonical(),
            windows
        );
        assert_eq!(
            definition
                .default_binding_for(ShortcutPlatform::MacOs)
                .canonical(),
            macos
        );
    }
}
```

- [ ] **Step 2: 运行表征测试**

Run: `cargo test -p nyaterm-desktop --lib tab_shortcuts_keep_platform_defaults`
Expected: PASS。该测试锁定已有行为，不虚构 RED；若失败，说明现有默认值与已确认约定不一致，先报告差异，不在本任务顺手改业务逻辑。

- [x] **Step 3: 提交**

```bash
cd F:/workspace/nyaterm
cargo fmt -p nyaterm-desktop && cargo check -p nyaterm-desktop
git add crates/nyaterm-desktop/src/shortcuts/mod.rs
git commit -m "test(shortcuts): 锁定页签快捷键的平台默认值" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"
```

---

### Task 7: 精确锁定关闭按钮的焦点环与可见性

**Files:**
- Modify: `crates/nyaterm-desktop/src/features/layout/workspace/surface/tabs.rs`（`SessionTabCloseButton` 的 debug selector）
- Test: 同文件现有测试模块，复用 `tab_close_fixture` / `draw_fixture` / `press_key`

**Interfaces:**
- Consumes: 现有 `SessionTabCloseButton` 焦点句柄与 `TabCloseFixture`。
- Produces: 仅 focused 时存在 `test-session-tab-close-focus-ring` 对应的 `{id}-focus-ring` debug selector；运行行为不变。

- [x] **Step 1: 写失败测试**

```rust
#[gpui::test]
fn close_button_reveals_focus_ring_and_activates_from_keyboard(cx: &mut TestAppContext) {
    let _cleanup = crate::test_support::TestAppCleanup::new(cx);
    let (cx, activations, opacity) = tab_close_fixture(cx);
    draw_fixture(cx);
    assert_eq!(opacity.get(), 0.);
    assert!(cx.debug_bounds("test-session-tab-close-focus-ring").is_none());

    cx.update(|window, cx| window.focus_next(cx));
    draw_fixture(cx);
    assert_eq!(opacity.get(), 1., "focus must reveal the close button");
    assert!(
        cx.debug_bounds("test-session-tab-close-focus-ring").is_some(),
        "the focused close button must expose its painted ring bounds"
    );

    press_key(cx, "space");
    assert_eq!(activations.get(), 1);
}
```

- [ ] **Step 2: 运行确认失败**

Run: `cargo test -p nyaterm-desktop --lib close_button_reveals_focus_ring`
Expected: FAIL；当前没有 `{id}-focus-ring` selector。

- [x] **Step 3: 只在 focused 时登记焦点环边界**

在 `SessionTabCloseButton::render` 开头、`self.id` 被移动前创建选择器：

```rust
let ring_selector = format!("{}-focus-ring", self.id);
```

在关闭按钮根 div 的 `.border_color(...)` 后追加：

```rust
.when(focused, |this| {
    let selector = ring_selector.clone();
    this.debug_selector(move || selector.clone())
})
```

保留现有 `track_focus`、`tab_stop(true)`、透明/不透明规则和 locked 行为，不改按钮尺寸或点击处理。

- [ ] **Step 4: 运行通过**

Run: `cargo test -p nyaterm-desktop --lib tab_close && cargo test -p nyaterm-desktop --lib close_button_reveals`
Expected: PASS；desktop 测试无法本机执行时记录并由 CI 补跑。

- [x] **Step 5: 提交**

```bash
cd F:/workspace/nyaterm
cargo fmt -p nyaterm-desktop && cargo check -p nyaterm-desktop
git add crates/nyaterm-desktop/src/features/layout/workspace/surface/tabs.rs
git commit -m "test(tabs): 精确锁定关闭按钮焦点环与可见性" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"
```

---

### Task 8: 文档与本阶段 release 验收

**Files:**
- Create: `scripts/tests/test_macos_docs_mentions.py`
- Modify: `docs/macos-visual-migration.md`
- Modify: `docs/superpowers/plans/2026-11-06-gpui-macos-exit-dialog-and-tabs.md`（只勾选已验证步骤）

**Interfaces:**
- Consumes: 以上全部任务。
- Produces: 文档小节“确认对话框与终端页签”，记录真实默认键位与已验证范围。

- [x] **Step 1: 写文档验收测试**

```python
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class DocsTest(unittest.TestCase):
    def test_migration_doc_documents_exit_dialog_and_tab_keys(self):
        text = (ROOT / "docs/macos-visual-migration.md").read_text(encoding="utf-8")
        self.assertIn("Escape", text)
        self.assertIn("Ctrl+Tab", text)
        self.assertIn("Ctrl+Shift+W", text)
        self.assertIn("Cmd+W", text)
```

- [ ] **Step 2: 运行确认失败**

Run: `python -m unittest scripts.tests.test_macos_docs_mentions`
Expected: FAIL；当前文档尚无这些退出与页签键位说明。

- [x] **Step 3: 写文档**

在 `docs/macos-visual-migration.md` 增加“确认对话框与终端页签”小节，明确写入：

- 确认对话框默认聚焦取消；无输入框时 Enter 触发取消，Escape 始终取消；带输入框的表单聚焦第一个输入控件。
- 危险确认保留原确认与后果文案，按钮使用主题 Primary 强调色而非红色实心块。
- 选中页签使用轻填充、透明边框、无阴影；未读使用 warning，断开优先于 active/unread，自定义色保持原 alpha。
- Windows/Linux：`Ctrl+Tab` / `Ctrl+Shift+Tab` 切换，`Ctrl+Shift+W` 关闭；macOS：`Cmd+Tab` / `Cmd+Shift+Tab` 切换，`Cmd+W` 或 `Cmd+Shift+W` 关闭。
- 关闭按钮在 hover、选中或键盘聚焦时可见；锁定页签仍不能通过关闭按钮或快捷键关闭。

- [x] **Step 4: 运行文档测试与本地验证**

Run: `python -m unittest scripts.tests.test_macos_docs_mentions`
Expected: PASS。

Run: `cargo test -p nyaterm-ui --lib`
Expected: PASS，并记录实际通过数量。

Run: `cargo fmt --all -- --check && cargo check --workspace --locked`
Expected: exit 0。desktop 测试若仍为本机 `os error 5`，只记录编译与未执行状态，等待 CI，不宣称完整测试通过。

- [ ] **Step 5: release 打包与真实应用验收**

```bash
cd F:/workspace/nyaterm
python scripts/release/package_native.py x86_64-pc-windows-msvc
```

启动 `target/native-package/NyaTerm-portable/NyaTerm.exe`，逐项确认：

- 有活动会话时请求退出：默认焦点在取消，直接 Enter 会取消；再次打开后焦点仍在取消。
- 鼠标或 Tab 移到确认按钮后按 Escape 只取消；再次请求退出仍能打开确认框。
- 危险确认按钮使用主题强调色，后果文案不变。
- 页签选中态为轻填充且无阴影；自定义色、断开、未读、同步、拖拽和锁定图标仍正确。
- `Ctrl+Tab` / `Ctrl+Shift+Tab` 循环切换，`Ctrl+Shift+W` 关闭当前未锁定页签；关闭按钮在 hover/选中/聚焦时可见。

只记录实际执行的 Windows release 验收；不得据此声称已在 macOS 真实 UI 验证 `Cmd` 键。

- [x] **Step 6: 提交**

```bash
cd F:/workspace/nyaterm
git add docs/macos-visual-migration.md scripts/tests/test_macos_docs_mentions.py docs/superpowers/plans/2026-11-06-gpui-macos-exit-dialog-and-tabs.md
git commit -m "docs: 记录退出确认与页签交互约定" -m "Co-Authored-By: Claude Code <noreply@anthropic.com>"
```
