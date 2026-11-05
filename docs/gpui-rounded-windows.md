# GPUI 圆角与独立窗口边框修复指南

本文记录 2026-11-03 独立设置窗口圆角、白色外框修复的实际结论。以后修改 GPUI 圆角、子窗口外框、DWM 窗口样式或相关透明背景时，先阅读本文，再改代码。整体视觉进度和其他坑点见 [GPUI macOS 风格化迁移记录](macos-visual-migration.md)。

对应视觉提交：`e4d2f1769`（`feat(theme): 对齐桌面面板与独立设置窗口视觉`）。后续实现如有变化，以当前源码和新的真实窗口验收为准，并同步更新本文。

## 1. 先分清窗口和渲染层

独立设置窗口与主窗口 `workspace_surface` 中的设置分支不是同一路径。修改主窗口设置页不能解决独立 `SettingsWindow` 的原生外框问题。

独立设置窗口至少有四层，不能只给最外层一个 `rounded()` 就认为完成：

1. GPUI 原生窗口背景与 `nya_root` 背景。
2. `SettingsWindow` 的根内容。
3. 全尺寸 `SettingsPanel` 背景。
4. 底部操作栏和 Windows 原生非客户区。

关键文件：

- `crates/nyaterm-desktop/src/features/settings/settings_runtime/window.rs`
- `crates/nyaterm-desktop/src/features/settings/settings_runtime/native_frame.rs`
- `crates/nyaterm-desktop/src/features/pages/settings/panel.rs`
- `crates/nyaterm-ui/src/root.rs`
- `crates/nyaterm-ui/src/sizing.rs`

全尺寸 `SettingsPanel` 曾把已经完成的下圆角重新涂成直角，因此原生窗口分支必须同时具有 `rounded(radius_xl())` 和 `overflow_hidden()`。底部操作栏保留下左、下右圆角，不能只修顶部。

## 2. 当前 Windows 有效组合

当前实现由 `native_frame.rs` 集中处理，不要在各个面板里重复添加 DWM 调用。

### GPUI 内容层

- `WindowBackgroundAppearance::Transparent`
- `set_nya_root_transparent(&mut root, true)`
- 根内容、全尺寸面板及 footer 使用各自所需的圆角和裁剪

### Windows 原生层

- 使用 `DWMWCP_ROUND`。
- `DWMWA_BORDER_COLOR = DWMWA_COLOR_NONE`。
- `DWMWA_CAPTION_COLOR = DWMWA_COLOR_NONE`。
- 同时移除 `WS_CAPTION | WS_THICKFRAME`，再通过 `SWP_FRAMECHANGED` 应用样式。
- 保留扩展样式和其他窗口 style 位，不做整值覆盖。
- 三项 DWM 调用及样式配置全部成功后才缓存 `native_frame_configured`；失败允许后续 render 重试。

实测中，单独设置 `DWMWCP_ROUND`，或只移除 caption/thickframe 中的一个，仍会留下顶部白线。不要把某一项单独抽走而预期视觉效果不变。

## 3. 无边框后必须保留缩放路径

`WS_THICKFRAME` 同时承担普通窗口的边缘缩放能力。直接删除它虽然能消除白框，但会使原来的边缘拖动失效，因此 `native_frame.rs` 还包含设置窗口局部 subclass：

1. `WM_NCHITTEST` 在窗口外缘返回八方向 `HTLEFT` 至 `HTBOTTOMRIGHT`。
2. `WM_NCLBUTTONDOWN` 将这些 hit code 按 Windows 的连续编号映射到对应 `SC_SIZE + WMSZ_*`，并用 `PostMessageW` 延迟投递；必须让当前按下消息返回后再进入系统 modal resize loop，不能在 subclass 内同步 `SendMessageW` 重入。
3. 其他消息继续调用 `DefSubclassProc`，交回 GPUI。
4. `WM_NCDESTROY` 移除 subclass。
5. 最大化状态不接管自定义 resize；按钮、客户区、右键和双击不得被转换成缩放命令。

命中宽度按窗口 DPI 计算，屏幕坐标按有符号 16 位高低字解析，支持负坐标显示器。纯函数测试覆盖八方向、内部、外侧和 `SC_SIZE` 映射。

**已验证的缩放修复：** 初版在 subclass 内同步 `SendMessageW(WM_SYSCOMMAND)`，直接投递 `WM_NCLBUTTONDOWN` 时似乎可用，但真实鼠标按下路径不能启动拖动。改为 `PostMessageW` 后，使用相同真实鼠标输入测得：右边缘从 816×568 扩到 876×568，右下角从 816×568 扩到 876×608；两次均恢复原尺寸，随后截图复核上下外圈白线仍为 0。后续修改此文件仍必须重做真实输入验收；`cargo check`、hit-code 单元测试和视觉截图不能替代这项验收。

## 4. 已排除或产生副作用的方案

以下方案不要再作为首选试错：

| 方案 | 实际结果 |
| --- | --- |
| 把 DWM border 改成主题背景色 | 顶部白点被覆盖，但整圈原生直角框变得可见，不能使用 |
| 只把 caption 改成 `DWMWA_COLOR_NONE` | 顶部白线仍在 |
| `DWMWCP_DONOTROUND` + 透明 border | 顶部圆角外缘仍有白色合成线 |
| 只删除扩展样式、caption 或 thickframe | 在当时的圆角配置下均未单独消除白线 |
| 禁用 `DWMWA_NCRENDERING_POLICY` | 截图无变化，不是有效修复 |
| `SetWindowRgn` | 硬裁剪、锯齿和左右不一致，已弃用 |
| 只给 `SettingsWindow` 根节点加圆角 | 下角会被全尺寸 `SettingsPanel` 背景重新覆盖 |
| 用 `PrintWindow` 判断透明窗口最终外观 | 与真实屏幕合成结果可能不同，不能作为验收依据 |

不要把“DWM 属性读回成功”当作视觉效果已经正确，也不要用单个像素或一次截图代替完整视觉验收。

## 5. 普通 UI 圆角规则

fork 的圆角尺度为 **6 / 8 / 10 / 14 px**，不是 GPUI 内建的 4 / 6 / 8 / 12 px。优先使用 `nyaterm-ui/src/sizing.rs` 的：

- `radius_sm()`
- `radius_md()`
- `radius_lg()`
- `radius_xl()`

gpui-kit 组件从 `Theme::radius` 派生圆角，主题桥接负责把组件基准映射到 fork 尺度。修改共享组件时优先调整共享原语，不要逐面板复制数值。

## 6. 后续修改检查清单

1. 确认目标是主窗口、内嵌设置页，还是独立子窗口。
2. 沿根背景、全尺寸面板、footer、原生非客户区逐层检查覆盖关系。
3. 小范围修改，一次只验证一个变量；不要同时改圆角、透明度、DWM 样式和布局。
4. 保留已经验收的层，不通过整段回退来“碰运气”。
5. 使用 release 实际程序，在亮色和暗色主题下查看真实屏幕截图，重点检查四角和整个外圈。
6. 修改 `native_frame.rs` 时，同时验证标题拖动、最小化、最大化、关闭及八方向 resize。
7. 本地完成 `fmt`、`cargo check`、release 构建和实际窗口验收后再提交；本机 desktop 测试若被执行策略阻止，应明确记录并由 CI 补跑，不能声称测试通过。
8. 若本文与新的实测结果不一致，先查明层级和时序，再更新实现与本文。
