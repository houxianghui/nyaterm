# GPUI 线 macOS 视觉重塑 — 设计文档

日期：2026-10-28
状态：待评审
前置文档：`2026-09-10-macos-visual-overhaul-design.md`（React 线，已落地）
分支：`gpui`（跟踪上游 v2.0.0 纯 Rust/GPUI 重写版）

## 背景

上游 nyakang/nyaterm v2.0.0 将应用整体重写为纯 Rust/GPUI，React 前端被删除。
fork 采用双线策略：`main` 留在 Tauri/React 线（跟随 `upstream/archive/tauri-v1`），
`gpui` 分支跟踪 `upstream/main`。本设计在 `gpui` 分支上重建 fork 的 macOS 视觉
identity，目标观感与 React 线 fork 现状一致。

## 目标

- GPUI 版默认外观 = fork 的 macOS 中性灰调（深浅两套），与 React 线色值逐 token 对齐
- Safari 悬浮胶囊标签、蓝色菜单高亮、Finder 式内缩选中、更大更连贯的圆角
- 全平台统一 mac 风（不只 macOS）；原位替换默认主题，不加开关、不新增主题条目

## 非目标（YAGNI）

- 不引入全局设计 token 层（GPUI 无 CSS 变量机制，token 化收益不确定）
- 不改主题系统架构、不动 `ThemeCatalog`/`resolve_theme` 逻辑
- 不做 GPUI 阴影/动效体系的深度改造（GPUI 支持有限，逐案简化处理）
- 不追求与 React 线像素级一致；以「观感同为 macOS 原生」为准

## 关键架构事实（已核实）

- GPUI 主题数据在 `crates/nyaterm-core/src/themes/builtins.json`，**字段结构与
  React 线 `src/lib/themes.ts` 的 ThemeColors 完全同构**（含 camelCase 命名与
  嵌套 terminal ANSI 16 色），色值可逐字段拷贝
- 默认主题 `DEFAULT_THEME_ID = "github-dark"`（`nyaterm-core/src/themes/mod.rs:8`），
  与 fork 原位替换策略兼容
- `ThemeConfig::validate()` 只校验格式（hex、label ≤14 字符），不断言具体色值；
  现有测试仅硬编码断言 windterm 两个主题，改 github-dark/light 不破测试
- 主标签条 `crates/nyaterm-desktop/src/features/layout/workspace/surface/tabs.rs`
  当前是 VS Code 风格（右侧分隔线 + 2px 主色指示条 + 方角），是 Safari 胶囊化
  的主战场
- `nyaterm-ui` 提供共享 widget（`tabs.rs` 已有 Pill 变体、`menu.rs` 有 selected 态），
  对应 fork「改共享原语而非逐面板」原则
- 验证链路：本机无 Rust 工具链 → push gpui → `gh workflow run main-snapshot.yml
  --repo houxianghui/nyaterm --ref gpui` → 代理下载 windows-x64 包并校验字节数 →
  用户目测

## 总体方案：四阶段

| 阶段 | 内容 | 改动面 | 验证 |
|------|------|--------|------|
| 1 | macOS 深浅色板原位替换（含终端 ANSI） | `builtins.json` + `theme.rs` 硬编码 palette/label/测试 + 6 个 locale | CI 测试 + 快照目测配色 |
| 2 | 共享原语 mac 化（菜单蓝底白字高亮、Finder 内缩选中、圆角约定） | `nyaterm-ui` 的 `menu.rs`/`tabs.rs` 等 | CI 快照目测浮层/菜单 |
| 3 | 主标签条 Safari 胶囊化（去分隔线、去 2px 指示条、浮胶囊） | `workspace/surface/tabs.rs` | CI 快照目测标签栏 |
| 4 | 逐屏核对微调 | 视前序结果 | CI 快照 |

每阶段独立 commit（可单独 revert），通过目测验收后才进入下一阶段。

## Stage 1 设计（本次实施）

### 改动

**开工前勘查发现：GPUI 的主题颜色是双数据源**——`theme_palette(id)` 对非
windterm 内置主题返回 `nyaterm-ui/src/theme.rs` 中 `builtin_palette()` 的
**硬编码 Rust 值**，`builtins.json` 只提供扩展 token（shadow、scroll_thumb、
find 色等）与 Theme Designer 数据。因此 Stage 1 改动四个位置：

1. `crates/nyaterm-core/src/themes/builtins.json` — `github-dark`/`github-light`
   两个条目：`name` 改 "macOS Dark"/"macOS Light"，`label` 改 "macOS"/"macOS Light"，
   swatch 与 colors 全套替换（色值表见下，数据源 fork main `src/lib/themes.ts`，
   字段名完全同构逐字段拷贝；`shadow`/`findMatchBackground` 的 rgba 字符串格式
   上游已支持）。
2. `crates/nyaterm-ui/src/theme.rs` — `builtin_palette()` 中 github-dark/light
   两个硬编码 `ThemePalette` 块替换为同一套 macOS 值（u32 hex）。其中
   `surface_elevated` 取 hover 同色（与 `palette_from_theme` 的
   `surface_elevated = bg_hover` 行为对齐，使硬编码与 JSON 推导结果全等）。
   `appearance_theme_label()`："github-dark" → "macOS"、"github-light" →
   "macOS Light"。
3. `crates/nyaterm-ui/src/theme.rs` 测试 — `semantic_colors_match_tauri_theme_tokens`
   的 github-dark/light 断言元组更新为 macOS 值；新增钉版测试断言
   `theme_palette(id) == palette_from_theme(resolve_theme(id))` 对两个 github id
   全等（把双数据源的一致性钉死，防上游合并回退）。
4. `crates/nyaterm-desktop/locales/*.json`（6 个语言）— 主题删除确认文案中的
   "GitHub Dark" 改 "macOS Dark"。

另：上游 `rust.yml` 只在 push 到 `main`/`migration/gpui` 时跑测试，需把 fork 的
`gpui` 分支加入触发列表，否则上述测试在 fork CI 永不执行。

**`github-dark` → macOS Dark**：`name: "macOS Dark"`，`label: "macOS"`，
`swatch: "#1d1d1f"`，colors 替换为：

```json
{
  "bg": "#131316", "bgPanel": "#313138", "bgTerminal": "#1d1d1f",
  "bgHover": "#34343a", "bgInput": "#232326", "bgSectionHeader": "#262629",
  "border": "#3a3a40", "text": "#f5f5f7", "textMuted": "#98989d",
  "textDimmed": "#6e6e73", "primary": "#0a84ff", "primaryHover": "#0071e3",
  "onPrimary": "#ffffff", "focusRing": "#0a84ff", "danger": "#ff453a",
  "dangerHover": "#ff6961", "success": "#30d158", "warning": "#ffd60a",
  "link": "#0a84ff", "shadow": "rgb(0 0 0 / 0.30)", "scrollThumb": "#3a3a40",
  "accent": "#bf5af2",
  "terminal": {
    "background": "#1d1d1f", "foreground": "#f5f5f7", "foregroundIntense": "#ffffff",
    "cursor": "#f5f5f7", "selectionBackground": "#323236", "lineHighlight": "#2a2a2e",
    "findMatchBackground": "rgba(10, 132, 255, 0.3)", "findMatchBorder": "#0a84ff",
    "black": "#48484c", "red": "#ff453a", "green": "#30d158", "yellow": "#ffd60a",
    "blue": "#0a84ff", "magenta": "#bf5af2", "cyan": "#64d2ff", "white": "#a5a5aa",
    "brightBlack": "#6e6e73", "brightRed": "#ff6961", "brightGreen": "#33d375",
    "brightYellow": "#ffd60a", "brightBlue": "#40a9ff", "brightMagenta": "#da8fff",
    "brightCyan": "#64d2ff", "brightWhite": "#f5f5f7"
  }
}
```

**`github-light` → macOS Light**：`name: "macOS Light"`，`label: "macOS Light"`，
`swatch: "#ffffff"`，colors 替换为：

```json
{
  "bg": "#ebebf0", "bgPanel": "#ffffff", "bgTerminal": "#f5f5f7",
  "bgHover": "#e8e8ed", "bgInput": "#ffffff", "bgSectionHeader": "#f2f2f7",
  "border": "#d2d2d7", "text": "#1d1d1f", "textMuted": "#6e6e73",
  "textDimmed": "#98989d", "primary": "#0a84ff", "primaryHover": "#0071e3",
  "onPrimary": "#ffffff", "focusRing": "#0a84ff", "danger": "#ff3b30",
  "dangerHover": "#e0312b", "success": "#34c759", "warning": "#ff9f0a",
  "link": "#0a84ff", "shadow": "rgb(0 0 0 / 0.12)", "scrollThumb": "#d2d2d7",
  "accent": "#af52de",
  "terminal": {
    "background": "#f5f5f7", "foreground": "#1d1d1f", "foregroundIntense": "#000000",
    "cursor": "#1d1d1f", "selectionBackground": "#b8d7ff", "lineHighlight": "#f2f2f7",
    "findMatchBackground": "rgba(10, 132, 255, 0.25)", "findMatchBorder": "#0a84ff",
    "black": "#6e6e73", "red": "#d70015", "green": "#2ea44f", "yellow": "#9a6700",
    "blue": "#0a7ffb", "magenta": "#a40ec4", "cyan": "#0a7e8c", "white": "#1d1d1f",
    "brightBlack": "#8e8e93", "brightRed": "#e03131", "brightGreen": "#34c759",
    "brightYellow": "#b8860b", "brightBlue": "#1a90ff", "brightMagenta": "#c539d1",
    "brightCyan": "#12b5c3", "brightWhite": "#000000"
  }
}
```

（数据源：fork main `src/lib/themes.ts` 的 `githubDark`/`githubLight`，逐字段拷贝。）

### 验证

1. push 到 `gpui` 分支后触发上游 `main-snapshot.yml`（workflow_dispatch，`--ref gpui`）
2. 用 `.claude/skills/merge-upstream/scripts/download-snapshot.sh` 下载并校验字节数
   （注意：该脚本当前硬编码 fork 的 release 资产名，gpui 线的资产命名需现场确认，
   必要时临时适配或手工 curl + 尺寸校验）
3. 用户目测：窗口底色、面板、终端 ANSI 配色、深浅两套切换

### 回滚

单 commit revert 即复原。

### 风险

- 低。改动集中在主题数据与其硬编码镜像，无逻辑变化；双数据源一致性由新增
  钉版测试锁定。
- 本机无 Rust 工具链：`theme.rs` 的编译与测试只能在 CI（`rust.yml`，已加入
  gpui 触发）首次运行，存在一次返工的可能。
- gpui 线快照产物的下载流程细节（资产命名/release 形态与 Tauri 线不同）需
  首次构建时现场确认。
- 已知视觉缺口（接受）：GPUI 阴影/动效与 CSS 体系不同，Stage 1 只保证颜色观感。

## Stage 2-4 原则（细节在各阶段开工前补充）

- **Stage 2**：菜单项选中 = 主色蓝实底 + 白字（映射 `--df-primary`/`--df-on-primary`
  等价物，不用全局 accent）；列表选中 = Finder 式内缩圆角实色块；圆角在
  `nyaterm-ui` 层约定统一常量（对齐 React 线 sm 6px / md 8px / lg 10px / xl 14px）。
- **Stage 3**：标签改为悬浮胶囊（圆角、与窗口底色分离、激活态面板色填充），
  删除右分隔线与 2px 主色指示条。
- **Stage 4**：对照 React 线 fork 逐屏核对，微调。
- 全程遵循 fork 视觉原则：改共享原语而非逐面板；工具栏按钮悬停保持灰色
  （不变蓝）是对的。
