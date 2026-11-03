# GPUI macOS 风格化迁移记录

本文汇总截至 2026-11-03 的 macOS 风格化工作、有效实现、验证结果和踩坑记录。后续继续视觉对齐时先阅读本文；涉及圆角、透明窗口或 Windows 原生边框时，还必须阅读 [GPUI 圆角与独立窗口边框修复指南](gpui-rounded-windows.md)。

## 1. 目标与边界

目标是在 fork 的 GPUI 分支上对齐 React/Tauri 线的 macOS 视觉语言，同时保留“视觉使用 fork、逻辑跟随上游”的策略：

- 菜单使用 macOS 蓝底白字选中态。
- 标签采用 Safari 式胶囊外观。
- 列表采用 Finder 式内嵌选中块。
- 面板、设置卡片和独立窗口使用一致的圆角、间距与背景。
- 标题栏与内容背景沉浸，去掉不必要的分割线。
- 图标对齐 1.X，不再显示渐变方块占位图标。
- Windows 独立设置窗口不得出现白色直角外框，并保留系统缩放能力。

这些修改不应改变 SSH、终端、存储、同步、认证或其他业务逻辑。

## 2. 已完成并提交的改动

| 阶段 | 提交 | 实际内容 |
| --- | --- | --- |
| 图标与尺寸基础 | `f13636079` | 替换应用图标资源，引入 fork 圆角常量层 |
| 窗口与面板图标 | `6a55a899c` | 左上角、关于页等位置使用真实终端 PNG 图标，移除渐变方块方案 |
| 第一轮视觉杠杆 | `dca16aeb3` | 将关键组件对齐 fork 的 6/8/10/14px 圆角尺度 |
| Safari 标签 | `54420ee27` | 工作区标签改为 Safari 式胶囊布局 |
| Stage 2 | `03c824874` | 菜单选中色、Finder 式列表选中块、胶囊标签测试修正 |
| 菜单白字修复 | `523f6815b` | 带快捷键的菜单项也继承 accent 前景色，真正实现蓝底白字 |
| 组件库圆角基准 | `fa4636c07` | gpui-kit 的 `Theme::radius` 等由 fork 的 `radius_md/lg` 驱动 |
| desktop 全量圆角 | `a065d9649` | 117 个 desktop 文件的内建圆角调用统一为 fork 尺度 |
| 面板与独立设置窗口 | `e4d2f1769` | 面板圆角和间距、标题栏沉浸、分割线清理、设置卡片与独立窗口无边框圆角 |
| 设置窗口缩放 | `c0683cfcd` | 用延迟 `PostMessageW(WM_SYSCOMMAND)` 修复真实鼠标无法拖动的问题 |
| 圆角专项文档 | `e9b904161` | 记录有效 DWM 组合、失败方案、渲染层与验收要求，并在 `AGENTS.md` 建立优先参考入口 |

### 2.1 菜单选中态

菜单背景来自 `tokens.accent`，文字来自 `accent_foreground`，并映射到 primary/on_primary。仅改 token 不够：`PopupMenuItem::element` 内部曾硬编码普通文字和快捷键文字颜色，覆盖了 hover 白字。

最终处理：

- 非 danger 项不再硬编码 `foreground`。
- 快捷键列不再硬编码 `muted_foreground`。
- danger 项继续使用危险色。
- 增加 accent token 跟随 primary 的回归测试。

### 2.2 圆角尺度与共享原语

fork 使用 **6 / 8 / 10 / 14px**，GPUI 内建尺度是 4 / 6 / 8 / 12px。统一使用 `nyaterm-ui/src/sizing.rs` 的 `radius_sm/md/lg/xl()`，组件库圆角基准在 `theme_bridge.rs` 中映射。

优先修改共享原语和布局容器，避免逐面板创造新数值。逐文件替换只用于把已有内建圆角调用迁移到共享函数。

### 2.3 标题栏、面板与分割线

已完成：

- 主标题栏移除自绘独立背景，使顶部与根背景沉浸。
- 删除标题栏、子窗口 header、panel header、设置卡片标题区等不必要的分割线。
- 主 shell、侧栏、检查器、底部面板、浮动面板和移动抽屉补充一致的圆角与间距。
- 设置表单卡片保留圆角背景，不再叠加整圈边框。

### 2.4 独立设置窗口

独立 `SettingsWindow` 与主窗口内嵌设置页不是同一路径。最终 Windows 方案由 `settings_runtime/native_frame.rs` 集中处理：

- GPUI 窗口和 root 使用透明背景。
- 根内容、全尺寸 `SettingsPanel` 和 footer 分层处理圆角与裁剪。
- 使用 `DWMWCP_ROUND`，border/caption 均为 `DWMWA_COLOR_NONE`。
- 同时移除 `WS_CAPTION | WS_THICKFRAME` 并应用 `SWP_FRAMECHANGED`。
- 设置窗口局部 subclass 补八方向 `WM_NCHITTEST`，再通过延迟 `SC_SIZE` 进入系统缩放。

详细原因和禁止重复的方案见 [圆角专项指南](gpui-rounded-windows.md)。

## 3. 当前已实现、等待最终视觉验收的改动

以下修改已在本地完成 `cargo check` 和 release 构建，并已启动便携版，但**尚未提交，也不写成用户已验收**：

- 删除左侧栏顶部齿轮和重复的“设置”整块标题区，回收 64px 空白。
- 删除右侧“常规”“终端”“AI”等页面大标题区，表单内容直接上移。
- 保留原生窗口标题栏、左侧导航、表单卡片标题和底部操作区。
- 修改共享 `NyaSettingsLayout`，独立设置窗口与主窗口内嵌设置页统一生效。

涉及文件：

- `crates/nyaterm-ui/src/settings.rs`
- `crates/nyaterm-desktop/src/features/pages/settings/panel.rs`

滚动容器、按 tab 保存的 `ScrollHandle`、响应式 padding、圆角和缩放代码未改变。

## 4. 已完成的验证

### CI

- `a065d9649`：三平台测试与 Lint 通过。
- `e4d2f1769`：Rust CI 的 Windows/Linux/macOS 测试、Lint、Release helpers、Plugin SDK、Workflow syntax、Rust gate 全部通过；Main Snapshot 全平台打包与发布成功。
- `c0683cfcd`：上述 Rust CI 项全部通过；Main Snapshot 成功。

### 本地实际应用

- release 构建和便携 ZIP 打包成功。
- 设置窗口圆角与无白框状态已经用户视觉确认。
- 缩放修复使用真实鼠标输入验证：
  - 右边缘：816×568 → 876×568。
  - 右下角：816×568 → 876×608。
  - 两次均恢复原尺寸，随后复核上下外圈白线为 0。
- 标题删减版已启动等待视觉验收，不能提前视为完成。

本机曾拒绝执行 desktop 测试二进制，错误为 `拒绝访问 (os error 5)`。这种情况下只能记录“测试已编译但未执行”，由 CI 补跑，不能声称本地测试通过。

## 5. 关键坑点与禁止重复的试错

### 5.1 菜单蓝底但文字仍是灰色

原因是组件内部文字色覆盖 token，不是 accent 没生效。先沿实际渲染组件查颜色来源，不要继续改主题猜测。

### 5.2 批量替换造成整文件 CRLF 假 diff

一次 Python 批量修改把行尾全部转换，产生 9629 行假变化。已还原并改用二进制读写保留原始行尾。以后批量修改后必须先看 `git diff --numstat` 和实际 diff，不能把行尾变化当视觉修改提交。

### 5.3 只给根节点加圆角，下角仍被涂成直角

全尺寸 `SettingsPanel` 背景会覆盖根圆角。根、全尺寸面板、footer 是不同绘制层，必须分别确认背景和 `overflow_hidden()`。

### 5.4 把 DWM border 改为背景色

这只遮住顶部白点，却让整圈原生直角边框变得可见，已明确否决。 caption 单独设为 `DWMWA_COLOR_NONE` 也不能解决白线。

### 5.5 只设置一个 DWM 或窗口样式属性

实测中，单独 `DWMWCP_ROUND`、只移除 caption、只移除 thickframe、只改扩展样式或禁用非客户区渲染，都不能稳定得到最终外观。当前组合必须整体保留，不要拆成单项“优化”。

### 5.6 使用 `SetWindowRgn`

会产生硬裁剪、锯齿和左右不一致，已弃用。不要再用它替代抗锯齿圆角。

### 5.7 用 `PrintWindow` 或单点像素做最终视觉判断

透明窗口的离屏捕获与真实 DWM 屏幕合成可能不同。真实屏幕截图和用户视觉确认优先；像素分析只能辅助定位，不能替代整窗验收。

### 5.8 直接投递消息成功，真实鼠标却失败

缩放初版在 subclass 内同步 `SendMessageW(WM_SYSCOMMAND)`，直接投递 `WM_NCLBUTTONDOWN` 时似乎可用，真实鼠标按下却因同步重入不能拖动。改为 `PostMessageW` 后真实输入通过。以后验证窗口消息修复必须使用真实输入路径，不能只 `PostMessage` 模拟成功。

### 5.9 删除 thickframe 后直接结束任务

`WS_THICKFRAME` 同时提供原生边缘缩放。视觉修好后必须继续验证缩放、标题拖动、最大化和关闭，不能用视觉通过掩盖功能回退。

### 5.10 窗口句柄和窗口身份误判

曾因设置子窗口关闭后句柄失效出现 `window not found`。自动化前应重新枚举真实窗口，不要长期使用缓存 HWND，也不要把其他应用的“设置”窗口当成 NyaTerm。

## 6. 构建、打包与下载注意事项

- 本地验证完成后再提交；常用最小检查为 `cargo fmt --check`、`cargo check`、release 构建和真实应用验收。
- 当前本地产物以便携 ZIP 为主；曾因 NSIS 下载缓慢未制作安装版，不能把便携包描述为 NSIS 安装包。
- 下载前先检查网络和代理。`https://git.rangerhou.chat/` 用于加速 GitHub release 资产，不适用于 Actions artifacts。
- Actions artifact 直连曾长时间低速并超时；使用系统代理时还遇到本机证书吊销检查错误。下载必须设置超时、避免无期限后台挂起，并在完成后检查 ZIP 完整性和实际文件大小。
- 临时截图、探针脚本、下载目录和便携构建目录不应混入视觉源码提交。

## 7. 后续工作顺序

1. 用户确认标题删减版的实际视觉效果。
2. 提交该共享布局修改并运行 CI。
3. 若继续逐屏核对，优先使用真实截图检查所有设置 tab、主窗口各面板、亮暗主题和窄窗口布局。
4. 圆角或原生窗口问题先查专项指南，不再从 DWM 属性逐项盲试。
5. 每次视觉修改同时检查导航、滚动、窗口缩放和底部操作区，避免只修外观。
