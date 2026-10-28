# GPUI 线 macOS 视觉重塑 Stage 1（默认色板移植）实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 `gpui` 分支把默认主题 github-dark/github-light 原位替换为 fork 的 macOS 深浅色板（含终端 ANSI 16 色），并用测试钉死双数据源一致性。

**Architecture:** GPUI 的主题颜色是双数据源：`nyaterm-ui/src/theme.rs` 的 `builtin_palette()` 提供硬编码基础 palette（运行时主用），`nyaterm-core/src/themes/builtins.json` 提供扩展 token 与 Theme Designer 数据。两处必须同步替换为同一套 macOS 值；`surface_elevated` 取 hover 同色以对齐 `palette_from_theme` 行为，使两个数据源推导结果全等，再用钉版测试锁定。

**Tech Stack:** Rust workspace（GPUI）、JSON 主题数据、GitHub Actions（rust.yml / main-snapshot.yml）。

**Spec:** `docs/superpowers/specs/2026-10-28-gpui-macos-visual-overhaul-design.md`

## Global Constraints

- 色值必须逐字段等于 fork main 分支 `src/lib/themes.ts` 的 `githubDark`/`githubLight`（本计划已内联全部值）。
- 主题 id 不变（`github-dark`/`github-light`）；`DEFAULT_THEME_ID` 不变。
- `label` ≤ 14 字符（`ThemeConfig::validate()` 限制）。
- 本机无 Rust 工具链：**禁止声称本地跑过 cargo fmt/clippy/test**；Rust 编译与测试以 `rust.yml` CI 结果为唯一准绳。
- 提交信息遵循 conventional commits，正文用中文。
- locale 文案只替换主题名（GitHub Dark → macOS Dark），不改句式。
- 所有改动在 `gpui` 分支（worktree `F:\workspace\nyaterm-gpui`），不碰 main 检出。

## Review Focus

1. **JSON 结构损坏**（手工编辑数组中的大对象易错位）→ Task 2 Step 3 的 `json.load` + 交叉校验脚本覆盖。
2. **双数据源色值分歧**（theme.rs 硬编码与 builtins.json 不一致，导致 Theme Designer 预览与实际渲染不同）→ Task 4 钉版测试 `theme_palette(id) == palette_from_theme(...)` 全等断言覆盖。
3. **`surface_elevated` 语义错位**（`palette_from_theme` 把它映射为 `bg_hover`，硬编码若给不同值会破坏全等）→ Task 3 明确取 hover 同色，Task 4 断言覆盖。
4. **rgba 字符串格式**（`rgb(0 0 0 / 0.30)` 与 `rgba(10, 132, 255, 0.3)` 两种写法）→ 上游现有条目两种都在用（github-dark 的 shadow / windterm 的 findMatchBackground），`parse_color` 均支持；Task 4 测试经 `resolve_theme`+`palette_from_theme` 路径隐式覆盖。
5. **fork 与上游共用 `main-snapshot` release**：gpui 快照会把该 tag 移到 gpui 提交并删除 Tauri 线产物 → Task 7 注意项（Tauri 产物已留存于 `dist-verify/`；重跑 main 的 Build Main Snapshot 可恢复）。

---

### Task 1: 建立 gpui 工作区

**Files:**
- Create: `F:\workspace\nyaterm-gpui`（git worktree，不入库）

**Interfaces:**
- Consumes: 本地分支 `gpui`（跟踪 `origin/gpui` = 上游 v2.0.0）
- Produces: 干净工作区 `F:\workspace\nyaterm-gpui`，后续 Task 2-6 均在其中操作

- [ ] **Step 1: 创建 worktree**

```bash
cd F:/workspace/nyaterm
git worktree add ../nyaterm-gpui gpui
```

- [ ] **Step 2: 验证分支与关键文件存在**

```bash
git -C ../nyaterm-gpui branch --show-current   # 期望: gpui
ls ../nyaterm-gpui/crates/nyaterm-ui/src/theme.rs ../nyaterm-gpui/crates/nyaterm-core/src/themes/builtins.json
```

Expected: 分支为 `gpui`，两个文件均存在。

### Task 2: builtins.json 原位替换 + 本地交叉校验

**Files:**
- Modify: `F:\workspace\nyaterm-gpui\crates\nyaterm-core\src\themes\builtins.json`（第 2 个对象 github-dark，及 id 为 github-light 的对象）
- Create: `F:\workspace\gpui-stage1-tools\check_themes.py`（一次性校验脚本，不入库）

**Interfaces:**
- Consumes: fork main 的 `F:\workspace\nyaterm\src\lib\themes.ts`（`githubDark`/`githubLight`）
- Produces: builtins.json 中两个条目的 name/label/swatch/colors 为 macOS 值（Task 4 的测试依赖 `resolve_theme` 能读到这些值）

- [ ] **Step 1: 替换 github-dark 条目**

将 `builtins.json` 中 `id` 为 `github-dark` 的整个对象（含嵌套 terminal）替换为：

```json
  {
    "id": "github-dark",
    "name": "macOS Dark",
    "label": "macOS",
    "swatch": "#1d1d1f",
    "colors": {
      "bg": "#131316",
      "bgPanel": "#313138",
      "bgTerminal": "#1d1d1f",
      "bgHover": "#34343a",
      "bgInput": "#232326",
      "bgSectionHeader": "#262629",
      "border": "#3a3a40",
      "text": "#f5f5f7",
      "textMuted": "#98989d",
      "textDimmed": "#6e6e73",
      "primary": "#0a84ff",
      "primaryHover": "#0071e3",
      "onPrimary": "#ffffff",
      "focusRing": "#0a84ff",
      "danger": "#ff453a",
      "dangerHover": "#ff6961",
      "success": "#30d158",
      "warning": "#ffd60a",
      "link": "#0a84ff",
      "shadow": "rgb(0 0 0 / 0.30)",
      "scrollThumb": "#3a3a40",
      "accent": "#bf5af2",
      "terminal": {
        "background": "#1d1d1f",
        "foreground": "#f5f5f7",
        "foregroundIntense": "#ffffff",
        "cursor": "#f5f5f7",
        "selectionBackground": "#323236",
        "lineHighlight": "#2a2a2e",
        "findMatchBackground": "rgba(10, 132, 255, 0.3)",
        "findMatchBorder": "#0a84ff",
        "black": "#48484c",
        "red": "#ff453a",
        "green": "#30d158",
        "yellow": "#ffd60a",
        "blue": "#0a84ff",
        "magenta": "#bf5af2",
        "cyan": "#64d2ff",
        "white": "#a5a5aa",
        "brightBlack": "#6e6e73",
        "brightRed": "#ff6961",
        "brightGreen": "#33d375",
        "brightYellow": "#ffd60a",
        "brightBlue": "#40a9ff",
        "brightMagenta": "#da8fff",
        "brightCyan": "#64d2ff",
        "brightWhite": "#f5f5f7"
      }
    }
  },
```

- [ ] **Step 2: 替换 github-light 条目**

将 `id` 为 `github-light` 的整个对象替换为：

```json
  {
    "id": "github-light",
    "name": "macOS Light",
    "label": "macOS Light",
    "swatch": "#ffffff",
    "colors": {
      "bg": "#ebebf0",
      "bgPanel": "#ffffff",
      "bgTerminal": "#f5f5f7",
      "bgHover": "#e8e8ed",
      "bgInput": "#ffffff",
      "bgSectionHeader": "#f2f2f7",
      "border": "#d2d2d7",
      "text": "#1d1d1f",
      "textMuted": "#6e6e73",
      "textDimmed": "#98989d",
      "primary": "#0a84ff",
      "primaryHover": "#0071e3",
      "onPrimary": "#ffffff",
      "focusRing": "#0a84ff",
      "danger": "#ff3b30",
      "dangerHover": "#e0312b",
      "success": "#34c759",
      "warning": "#ff9f0a",
      "link": "#0a84ff",
      "shadow": "rgb(0 0 0 / 0.12)",
      "scrollThumb": "#d2d2d7",
      "accent": "#af52de",
      "terminal": {
        "background": "#f5f5f7",
        "foreground": "#1d1d1f",
        "foregroundIntense": "#000000",
        "cursor": "#1d1d1f",
        "selectionBackground": "#b8d7ff",
        "lineHighlight": "#f2f2f7",
        "findMatchBackground": "rgba(10, 132, 255, 0.25)",
        "findMatchBorder": "#0a84ff",
        "black": "#6e6e73",
        "red": "#d70015",
        "green": "#2ea44f",
        "yellow": "#9a6700",
        "blue": "#0a7ffb",
        "magenta": "#a40ec4",
        "cyan": "#0a7e8c",
        "white": "#1d1d1f",
        "brightBlack": "#8e8e93",
        "brightRed": "#e03131",
        "brightGreen": "#34c759",
        "brightYellow": "#b8860b",
        "brightBlue": "#1a90ff",
        "brightMagenta": "#c539d1",
        "brightCyan": "#12b5c3",
        "brightWhite": "#000000"
      }
    }
  },
```

注意：被替换对象后面的逗号保留；文件其余 17 个主题对象一个字节不动。

- [ ] **Step 3: 写并运行交叉校验脚本**

写入 `F:\workspace\gpui-stage1-tools\check_themes.py`：

```python
"""交叉校验: gpui worktree 的 builtins.json 与 main 的 themes.ts 色值一致。"""
import json
import re
import sys


def ts_block(src, name):
    i = src.index(f"const {name} = {{")
    depth = 0
    for j in range(i, len(src)):
        if src[j] == "{":
            depth += 1
        elif src[j] == "}":
            depth -= 1
            if depth == 0:
                return src[i : j + 1]
    raise ValueError(f"block not found: {name}")


def ts_flat(block):
    # 扁平化 "key": "value" 对; themes.ts 外层与 terminal 内层键名无冲突
    return dict(re.findall(r"(\w+):\s*\"([^\"]*)\"", block))


def json_flat(entry):
    flat = {k: v for k, v in entry["colors"].items() if isinstance(v, str)}
    for k, v in entry["colors"]["terminal"].items():
        flat[k] = v
    return flat


def main():
    main_repo, gpui_tree = sys.argv[1], sys.argv[2]
    ts_src = open(
        main_repo + "/src/lib/themes.ts", encoding="utf-8"
    ).read()
    builtins = json.load(
        open(
            gpui_tree + "/crates/nyaterm-core/src/themes/builtins.json",
            encoding="utf-8",
        )
    )
    failures = 0
    for ts_name, theme_id in [("githubDark", "github-dark"), ("githubLight", "github-light")]:
        expected = ts_flat(ts_block(ts_src, ts_name))
        entry = next(t for t in builtins if t["id"] == theme_id)
        actual = json_flat(entry)
        for meta in ("name", "label", "swatch"):
            if entry[meta] != expected[meta]:
                print(f"MISMATCH {theme_id}.{meta}: {entry[meta]!r} != {expected[meta]!r}")
                failures += 1
        for key, want in expected.items():
            if key in ("id", "name", "label", "swatch"):
                continue
            got = actual.get(key)
            if got != want:
                print(f"MISMATCH {theme_id}.{key}: {got!r} != {want!r}")
                failures += 1
        extra = set(actual) - set(expected)
        if extra:
            print(f"EXTRA keys in {theme_id}: {sorted(extra)}")
            failures += 1
        print(f"{theme_id}: checked {len(expected) - 4} colors")
    if failures:
        print(f"FAILED: {failures} mismatches")
        sys.exit(1)
    print("OK: builtins.json 与 themes.ts 完全一致")


main()
```

Run:

```bash
mkdir -p /f/workspace/gpui-stage1-tools
# (写入上面的脚本后)
python -I /f/workspace/gpui-stage1-tools/check_themes.py /f/workspace/nyaterm /f/workspace/nyaterm-gpui
```

Expected: 输出 `OK: builtins.json 与 themes.ts 完全一致`（若 MISMATCH，修正 JSON 后重跑）。

### Task 3: theme.rs 硬编码 palette 与 label 替换

**Files:**
- Modify: `F:\workspace\nyaterm-gpui\crates\nyaterm-ui\src\theme.rs`（`appearance_theme_label` 约 line 146/160；`builtin_palette` github-dark 块约 line 175-207；github-light 块约 line 616-648；`semantic_colors_match_tauri_theme_tokens` 测试元组）

**Interfaces:**
- Consumes: Task 2 的 builtins.json macOS 值（本 task 的硬编码值必须与其逐一对应）
- Produces: `builtin_palette("github-dark"|"github-light")` 返回 macOS palette；`appearance_theme_label` 返回 "macOS"/"macOS Light"；Task 4 的钉版测试依赖这里的 `surface_elevated == hover`

- [ ] **Step 1: 替换 appearance_theme_label 的两行**

```rust
        "github-dark" => "GitHub Dark",
```
改为
```rust
        "github-dark" => "macOS",
```
以及
```rust
        "github-light" => "GitHub Light",
```
改为
```rust
        "github-light" => "macOS Light",
```

- [ ] **Step 2: 替换 builtin_palette 的 github-dark 块**

`"github-dark" => ThemePalette {` 块中，头部 8 个占位字段（`ui_terminal_bg: 0,` 到 `terminal_find_border: 0,`）**保持不动**，其余字段替换为：

```rust
            bg: 0x131316,
            surface: 0x313138,
            surface_elevated: 0x34343a,
            section_header: 0x262629,
            hover: 0x34343a,
            input: 0x232326,
            border: 0x3a3a40,
            text: 0xf5f5f7,
            text_muted: 0x98989d,
            text_dimmed: 0x6e6e73,
            primary: 0x0a84ff,
            primary_hover: 0x0071e3,
            on_primary: 0xffffff,
            focus_ring: 0x0a84ff,
            link: 0x0a84ff,
            accent: 0xbf5af2,
            success: 0x30d158,
            warning: 0xffd60a,
            danger: 0xff453a,
            terminal_bg: 0x1d1d1f,
            terminal_fg: 0xf5f5f7,
            terminal_cursor: 0xf5f5f7,
            terminal_selection: 0x323236,
            terminal_ansi: [
                0x48484c, 0xff453a, 0x30d158, 0xffd60a, 0x0a84ff, 0xbf5af2, 0x64d2ff, 0xa5a5aa,
                0x6e6e73, 0xff6961, 0x33d375, 0xffd60a, 0x40a9ff, 0xda8fff, 0x64d2ff, 0xf5f5f7,
            ],
```

- [ ] **Step 3: 替换 builtin_palette 的 github-light 块**

同样保留头部 8 个占位字段，其余替换为：

```rust
            bg: 0xebebf0,
            surface: 0xffffff,
            surface_elevated: 0xe8e8ed,
            section_header: 0xf2f2f7,
            hover: 0xe8e8ed,
            input: 0xffffff,
            border: 0xd2d2d7,
            text: 0x1d1d1f,
            text_muted: 0x6e6e73,
            text_dimmed: 0x98989d,
            primary: 0x0a84ff,
            primary_hover: 0x0071e3,
            on_primary: 0xffffff,
            focus_ring: 0x0a84ff,
            link: 0x0a84ff,
            accent: 0xaf52de,
            success: 0x34c759,
            warning: 0xff9f0a,
            danger: 0xff3b30,
            terminal_bg: 0xf5f5f7,
            terminal_fg: 0x1d1d1f,
            terminal_cursor: 0x1d1d1f,
            terminal_selection: 0xb8d7ff,
            terminal_ansi: [
                0x6e6e73, 0xd70015, 0x2ea44f, 0x9a6700, 0x0a7ffb, 0xa40ec4, 0x0a7e8c, 0x1d1d1f,
                0x8e8e93, 0xe03131, 0x34c759, 0xb8860b, 0x1a90ff, 0xc539d1, 0x12b5c3, 0x000000,
            ],
```

注意 `surface_elevated` 必须与 `hover` 同值（对齐 `palette_from_theme` 的 `surface_elevated = bg_hover` 映射，否则 Task 4 的全等断言会失败）。

- [ ] **Step 4: 更新 semantic_colors_match_tauri_theme_tokens 的两个断言元组**

`github-dark` 元组改为：

```rust
            (
                "github-dark",
                0x0a84ff,
                0x0071e3,
                0xffffff,
                0x0a84ff,
                0x0a84ff,
                0xbf5af2,
            ),
```

`github-light` 元组改为：

```rust
            (
                "github-light",
                0x0a84ff,
                0x0071e3,
                0xffffff,
                0x0a84ff,
                0x0a84ff,
                0xaf52de,
            ),
```

（元组字段顺序为 `(id, primary, primary_hover, on_primary, focus_ring, link, accent)`，与测试末尾的解构一致。）

### Task 4: 新增双数据源钉版测试

**Files:**
- Modify: `F:\workspace\nyaterm-gpui\crates\nyaterm-ui\src\theme.rs`（`mod tests` 内，`semantic_colors_match_tauri_theme_tokens` 之后）

**Interfaces:**
- Consumes: Task 2/3 的两个数据源；`super::palette_from_theme`、`nyaterm_core::themes::{resolve_theme, ThemeCatalog}`（文件内已有同路径用法）
- Produces: CI 上对 github-dark/light 的全字段一致性断言

- [ ] **Step 1: 在 `semantic_colors_match_tauri_theme_tokens` 测试结束后追加**

```rust
    #[test]
    fn github_builtins_are_macos_palettes_and_match_json() {
        let catalog = nyaterm_core::themes::ThemeCatalog::default();
        for (id, bg, primary, ansi_black, ansi_bright_white) in [
            ("github-dark", 0x131316, 0x0a84ff, 0x48484c, 0xf5f5f7),
            ("github-light", 0xebebf0, 0x0a84ff, 0x6e6e73, 0x000000),
        ] {
            let theme = nyaterm_core::themes::resolve_theme(id, &catalog);
            assert!(theme.validate().is_empty(), "{id} validate");
            let palette = theme_palette(id);
            // 硬编码 palette 与 JSON 推导结果必须全等(双数据源一致性)
            assert_eq!(palette, super::palette_from_theme(&theme), "{id} json parity");
            assert_eq!(palette.bg, bg, "{id} bg");
            assert_eq!(palette.primary, primary, "{id} primary");
            assert_eq!(palette.terminal_ansi[0], ansi_black, "{id} ansi black");
            assert_eq!(
                palette.terminal_ansi[15], ansi_bright_white,
                "{id} ansi bright white"
            );
        }
    }
```

注：`theme.validate()` 是 `ThemeConfig` 的方法，测试经 `resolve_theme` 拿到的对象上可用（`themes/tests.rs` 第 11 行有同款用法）。

- [ ] **Step 2: 本地能做的唯一检查——括号配平目测**

```bash
grep -c 'fn github_builtins_are_macos_palettes_and_match_json' /f/workspace/nyaterm-gpui/crates/nyaterm-ui/src/theme.rs   # 期望 1
```

本机无 cargo，编译/测试留给 Task 7 的 CI；此处在计划里如实记录。

### Task 5: locales 主题名文案更新

**Files:**
- Modify: `F:\workspace\nyaterm-gpui\crates\nyaterm-desktop\locales\{en,fr,ja,ko,zh-CN,zh-TW}.json`（各 1 处 `deleteConfirmation`）

- [ ] **Step 1: 机械替换**

```bash
cd /f/workspace/nyaterm-gpui
sed -i 's/GitHub Dark/macOS Dark/g' crates/nyaterm-desktop/locales/en.json \
  crates/nyaterm-desktop/locales/fr.json crates/nyaterm-desktop/locales/ja.json \
  crates/nyaterm-desktop/locales/ko.json crates/nyaterm-desktop/locales/zh-CN.json \
  crates/nyaterm-desktop/locales/zh-TW.json
```

- [ ] **Step 2: 验证替换范围恰好 6 处且 JSON 仍合法**

```bash
grep -c 'macOS Dark' crates/nyaterm-desktop/locales/*.json   # 每个文件恰好 1
grep -rn 'GitHub Dark' crates/ --include='*.json' | grep -v builtins.json   # 期望无输出
python -I -c "import json,glob; [json.load(open(f,encoding='utf-8')) for f in glob.glob('crates/nyaterm-desktop/locales/*.json')]; print('locales JSON OK')"
```

Expected: 每文件 1 处；无残留 "GitHub Dark"（builtins.json 除外——它已在 Task 2 改为 "macOS Dark"，grep 命中的应是 macOS 而非 GitHub）；全部 locale JSON 解析通过。

### Task 6: rust.yml 触发分支加入 gpui

**Files:**
- Modify: `F:\workspace\nyaterm-gpui\.github\workflows\rust.yml:5-8`

**Interfaces:**
- Produces: push 到 `gpui` 触发 fmt/clippy/test（Task 7 依赖）

- [ ] **Step 1: 修改触发分支**

```yaml
on:
  push:
    branches: [main, migration/gpui]
  pull_request:
    branches: [main, migration/gpui]
```

改为：

```yaml
on:
  push:
    branches: [main, migration/gpui, gpui]
  pull_request:
    branches: [main, migration/gpui, gpui]
```

（保持原文件的其余 `on:` 字段不动；以上仅示意 branches 行。）

### Task 7: 提交、推送、CI 验证、快照构建与产物校验

**Files:** 无新增；产出为远端状态。

- [ ] **Step 1: 提交色板改动（Task 2-5 一个 commit）**

```bash
cd /f/workspace/nyaterm-gpui
git add crates/nyaterm-core/src/themes/builtins.json crates/nyaterm-ui/src/theme.rs crates/nyaterm-desktop/locales/
git commit -m "feat(theme): 默认主题原位替换为 macOS 色板

github-dark/github-light 的色值、标签与终端 ANSI 调色板替换为 fork 的
macOS 深浅色板(双数据源: builtins.json 与 theme.rs 硬编码同步)。
新增钉版测试锁定 theme_palette 与 palette_from_theme 全等。
locale 删除确认文案中的主题名同步为 macOS Dark。"
```

- [ ] **Step 2: 提交 CI 触发改动（独立 commit）**

```bash
git add .github/workflows/rust.yml
git commit -m "ci: rust.yml 触发分支加入 gpui

fork 的 gpui 实验分支需要 fmt/clippy/test 安全网。"
```

- [ ] **Step 3: 推送 gpui**

```bash
git push origin gpui
```

若 github.com 直连失败，隔 30s 重试至多 3 次；仍失败则**停止并向用户报告**（无推送则无 CI，不得跳过）。

- [ ] **Step 4: 确认 rust.yml 在 gpui 上跑绿**

```bash
gh run list --repo houxianghui/nyaterm --branch gpui --workflow rust.yml --limit 1
gh run watch <run-id> --repo houxianghui/nyaterm --exit-status
```

Expected: success。若失败（编译错/测试红），修复后回到 Step 1 循环；**不得声称通过除非实际看到 success**。

- [ ] **Step 5: 触发快照构建**

```bash
gh workflow run main-snapshot.yml --repo houxianghui/nyaterm --ref gpui
sleep 15
gh run list --repo houxianghui/nyaterm --workflow "Main Snapshot" --limit 1 --json databaseId,headBranch,status
```

确认 `headBranch` 为 `gpui`，记录 run id。

注意（Review Focus #5）：此构建会把 `main-snapshot` release 的 tag 移到 gpui 提交并**删除 Tauri 线快照资产**；Tauri 产物已留存于 `F:\workspace\nyaterm\dist-verify\`，可接受。

- [ ] **Step 6: 监视构建并发现资产名**

```bash
gh run watch <run-id> --repo houxianghui/nyaterm --exit-status
gh release view main-snapshot --repo houxianghui/nyaterm --json assets --jq '.assets[] | "\(.name) \(.size)"'
```

Expected: build success；资产列表中出现 windows-x64 安装包/便携包（GPUI 线命名与 Tauri 线不同，以实际输出为准）。

- [ ] **Step 7: 代理下载 windows-x64 资产并逐字节校验**

对资产名中含 `windows-x64` 的每个文件（若同时有 setup 与 portable 则都下）：

```bash
mkdir -p /f/workspace/nyaterm/dist-verify-gpui
curl -sSL --retry 5 --retry-all-errors -f -o "/f/workspace/nyaterm/dist-verify-gpui/<asset>" \
  "https://git.rangerhou.chat/https://github.com/houxianghui/nyaterm/releases/download/main-snapshot/<asset>"
stat -c '%s' "/f/workspace/nyaterm/dist-verify-gpui/<asset>"
```

Expected: `stat` 字节数与 Step 6 的 `.size` **逐一相等**（代理截断是静默的，不相等必须重下）。**禁止用 `gh release download`**（HTTP/2 PROTOCOL_ERROR 会在 20 MiB 处截断）。

- [ ] **Step 8: 交付用户目测**

告知用户产物位置（`F:\workspace\nyaterm\dist-verify-gpui\`），请其安装/解压后核对：窗口底色、面板灰调、终端 ANSI 配色、深浅主题切换（设置中显示为 "macOS"/"macOS Light"）。目测通过后 Stage 1 完成；Stage 2（共享原语 mac 化）另立计划。
