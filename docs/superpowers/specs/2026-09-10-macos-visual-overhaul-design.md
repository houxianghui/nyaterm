# macOS 视觉重塑（全局默认外观）— 设计文档

日期：2026-09-10
状态：待评审

## 目标

把 nyaterm 的默认外观替换为「macOS 中性灰调」，让界面整体更接近 Apple 桌面软件的观感：柔和的中性灰表面、更大更连贯的圆角、柔和扩散的阴影、更克制的缩放动效，以及 SF Pro 优先的排版。

**范围（已与用户确认）：**
- 交付形态：**直接替换默认外观**（无开关，不保留旧风格可选）
- 颜色基调：**macOS 中性灰调**（不是仅保留现有深色色板）
- 排版/字体：SF Pro 优先，Windows 回退到 Inter（已在 `--font-sans` 栈中具备）

## 现状梳理（样式体系链路）

- **颜色**：颜色 token 在 `src/lib/themes.ts` 中定义，通过 `applyThemeToDOM()`（`src/context/ThemeContext.tsx:39-63`）在运行时注入为 `--df-*` CSS 变量。默认主题 `DEFAULT_THEME_ID = "github-dark"`（`themes.ts:1067`）。
- **圆角**：`components.json` 是 `new-york` 风格，`--radius: 0.5rem`。`src/index.css:37-44` 的 `@theme inline` 块把 `--radius` 映射为 `--radius-sm/md/lg/xl/2xl/3xl/4xl`。
- **阴影**：各 ui 组件用 `shadow-sm/md/lg`（`src/components/ui/dialog.tsx`、`popover.tsx` 等）实现；`ThemeColors.shadow`（`themes.ts:51`）是阴影颜色 token。
- **动效**：`tw-animate-css` 的 `animate-in/out`、`zoom-in/out`、`fade-in/out`、`duration-200`。
- **字体**：`--font-sans` 已在 `index.css:22-25` 声明（`system-ui, -apple-system, ...`），Inter/JetBrains Mono 由 `@fontsource` 引入。

**`rounded-*` 使用规模**：432 处、122 文件，绝大多数使用语义化标尺（`rounded-md/lg/...`），少量 `rounded-full` 和 `rounded-[..]`。

## 设计目标

### 1. 颜色：macOS 中性灰调默认主题（深色 + 浅色）

把默认主题改为中性灰蓝调，参照 Apple 系统界面（窗口 `#1d1d1f`、侧栏/面板 `#2a2a2e`、悬停 `#3a3a3e` 一类）。调整目标 token：

**深色（默认）**

| Token | 目标值 | 说明 |
|-------|--------|------|
| `bg` | `#1d1d1f` | 窗口底色 |
| `bgPanel` | `#2a2a2e` | 面板/卡片 |
| `bgTerminal` | `#1d1d1f` | 终端底色 |
| `bgHover` | `#34343a` | 悬停 |
| `bgInput` | `#232326` | 输入框 |
| `bgSectionHeader` | `#262629` | 分区头 |
| `border` | `#3a3a40` | 描边 |
| `text` | `#f5f5f7` | 正文（Apple 系近白） |
| `textMuted` | `#98989d` | 次要 |
| `textDimmed` | `#6e6e73` | 更弱 |
| `primary` | `#0a84ff` | macOS 系统蓝 |
| `primaryHover` | `#0071e3` | 蓝色系悬停 |
| `onPrimary` | `#ffffff` | 主色之上文字 |

**浅色（同步）**

| Token | 目标值 | 说明 |
|-------|--------|------|
| `bg` | `#f5f5f7` | 窗口底色 |
| `bgPanel` | `#ffffff` | 面板/卡片 |
| `bgTerminal` | `#ffffff` | 终端底色 |
| `bgHover` | `#e8e8ed` | 悬停 |
| `bgInput` | `#ffffff` | 输入框 |
| `bgSectionHeader` | `#f2f2f7` | 分区头 |
| `border` | `#d2d2d7` | 描边 |
| `text` | `#1d1d1f` | 正文（近黑） |
| `textMuted` | `#6e6e73` | 次要 |
| `textDimmed` | `#98989d` | 更弱 |
| `primary` | `#0a84ff` | macOS 系统蓝 |
| `primaryHover` | `#0071e3` | 蓝色系悬停 |
| `onPrimary` | `#ffffff` | 主色之上文字 |

其余 `danger/success/warning/link/focusRing/scrollThumb/accent` 与上面的新中性灰保持协调（`focusRing` 用 `#0a84ff`、`scrollThumb` 深 `#3a3a40` / 浅 `#d2d2d7`）。`shadow` 颜色改为柔和偏蓝的黑（见阴影一节）。

终端 16 ANSI 调色板同步 mac 化：深/浅两套终端配色都改为 macOS 终端柔和配色（低饱和、可读性优先，避免荧光高饱和色）。

### 2. 圆角：更大、更连贯

把 `--radius` 从 `0.5rem`（8px）调至 `0.625rem`（10px）。`index.css:37-44` 的 calc 映射也随之放大，得到：
- 窗口/大容器 `rounded-xl`：14px
- 卡片/弹窗 `rounded-lg`：10px
- 输入框/按钮 `rounded-md`：8px
- 小元素 `rounded-sm`：6px

`--radius-2xl/3xl/4xl` 保持逐步放大以供图表/特殊控件使用。

### 3. 阴影：柔和扩散

把 `shadow-md/lg/xl` 改为多层柔和的 rgba 阴影（低不透明度、较大 blur、蓝色调），替换现在较硬的 `shadow-lg`。macOS 阴影特点：小偏移、大扩散、低对比。`ThemeColors.shadow` 改为 `rgb(0 0 0 / 0.30)` 系。

### 4. 动效：更克制的缩放

- 弹窗/浮层/下拉：从 `zoom-in-95`（居中放大）改为更柔和的 `zoom-in-105` 或带 `duration-200` → `duration-150`，以及更自然 ease。
- 承接 macOS 的「轻弹出」节奏，不改变现有结构，仅调动画参数（`tw-animate-css` + `duration`/`ease` 类）。

### 5. 排版：SF Pro 优先

`--font-sans` 保持 `system-ui, -apple-system, ...` 为首；在 Apple 平台会优先选 SF Pro。加入 `SF Pro Text`/`SF Pro Display` 作为备选（虽非可下载字体，但系统若存在会优先）。字重沿用 `font-weight 650` 风格标题，无需引入新字体资源。

## 涉及文件

### 核心改动
- `src/lib/themes.ts` — 新增「macOS Dark」+「macOS Light」主题（或原地替换 `githubDark` / `githubLight` 的易错默认值，避免产生冗余主题条目）；对齐深浅两套 UI token 与终端调色盘；更新 `DEFAULT_THEME_ID` 与 `themeList` 排序（macOS 主题置顶）。
- `src/index.css` — `@theme inline` 块的 `--radius` 标尺，`.active-tab` 改用圆角（个头小的物料）。按需加少量 macOS 风格阴影工具类。
- `src/components/ui/*.tsx` — `dialog.tsx`/`popover.tsx`/`dropdown-menu.tsx`/`context-menu.tsx` 的阴影与动效类微调。
- `src/components/dialog/theme/ThemeDesignerDialog.tsx` — 若色板相关标签需要调整（核对项）。

### 验证范围
逐屏核对：设置/新建会话/快捷命令/文件浏览器/OTP/终端工具栏/录像等关键浮层圆角与阴影无明显突兀。

## 拆分与顺序（供后续实现计划）

1. 先加 macOS 深浅两套色板 token + 终端调色盘（`themes.ts`）→ 默认替换，核验整体配色
2. 再调圆角标尺 + 关键 ui 组件阴影/动效
3. 逐屏核验 + 微调
4. 提交

## 尚未拍板 / 待实现时确认

（已落定，无遗留开放项）

- 浅色主题（`github-light` 等）同步改 mac 浅色中性灰：深色默认 + 浅色都官改为 macOS 中性灰调。
- 终端 16 ANSI 调色板同步 mac 化：深/浅两套终端配色都对齐 macOS 柔和观感。