# macOS 视觉重塑（全局默认外观）实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 nyaterm 默认外观替换为 macOS 中性灰调（深浅两套 + 终端调色盘 mac 化 + 更柔和的圆角/阴影/动效）。

**Architecture:** 复用现有主题与 CSS 变量体系：`src/lib/themes.ts` 里的颜色 token 通过 `applyThemeToDOM()`（`src/context/ThemeContext.tsx:39-63`）注入为 `--df-*`；圆角由 `src/index.css` 里 `:root { --radius }` + `@theme inline` 的 calc 标尺驱动；阴影由 Tailwind v4 的 `--shadow-*` 主题变量驱动。做法是**原地改值**（保留 `github-dark`/`github-light` 的 `id` 与 swatch 不动，只改 `name`/`label`/`colors`），避免新增冗余主题条目，也保证现有 `resolveTheme("github-dark")` 引用与持久化设置不破坏。

**Tech Stack:** React + Tailwind CSS v4 + shadcn/ui (new-york) + tw-animate-css；测试 Vitest + Testing Library。

**Spec:** `docs/superpowers/specs/2026-09-10-macos-visual-overhaul-design.md`

## Global Constraints

- 深色默认主题 `id` 保持 `github-dark`，浅色保持 `github-light`（`DEFAULT_THEME_ID = "github-dark"`，`themeList` 顺序 `githubDark` 第一、`githubLight` 第十一，均不变）。
- UI 正文近白 `#f5f5f7` / 窗口灰 `#1d1d1f`；主色 `#0a84ff`、悬停 `#0071e3`。
- 圆角基准 `--radius: 0.625rem`（10px）。
- 所有新增/修改的 `ThemeColors` 字段必须为字符串（含带 alpha 的 `rgba()` 或 `#hex`）。
- 不新增任何运行时开关或主题条目；不引入新字体资源。

---

### Task 1: macOS 深浅主题 + 终端调色板

**Files:**
- Modify: `src/lib/themes.ts`（`githubDark` 对象，约第 68-123 行；`githubLight` 对象，约第 754-825 行）
- Test: `src/lib/themes.test.ts`（新建）

**Interfaces:**
- Consumes: `ThemeColors` / `TerminalColors` 接口（`themes.ts:4-56`，字段名不变）。
- Produces: `themes[DEFAULT_THEME_ID]`（`github-dark`）与 `themes["github-light"]` 的 `colors` 字段，被 `ThemeContext`、`main.tsx`、`backgroundImage.test.ts` 等消费。字段名与类型不变，仅换值。

- [ ] **Step 1: 写失败测试**

新建 `src/lib/themes.test.ts`：

```ts
import { describe, expect, it } from "vitest";
import { DEFAULT_THEME_ID, themes } from "./themes";

describe("macOS default theme", () => {
  it("keeps the default theme id stable", () => {
    expect(DEFAULT_THEME_ID).toBe("github-dark");
    expect(themes["github-dark"]).toBeDefined();
  });

  it("uses the macOS neutral gray palette", () => {
    const c = themes["github-dark"].colors;
    expect(c.bg).toBe("#1d1d1f");
    expect(c.bgPanel).toBe("#2a2a2e");
    expect(c.text).toBe("#f5f5f7");
    expect(c.primary).toBe("#0a84ff");
    expect(c.primaryHover).toBe("#0071e3");
  });

  it("ships a mac-ified light theme", () => {
    const c = themes["github-light"].colors;
    expect(c.bg).toBe("#f5f5f7");
    expect(c.bgPanel).toBe("#ffffff");
    expect(c.text).toBe("#1d1d1f");
  });

  it("defines full 16-color terminal palettes", () => {
    for (const id of ["github-dark", "github-light"]) {
      const t = themes[id].colors.terminal;
      for (const key of [
        "background", "foreground", "cursor", "selectionBackground",
        "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
        "brightBlack", "brightRed", "brightGreen", "brightYellow",
        "brightBlue", "brightMagenta", "brightCyan", "brightWhite",
      ]) {
        expect(t[key], `${id}.${key}`).toBeTypeOf("string");
      }
    }
  });
});
```

- [ ] **Step 2: 运行确认失败**

Run: `pnpm test src/lib/themes.test.ts`
Expected: FAIL（颜色断言与当前 GitHub 色值不符，例如 `c.bg` 为 `#0d1117` 而非 `#1d1d1f`）。

- [ ] **Step 3: 原地替换 `githubDark` 的 UI token 与终端调色板**

把 `themes.ts` 中 `githubDark` 的 `name`/`label` 改为 `"macOS Dark"` / `"macOS"`（`swatch` 保持 `#0d1117`），并把 `colors` 改为：

```ts
colors: {
  bg: "#1d1d1f",
  bgPanel: "#2a2a2e",
  bgTerminal: "#1d1d1f",
  bgHover: "#34343a",
  bgInput: "#232326",
  bgSectionHeader: "#262629",
  border: "#3a3a40",
  text: "#f5f5f7",
  textMuted: "#98989d",
  textDimmed: "#6e6e73",
  primary: "#0a84ff",
  primaryHover: "#0071e3",
  onPrimary: "#ffffff",
  focusRing: "#0a84ff",
  danger: "#ff453a",
  dangerHover: "#ff6961",
  success: "#30d158",
  warning: "#ffd60a",
  link: "#0a84ff",
  shadow: "rgb(0 0 0 / 0.30)",
  scrollThumb: "#3a3a40",
  accent: "#bf5af2",
  terminal: {
    background: "#1d1d1f",
    foreground: "#f5f5f7",
    cursor: "#f5f5f7",
    selectionBackground: "#323236",
    lineHighlight: "#2a2a2e",
    findMatchBackground: "rgba(10, 132, 255, 0.3)",
    findMatchBorder: "#0a84ff",
    black: "#48484c",
    red: "#ff453a",
    green: "#30d158",
    yellow: "#ffd60a",
    blue: "#0a84ff",
    magenta: "#bf5af2",
    cyan: "#64d2ff",
    white: "#a5a5aa",
    brightBlack: "#6e6e73",
    brightRed: "#ff6961",
    brightGreen: "#33d375",
    brightYellow: "#ffd60a",
    brightBlue: "#40a9ff",
    brightMagenta: "#da8fff",
    brightCyan: "#64d2ff",
    brightWhite: "#f5f5f7",
  },
},
```

- [ ] **Step 4: 原地替换 `githubLight` 的 UI token 与终端调色板**

把 `githubLight` 的 `name`/`label` 改为 `"macOS Light"` / `"macOS Light"`，`colors` 改为：

```ts
colors: {
  bg: "#f5f5f7",
  bgPanel: "#ffffff",
  bgTerminal: "#ffffff",
  bgHover: "#e8e8ed",
  bgInput: "#ffffff",
  bgSectionHeader: "#f2f2f7",
  border: "#d2d2d7",
  text: "#1d1d1f",
  textMuted: "#6e6e73",
  textDimmed: "#98989d",
  primary: "#0a84ff",
  primaryHover: "#0071e3",
  onPrimary: "#ffffff",
  focusRing: "#0a84ff",
  danger: "#ff3b30",
  dangerHover: "#e0312b",
  success: "#34c759",
  warning: "#ff9f0a",
  link: "#0a84ff",
  shadow: "rgb(0 0 0 / 0.12)",
  scrollThumb: "#d2d2d7",
  accent: "#af52de",
  terminal: {
    background: "#ffffff",
    foreground: "#1d1d1f",
    cursor: "#1d1d1f",
    selectionBackground: "#b8d7ff",
    lineHighlight: "#f2f2f7",
    findMatchBackground: "rgba(10, 132, 255, 0.25)",
    findMatchBorder: "#0a84ff",
    black: "#6e6e73",
    red: "#d70015",
    green: "#2ea44f",
    yellow: "#9a6700",
    blue: "#0a7ffb",
    magenta: "#a40ec4",
    cyan: "#0a7e8c",
    white: "#1d1d1f",
    brightBlack: "#8e8e93",
    brightRed: "#e03131",
    brightGreen: "#34c759",
    brightYellow: "#b8860b",
    brightBlue: "#1a90ff",
    brightMagenta: "#c539d1",
    brightCyan: "#12b5c3",
    brightWhite: "#000000",
  },
},
```

- [ ] **Step 5: 运行测试确认通过**

Run: `pnpm test src/lib/themes.test.ts`
Expected: PASS（4 个用例全绿）。

另跑现有引用 `github-dark` 的测试做回归：`pnpm test src/lib/backgroundImage.test.ts`
Expected: PASS。

- [ ] **Step 6: 提交**

```bash
git add src/lib/themes.ts src/lib/themes.test.ts
git commit -m "feat(ui): mac-neutral dark/light default themes + terminal palette"
```

---

## Task 2: 圆角 + 柔和阴影 token

**Files:**
- Modify: `src/index.css`（`:root` 第 79-81 行；`@theme` 块第 22-35 行）

**Interfaces:**
- Consumes: Tailwind v4 `@theme` 变量命名空间（`--radius`、`--shadow-*`）。
- Produces: 全局 `--radius`（被 `@theme inline` 第 37-44 行的 `--radius-sm/md/lg/xl/2xl/3xl/4xl` 消费）、`--shadow-xs/sm/md/lg/xl`（被 `shadow-xs/md/lg` 等工具类消费）。

- [ ] **Step 1: 调大圆角基准 + 补 SF Pro 字体候选**

把 `index.css` 中 `:root` 块里的：

```css
  --radius: 0.5rem;
```

改为：

```css
  --radius: 0.625rem;
```

再把 `--font-sans`（`@theme` 内第 23-25 行）里 `-apple-system` 之后插入 `"SF Pro Text", "SF Pro Display", `，得到：

```css
  --font-sans:
    system-ui, -apple-system, "SF Pro Text", "SF Pro Display", BlinkMacSystemFont,
    "Segoe UI", "PingFang SC", "Microsoft YaHei", "Noto Sans SC", "Noto Sans CJK SC",
    "Helvetica Neue", Arial, sans-serif;
```

（对 `--font-display` 做同样插入；`--font-mono` 不动。）

- [ ] **Step 2: 加入 macOS 柔和阴影主题变量**

在 `index.css` 的 `@theme` 块（第 22-35 行，含 `--font-*` 与 `--color-primary` 的那个）内，末尾新增以下静态变量（放在 `--color-primary-hover` 之后）：

```css
  --shadow-xs: 0 1px 2px rgb(0 0 0 / 0.05);
  --shadow-sm: 0 1px 2px rgb(0 0 0 / 0.06), 0 1px 3px rgb(0 0 0 / 0.08);
  --shadow-md: 0 4px 8px rgb(0 0 0 / 0.10), 0 1px 3px rgb(0 0 0 / 0.06);
  --shadow-lg: 0 8px 24px rgb(0 0 0 / 0.16), 0 2px 8px rgb(0 0 0 / 0.08);
  --shadow-xl: 0 16px 40px rgb(0 0 0 / 0.22), 0 4px 12px rgb(0 0 0 / 0.10);
```

- [ ] **Step 3: 构建 + 视觉验证**

Run: `pnpm build`（`tsc` + Vite 构建，确认 CSS 变量语法合法、无编译错误）。

视觉验证（`pnpm dev` + 浏览器）：确认卡片/弹窗/按钮/下拉的圆角变大到 10px 基准、modal 阴影变柔和扩散、无明显断层。（若本会话可在浏览器运行 `vite`，用浏览器验证；否则标注为手动验证项。）

- [ ] **Step 4: 提交**

```bash
git add src/index.css
git commit -m "feat: mac-radius scale + soft layered shadows"
```

---

## Task 3: UI 原语动效微调

**Files:**
- Modify: `src/components/ui/dialog.tsx`（第 77 行 className）
- Modify: `src/components/ui/popover.tsx`（第 27 行 className）
- Modify: `src/components/ui/dropdown-menu.tsx`（第 34、202 行 className）
- Modify: `src/components/ui/context-menu.tsx`（第 68、96 行 className）

**Interfaces:**
- Consumes: tw-animate-css 的 `duration-*` 工具类。
- Produces: 各浮层的进入/离开动效时长，观感对齐 macOS 的「轻弹出」节奏。

- [ ] **Step 1: 缩短弹窗/浮层动效时长**

对上述 4 个文件中 `data-[state=open]:animate-in` 所在的浮动内容 className，把 `duration-200` 改为 `duration-150`（弹窗 `dialog.tsx` 的第 77 行内容矩形 `duration-200` 也改为 `duration-150`，使淡入淡出更利落）。

改法：在每处 className 里定位 ` duration-200` 并替换为 ` duration-150`（保持 `zoom-in-95`/`zoom-out-95` 不变——这已是 95%→100% 的克制缩放，符合 macOS 手感）。

- [ ] **Step 2: 构建验证**

Run: `pnpm build`
Expected: 通过（无类型/编译错误）。

- [ ] **Step 3: 视觉验证**

`pnpm dev`：打开/关闭设置弹窗、右键菜单、主题下拉，确认动效更轻快、无跳动。（同上，可运行则浏览器验证，否则手动验证项。）

- [ ] **Step 4: 提交**

```bash
git add src/components/ui/dialog.tsx src/components/ui/popover.tsx src/components/ui/dropdown-menu.tsx src/components/ui/context-menu.tsx
git commit -m "feat(ui): tighten float animation duration to 150ms"
```

---

## 验收清单

- [ ] `pnpm test` 全绿（含新增 `themes.test.ts` 与回归的 `backgroundImage.test.ts`）。
- [ ] `pnpm build` 通过。
- [ ] 深色默认主题为 macOS 中性灰；浅色主题为 macOS 浅灰。
- [ ] 卡片/弹窗/输入框/按钮圆角放大到 10px 基准，阴影柔和扩散。
- [ ] 弹窗/菜单动效 150ms 利落。