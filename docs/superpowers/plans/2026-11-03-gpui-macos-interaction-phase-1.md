# GPUI macOS Interaction Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 建立可复用的 macOS segmented 与 grouped-list 交互，并完成常规、交互、外观、快捷键四个设置页的第一阶段迁移。

**Architecture:** `NyaSegmentedControl` 封装 gpui-kit 已有 `ToggleGroup.segmented()`，不另写焦点和键盘系统。设置页只替换展示与输入控件，事件继续写回现有 `SettingsFeatureState` 和 draft；背景图预览复用现有 wallpaper cache。

**Tech Stack:** Rust 2024、GPUI、gpui-kit、rust-i18n、Cargo、GPUI visual tests。

**Spec:** `docs/superpowers/specs/2026-11-03-gpui-macos-interaction-design.md`

## Global Constraints

- 保留 draft、保存、取消、校验、凭据与持久化语义。
- desktop 不直接依赖 gpui-kit；只通过 `nyaterm-ui` 使用组件。
- 长列表继续使用 `NyaSelect`，短互斥选项才使用 segmented。
- 不改变现有圆角、无边框窗口和八方向 resize。
- 每个任务先写失败测试，再写最小实现；本机测试被执行策略阻止时记录并由 CI 补跑。

## Review Focus

- 空选项、越界选中值和未知配置值不得 panic 或误改其他设置。
- disabled 控件以及纯键盘路径不得产生意外选择事件。
- 搜索与“仅自定义”组合为空时，快捷键页显示明确空状态而不是空白。
- 背景图路径失效、解码失败或旧 wallpaper 结果不得显示错误预览。
- hover-only 的关闭/重置操作在键盘焦点路径上必须可发现。

---

### Task 1: NyaSegmentedControl

**Files:**
- Create: `crates/nyaterm-ui/src/segmented.rs`
- Modify: `crates/nyaterm-ui/src/lib.rs`
- Test: `crates/nyaterm-ui/src/segmented.rs` 内 `#[cfg(test)]`

**Interfaces:**
- Consumes: `gpui_kit::component::button::{Toggle, ToggleGroup, ToggleVariant}`，以及 `Disableable` 和 `ToggleVariants`。
- Produces: `NyaSegmentedControl::new(id).items(...).selected_index(...).disabled(...).on_select(...)`，回调签名为 `Fn(&usize, &mut Window, &mut App)`。

- [ ] **Step 1: 写点击、禁用和键盘失败测试**

使用 `TestAppContext` 创建三项控件，断言点击第二项只回调索引 `1`；disabled 点击无回调；聚焦后 `right` 到下一项、`home` 到第一项、`end` 到最后一项。增加空选项和 `selected_index(Some(9))` 不 panic 的渲染测试。

```rust
let clicks = Rc::new(Cell::new(Vec::<usize>::new()));
let control = NyaSegmentedControl::new("log-level")
    .items(["Warn", "Info", "Debug"])
    .selected_index(Some(0))
    .on_select(|index, _, _| clicks.borrow_mut().push(*index));
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p nyaterm-ui segmented --lib`  
Expected: 因 `nyaterm_ui::segmented` 尚不存在而失败。

- [ ] **Step 3: 实现单选 segmented 封装**

字段与 `NyaRadioGroup` 保持一致：`id/items/selected_index/disabled/on_select`。渲染时把每个 item 转成 `Toggle::new(index).label(item).checked(selected_index == Some(index))`，再组成 `ToggleGroup::new(id).segmented().with_variant(ToggleVariant::Outline).disabled(disabled)`。

`ToggleGroup` 是多选组件，因此回调必须只把新变为 `true` 且与旧选中不同的索引发给 `on_select`；空列表直接渲染空容器，越界索引不视为任何选项被选中。禁止在 `on_select` 内再次构造新控件或修改持久化值。

```rust
let previous = self.selected_index;
let on_select = self.on_select;
let group = ToggleGroup::new(self.id)
    .segmented()
    .with_variant(ToggleVariant::Outline)
    .disabled(self.disabled)
    .children(self.items.into_iter().enumerate().map(|(index, label)| {
        Toggle::new(index)
            .label(label)
            .checked(previous == Some(index))
    }))
    .on_click(move |checks, window, cx| {
        let next = checks.iter().position(|checked| *checked);
        if next != previous
            && let (Some(index), Some(handler)) = (next, &on_select
        {
            handler(&index, window, cx);
        }
    });
```

- [ ] **Step 4: 导出并运行测试**

在 `lib.rs` 添加 `mod segmented;` 和 `pub use segmented::NyaSegmentedControl;`。

Run: `cargo test -p nyaterm-ui segmented --lib`  
Expected: 点击、disabled、键盘、空列表和越界测试全部通过。

- [ ] **Step 5: 格式检查并提交**

Run: `cargo fmt -p nyaterm-ui -- --check && cargo check -p nyaterm-ui`

```bash
git add crates/nyaterm-ui/src/segmented.rs crates/nyaterm-ui/src/lib.rs
git commit -m "feat(ui): 增加 macOS segmented 单选控件"
```

### Task 2: 菜单勾选、嵌套列表与焦点可见性

**Files:**
- Modify: `crates/nyaterm-desktop/src/features/layout/activity_bar.rs`
- Modify: `crates/nyaterm-desktop/src/features/pages/settings/workspace/general.rs`
- Modify: `crates/nyaterm-desktop/src/features/layout/workspace/surface/tabs.rs`
- Modify: `crates/nyaterm-desktop/src/features/layout/sidebar/sessions.rs`
- Test: 对应文件内现有测试模块

**Interfaces:**
- Consumes: 当前 `ActivityBarZone`、`entry_zone`、标签/会话 hover 状态。
- Produces: 菜单当前区域带 `icons/check.svg`；设置嵌套行无竖线；键盘聚焦的行操作保持可见。

- [ ] **Step 1: 添加菜单状态与可见性失败测试**

测试当前 zone 产生 check icon，其他 zone 不产生；startup restore 嵌套行不再包含 `border_l_1`；标签和会话操作在 focused 时最终 opacity 为 `1.0` 而非 `0.0`。

- [ ] **Step 2: 运行定向测试确认失败**

Run: `cargo test -p nyaterm-desktop activity_bar_menu --lib`  
Run: `cargo test -p nyaterm-desktop tab_close --lib`  
Expected: 当前实现缺少 check icon 和 focused 可见分支。

- [ ] **Step 3: 最小实现**

Activity Bar 保留选中项不可重复移动，但在 label 前添加 check icon。General 删除 `.border_l_1().border_color(...)`，保留 `.pl_3().ml_1()`。标签/会话 opacity 条件改为 `hovered || selected || focused`，不得让必要关闭操作只在鼠标 hover 时出现。

```rust
.when(selected, |this| {
    this.child(svg().size(px(14.)).path("icons/check.svg"))
})
.opacity(if hovered || selected || focused { 1.0 } else { 0.0 })
```

- [ ] **Step 4: 验证并提交**

Run: `cargo test -p nyaterm-desktop activity_bar_menu --lib`  
Run: `cargo test -p nyaterm-desktop tab_close --lib`  
Run: `cargo check -p nyaterm-desktop`

```bash
git add crates/nyaterm-desktop/src/features/layout/activity_bar.rs crates/nyaterm-desktop/src/features/pages/settings/workspace/general.rs crates/nyaterm-desktop/src/features/layout/workspace/surface/tabs.rs crates/nyaterm-desktop/src/features/layout/sidebar/sessions.rs
git commit -m "feat(ui): 统一菜单勾选与焦点可见性"
```

### Task 3: 常规页 segmented 与 grouped rows

**Files:**
- Modify: `crates/nyaterm-desktop/src/features/pages/settings/mod.rs`
- Modify: `crates/nyaterm-desktop/src/features/pages/settings/workspace/general.rs`
- Test: `crates/nyaterm-desktop/src/features/pages/settings/mod.rs` 内测试模块

**Interfaces:**
- Consumes: `NyaSegmentedControl`。
- Produces: `settings_segmented_control(id, labels, selected_index, disabled, on_select)`，不持有 SelectRegistry。

- [ ] **Step 1: 写选项索引和状态回写失败测试**

为 `["warn", "info", "debug"]` 断言当前 `info` 的索引为 `1`，未知值返回 `None`。调用现有 `set_diagnostics_level("debug")` 和 `set_diagnostics_retention_days(14)`，断言只改变对应 draft 字段。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p nyaterm-desktop settings_segmented --lib`  
Expected: helper 不存在，测试失败。

- [ ] **Step 3: 添加共享 helper 并迁移常规页**

实现：

```rust
fn settings_segmented_control(
    id: impl Into<String>,
    labels: impl IntoIterator<Item = impl Into<SharedString>>,
    selected_index: Option<usize>,
    disabled: bool,
    on_select: impl Fn(&usize, &mut Window, &mut App) + 'static,
) -> NyaSegmentedControl
```

日志级别使用 Warn/Info/Debug 三项，保留天数使用 3/7/14/30 四项。`on_select` 通过现有 `with_app` 调用 `set_diagnostics_level` / `set_diagnostics_retention_days`，随后沿用现有刷新和保存路径。语言、Header Status 长选项继续使用下拉。

- [ ] **Step 4: 验证并提交**

Run: `cargo test -p nyaterm-desktop settings_segmented --lib`  
Run: `cargo test -p nyaterm-desktop diagnostics_ --lib`  
Run: `cargo check -p nyaterm-desktop`

```bash
git add crates/nyaterm-desktop/src/features/pages/settings/mod.rs crates/nyaterm-desktop/src/features/pages/settings/workspace/general.rs
git commit -m "feat(settings): 常规页短选项改为 segmented"
```

### Task 4: 交互页短行为与数值控件

**Files:**
- Modify: `crates/nyaterm-desktop/src/features/pages/settings/workspace/interaction.rs`
- Test: 同文件测试模块

**Interfaces:**
- Consumes: Task 3 的 `settings_segmented_control`、现有 `set_terminal_right_click_action` 和 `NyaNumberInput`。

- [ ] **Step 1: 写右键行为映射失败测试**

验证 `none/menu/paste` 对应索引 0/1/2，选择 paste 只调用现有右键行为 setter，旧 `interaction_right_click_paste` 兼容映射不被破坏。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p nyaterm-desktop interaction_right_click --lib`  
Expected: 当前下拉实现没有 segmented 映射。

- [ ] **Step 3: 迁移右键行为并保留数值 stepper**

将 terminal right-click 三项改为 segmented。命令建议最小/最大字符继续使用现有 `NyaNumberInput` 的步进按钮、边界 clamp 和 draft setter，不另造数值组件。Tab 双击/中键/右键的其他行为逐项迁移，未知持久化值继续由现有 `NyaSelect` 显示，不能静默改成默认值。

```rust
settings_segmented_control(
    "settings.interaction.terminal-right-click-action",
    [t!("settings.terminalRightClickNone"), t!("settings.terminalRightClickMenu"), t!("settings.terminalRightClickPaste")],
    selected_index,
    false,
    cx.listener(|this, index, _, cx| {
        let action = ["none", "menu", "paste"][*index];
        this.with_app(cx, |app, cx| app.settings.set_terminal_right_click_action(action));
    }),
)
```

- [ ] **Step 4: 验证并提交**

Run: `cargo test -p nyaterm-desktop interaction_right_click --lib`  
Run: `cargo test -p nyaterm-desktop command_suggestion --lib`  
Run: `cargo check -p nyaterm-desktop`

```bash
git add crates/nyaterm-desktop/src/features/pages/settings/workspace/interaction.rs
git commit -m "feat(settings): 交互页短行为改为 segmented"
```

### Task 5: 外观页背景图预览与安全拖放

**Files:**
- Modify: `crates/nyaterm-desktop/src/features/pages/settings/workspace/appearance.rs`
- Modify: `crates/nyaterm-desktop/src/features/shell/appearance.rs`
- Test: `crates/nyaterm-desktop/src/features/pages/settings/workspace/appearance.rs` 内测试模块

**Interfaces:**
- Consumes: `app.shell.wallpaper_asset()`、`WallpaperAsset::image()`、`gpui::ExternalPaths`、现有 `select_background_image` 和 `queue_wallpaper_refresh`。
- Produces: 当前 wallpaper 的只读缩略预览和单文件图片 drop 入口。

- [ ] **Step 1: 写路径和 drop 失败测试**

覆盖：无路径无预览；当前 cache 与 draft 路径一致才显示预览；旧路径 cache 不显示；多文件 drop 不修改 draft；目录或非 png/jpg/jpeg/webp 扩展不修改 draft；单个有效图片调用现有选择路径和刷新逻辑。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p nyaterm-desktop appearance_wallpaper_drop --lib`  
Expected: drop 过滤和 preview 一致性 helper 不存在。

- [ ] **Step 3: 实现预览和 drop**

通过 `with_app` 只读克隆当前 `WallpaperAsset` 图像，并使用 `img(image).object_fit(ObjectFit::Cover)` 渲染固定高度预览；路径与 cache 不一致时显示现有 path row，不显示旧图。drop 容器使用 `can_drop(|drag, _, _| drag.is::<ExternalPaths>())`，只接受一个文件和明确图片扩展，成功后调用 `select_background_image` 与 `queue_wallpaper_refresh`。解码仍由原 wallpaper loader 完成，失败不显示旧图，并保留现有错误提示，不覆盖其他外观 draft 字段。

```rust
let current_path = self.settings.summary().background_image_path.clone();
let wallpaper = self.with_app(cx, |app, _| {
    current_path
        .as_deref()
        .filter(|path| app.shell.wallpaper_is_requested(path))
        .and_then(|_| app.shell.wallpaper_asset())
        .map(|asset| asset.image().clone())
});
let can_accept = paths.len() == 1
    && paths[0].is_file()
    && matches!(paths[0].extension().and_then(|ext| ext.to_str()),
        Some("png" | "jpg" | "jpeg" | "webp"));
```

- [ ] **Step 4: 验证 slider 未回退并提交**

确认现有 `appearance_opacity_slider` 的键盘 left/right/home/end 与数值标签原样保留。

Run: `cargo test -p nyaterm-desktop appearance_wallpaper_drop --lib`  
Run: `cargo test -p nyaterm-desktop appearance_opacity --lib`  
Run: `cargo check -p nyaterm-desktop`

```bash
git add crates/nyaterm-desktop/src/features/pages/settings/workspace/appearance.rs crates/nyaterm-desktop/src/features/shell/appearance.rs
git commit -m "feat(settings): 背景图增加预览与图片拖放"
```

### Task 6: 快捷键“仅自定义”筛选与空状态

**Files:**
- Modify: `crates/nyaterm-desktop/src/features/settings/state.rs`
- Modify: `crates/nyaterm-desktop/src/features/pages/settings/workspace/keybindings.rs`
- Test: `crates/nyaterm-desktop/src/features/settings/state.rs` 和 `keybindings.rs` 内测试模块

**Interfaces:**
- Consumes: 现有 `keybindings` override map、搜索和原位录制。
- Produces: `KeybindingPresentationState.custom_only: bool` 与 `toggle_keybinding_custom_only()`。

- [ ] **Step 1: 写筛选状态失败测试**

验证默认 `custom_only == false`，toggle 后只保留 override；搜索与 custom-only 同时为空结果时显示 `settings.keybindingsNoMatches`；录制、冲突和重置路径不被筛选状态清除。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p nyaterm-desktop keybinding_custom_only --lib`  
Expected: `custom_only` 字段不存在。

- [ ] **Step 3: 实现过滤和空状态**

在 `KeybindingSettingsState` 与 presentation state 增加 `custom_only`，由 `keybinding_presentation()` 复制。`shortcut_category_group` 在现有搜索过滤后增加 `!custom_only || overrides.contains(shortcut.id)`。当所有分类为空时显示本地化空状态；切换 filter 只通知刷新，不调用 begin/cancel recording。

```rust
.filter(|shortcut| {
    !custom_only || self.settings.summary().keybindings.contains_key(shortcut.id.as_str())
})
```

```rust
pub(in crate::features) fn toggle_keybinding_custom_only(&mut self) {
    self.keybindings.custom_only = !self.keybindings.custom_only;
}
```

- [ ] **Step 4: 验证并提交**

Run: `cargo test -p nyaterm-desktop keybinding_custom_only --lib`  
Run: `cargo test -p nyaterm-desktop keybinding_conflict --lib`  
Run: `cargo check -p nyaterm-desktop`

```bash
git add crates/nyaterm-desktop/src/features/settings/state.rs crates/nyaterm-desktop/src/features/pages/settings/workspace/keybindings.rs
git commit -m "feat(settings): 快捷键支持仅自定义筛选"
```

### Task 7: 阶段验收与文档

**Files:**
- Modify: `docs/macos-visual-migration.md`
- Modify: `docs/superpowers/plans/2026-11-03-gpui-macos-interaction-phase-1.md`

- [ ] **Step 1: 运行本地验证**

Run: `cargo fmt --all -- --check`  
Run: `cargo check --workspace --locked`  
Run: `cargo test -p nyaterm-ui segmented --lib`  
Run: `cargo test -p nyaterm-desktop settings_segmented --lib`  
Run: `cargo test -p nyaterm-desktop keybinding_custom_only --lib`  
Run: `cargo build --release --target x86_64-pc-windows-msvc -p nyaterm-app --bin nyaterm`

若 desktop 测试因 `os error 5` 未执行，记录准确命令和原因，不修改测试来绕过。

- [ ] **Step 2: 真实应用验收**

启动便携版，逐页检查常规、交互、外观、快捷键：点击、左右/Home/End、disabled、搜索与筛选空状态、背景图无效路径、亮暗主题、窄窗口、滚动恢复、独立设置窗口圆角和右下缩放。

- [ ] **Step 3: 更新状态并提交**

只把实际通过的页面标为完成；未执行的测试和未验收项保留明确限制。

```bash
git add docs/macos-visual-migration.md docs/superpowers/plans/2026-11-03-gpui-macos-interaction-phase-1.md
git commit -m "docs: 记录 macOS 交互第一阶段验收"
```

## 后续独立计划

- 阶段 2：其余设置页。
- 阶段 3：简洁 source list 与单 Inspector。
- 阶段 4：Transfers、Connections、Security、History、Plugins。
- 阶段 5：AI、Resource、Processes、Docker 和剩余页面。

每个后续子系统在开始前依据同一 spec 和当前实现编写独立实施计划，避免跨子系统共享未定义接口。
