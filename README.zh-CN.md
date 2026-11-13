<p align="center">
  <img src="./docs-site/static/img/nyaterm.svg" alt="NyaTerm" width="128" height="128">
</p>

<h1 align="center">NyaTerm — macOS 视觉分支</h1>

<p align="center">
  <strong>基于 GPUI 构建的现代原生远程终端工作区。</strong><br/>
  来自 <a href="https://github.com/nyakang/nyaterm">nyakang/nyaterm</a> fork，专注 macOS 风格的整体视觉重塑。<br/>
  <a href="https://nyaterm.app"><strong>nyaterm.app</strong></a> ·
  <a href="https://nyaterm.app/docs/"><strong>在线文档</strong></a>
</p>

<p align="center">
  在一个桌面客户端中处理 SSH、本地 Shell、Telnet、串口、RDP、VNC、SFTP、隧道、OTP、AI 辅助与加密同步。
</p>

<p align="center">
  <a href="https://discord.gg/Tgb9ugMKdM" target="_blank"><img src="https://dcbadge.limes.pink/api/server/Tgb9ugMKdM" alt="Discord" /></a>
  &nbsp;
  <a href="https://docs.qq.com/doc/DYW5TUkZnakxOWUFW" target="_blank"><img src="https://img.shields.io/badge/微信交流群-点击加入-07C160?style=for-the-badge&logo=wechat&logoColor=white" alt="微信交流群" /></a>
  <br/>
  <a href="https://linux.do" target="_blank" rel="noopener noreferrer">
    <img src="https://img.shields.io/badge/LINUX%20DO-论坛-F0B003?style=for-the-badge&logo=discourse&logoColor=white" alt="LINUX DO" />
  </a>
  <br/>
  <a href="https://nyaterm.app"><img alt="Version" src="https://img.shields.io/github/v/release/nyakang/nyaterm.svg?style=flat-square&logo=github&color=0EA5E9&labelColor=334155&label=%E7%89%88%E6%9C%AC&cacheSeconds=300"></a>
  &nbsp;
  <a href="https://github.com/nyakang/nyaterm/releases">
    <img
      alt="GitHub downloads"
      src="https://img.shields.io/github/downloads/nyakang/nyaterm/total.svg?style=flat-square&logo=github&color=0EA5E9&label=%E4%B8%8B%E8%BD%BD%E9%87%8F&labelColor=334155&v=2"
    >
  </a>
  &nbsp;
  <img
    alt="Platform"
    src="https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-0EA5E9.svg?style=flat-square&logo=linux&labelColor=334155&label=%E6%94%AF%E6%8C%81%E5%B9%B3%E5%8F%B0&v=2"
  >
  &nbsp;
  <a href="LICENSE">
    <img
      alt="License"
      src="https://img.shields.io/badge/License-Apache%202.0-0EA5E9.svg?style=flat-square&logo=readthedocs&labelColor=334155&label=%E5%8D%8F%E8%AE%AE&v=2"
    >
  </a>
</p>
<p align="center">
  <a href="./README.md">English</a> · <a href="./README.zh-CN.md">简体中文</a>
</p>

---

## 关于本分支

本分支持续跟踪上游 [`nyakang/nyaterm`](https://github.com/nyakang/nyaterm) 并定期合并其更新。上游的全部功能——SSH、本地终端、Telnet、串口、RDP、VNC、SFTP、隧道、OTP、AI 助手、加密同步——在本分支中与上游完全一致。

本分支改动的是**外观与交互**：一套完整的 macOS 风格默认外观重塑、工作区界面重设计，以及全新的应用图标。完整功能列表请参见[上游 README](https://github.com/nyakang/nyaterm#readme)。

---

## 与原版的差异

### macOS 视觉重塑（默认外观）

- **macOS 中性灰默认主题**（深色 + 浅色），替换原 GitHub 配色默认值，强调色采用 macOS 系统蓝 `#0a84ff`
- **Mac 柔和终端调色板** —— 深浅两套低饱和 16 色 ANSI 终端配色
- **更大、更连贯的圆角** —— 10px 基准标尺（容器 14px、卡片 10px、输入框 8px、小元素 6px）
- **柔和分层阴影** —— 低不透明度、大扩散的柔和阴影，替换原先较硬的高程阴影
- **SF Pro 优先排版** —— 系统字体栈以 SF Pro 领衔，Windows 回退 Inter
- **更克制的动效** —— 150ms 浮动动画，弹窗/浮层/菜单采用更柔和的缩放与缓动；设置中提供界面动画开关

### 界面重设计

- **Safari 风格浮动胶囊标签页**
- **Edge 风格聚合菜单**，替换经典菜单栏
- **亚克力菜单表面** —— 半透明磨砂的菜单与浮层
- **胶囊输入框**
- **Finder 风格选中态**的已保存连接列表
- **macOS 风格活动栏与面板标题栏**，菜单蓝色高亮
- **沉浸式头部栏**，内嵌浮动面板卡片与更紧凑的面板间距
- **悬停才显示的分隔线**；圆角、透明的子窗口
- **Windows 打磨** —— 移除透明窗口的 DWM 非客户区边框，实现干净的无边框观感

### 品牌

- **全新终端应用图标** —— 覆盖 OS 打包图标（Windows/macOS/Linux/Android）与应用内 logo

### 构建 / CI

- 主干快照构建改为手动触发（`workflow_dispatch`），快照 release 标签通过 REST API 更新
- 使用本分支自己的 Tauri 更新签名密钥对
- 移除 Windows 7 构建目标
<img width="1920" height="1032" alt="image" src="https://github.com/user-attachments/assets/726dc223-c06a-4e0c-b55c-9db5b3490b31" />
<img width="800" height="560" alt="image" src="https://github.com/user-attachments/assets/f8da9b06-7c45-4b19-8fb1-6e7be7efa746" />

<img width="1920" height="1032" alt="image" src="https://github.com/user-attachments/assets/9d90ca1b-376b-43ec-b781-2b18d229d7fa" />
<img width="800" height="560" alt="image" src="https://github.com/user-attachments/assets/ad2f73b4-6aea-45b7-8e3e-326eaf3099ff" />

---

<a name="nyaterm-是什么"></a>
# NyaTerm 是什么

**NyaTerm** 是一个面向 SSH 运维和混合终端工作流的原生 GPUI 桌面客户端。它通过 Rust 工作区统一实现桌面界面、终端渲染、传输协议、持久化、认证、网络工具、AI 辅助终端操作、导入导出、诊断、加密同步与备份。

- **NyaTerm 是** 面向开发者、系统管理员和 DevOps 工程师的 SSH 客户端
- **NyaTerm 是** 支持标签页、横向分屏和纵向分屏的终端工作区
- **NyaTerm 是** 带传输队列和“本地编辑后回传”流程的 SFTP 文件浏览器
- **NyaTerm 支持** SSH、本地终端、Telnet、串口、RDP 和 VNC 会话

---

<a name="为什么选择-nyaterm"></a>
# 为什么选择 NyaTerm

NyaTerm 适合每天在服务器、本地命令、设备调试和配置文件之间来回切换的工作方式。

- **工作区优先**：用标签页、分屏、侧边面板和子窗口组织相关任务
- **远程操作不脱离上下文**：在会话旁浏览 SFTP 文件、跟随终端路径、管理传输、编辑远程文件
- **面向安全的工作流**：管理凭据、私钥、known hosts、OTP、锁屏和主密码保护的本地存储
- **可迁移配置**：从已有工具导入会话，导出加密 `.nya` 备份，并通过 WebDAV 或 S3 兼容存储同步加密快照
- **恰到好处的 AI**：从当前终端上下文生成命令、检查输出，并在审批后执行多步操作

---

<a name="功能特性"></a>
# 功能特性

## 会话与工作区

- 支持 SSH、本地终端、Telnet、串口、RDP 和 VNC 会话
- 多标签页工作区，标签页内支持横向/纵向分屏、标签拖拽停靠和布局恢复
- RDP 与 VNC 远程桌面 pane；VNC 当前支持 direct TCP、None / classic VNC Auth、Raw / ZRLE / Tight / Tight JPEG framebuffer 更新、窗口缩放、有界重连，以及 Latin-1 文本剪贴板交换
- 已保存连接支持分组、图标、元数据、复制、键盘复制、重连和导入
- 命令面板与会话快速切换器，可查找应用动作、已打开会话、已保存连接和新建会话入口
- 主窗口支持 `Background Image` 背景图自定义，可调 `cover` / `contain` / `stretch` / `tile` 以及 `Background Content Opacity`
- 左右活动栏覆盖文件浏览器、网络、Security/Auth、Sync & Backup、AI Assistant、Notes、已保存连接、活动会话、命令历史、资产监控和资源监控
- Notes 面板和编辑器支持树形导航、右键菜单、自动保存、工具栏控制，以及可参与同步 / 备份的持久化
- 资产监控工作区支持连接分组视图、面包屑导航、表格 / 卡片布局，并集成资源、GPU 与 NPU 状态
- SSH 会话远程主机监控面板覆盖资源监控、NVIDIA / Ascend GPU 与 NPU 总览、进程管理和 Docker 管理
- 设置、新建连接、快捷命令编辑、远程文件编辑和自动上传提示使用独立子窗口
- 支持托盘、关闭时最小化到托盘和隐藏主窗口

## 终端体验

- 终端搜索支持结果跳转和搜索历史，另有复制粘贴、右键菜单和选中文本操作
- 命令历史、模糊建议、用于过滤噪音命令的长度配置，以及交互式程序中的建议抑制
- 可选行号和时间戳 gutter
- 可选动作链接，识别 IPv4、`host:port` 和压缩包文件名
- 可选关键词高亮，支持内置预设、自定义规则和 JSON 导入
- 支持终端缩放、工作区 padding、字体粗细、原生输入法和图片路径粘贴
- 可直接让 AI 解释最近输出，并通过 `Terminal Output Lines` 控制 Agent 命令的终端内联输出
- 大输出保护、可配置回滚缓冲区、SSH Keep-Alive 和会话录制
- 可从选中文本发起在线搜索和翻译
- 支持直接在终端中使用 Zmodem 进行文件传输
- 支持自定义终端与界面操作的键盘快捷键，Telnet / 串口会话还支持 `Backspace Mode`

## SFTP 与文件工作流

- SSH 会话内置 SFTP 文件浏览器
- 支持上传、下载、重命名、移动、删除、属性、新建文件/文件夹和 OpenSSH 兼容符号链接
- 支持文件夹上传 / 下载、多目录上传选择、多选、可编辑路径栏，以及与终端 cwd 手动/自动同步
- 传输队列支持速度显示、暂停、继续、取消、失败重试、重复目标处理、时间戳保留和并发配置
- 增强 SCP 和目录传输处理，并在可用时保留原始属性
- 可在本地编辑远程文件，保存后通过 watcher 驱动的自动上传流程回传
- 多标签只读远程文件预览，支持文本、JSON、Markdown、CSV/TSV、图片和 PDF，带每标签会话标识、缩放/旋转/适配控制以及分页 PDF 渲染
- 预览的加载与解码在渲染线程之外进行，并按格式设置大小上限，超大或不支持的文件会给出明确提示而不会阻塞界面
- Windows 下支持从系统文件管理器拖拽文件或文件夹到文件浏览器上传
- Zmodem 传输会记录本地路径，并可在系统文件管理器中显示已完成文件

## 安全、认证与网络

- 密码认证、私钥、主机密钥校验和本地加密持久化
- 凭据管理支持基于正则的终端自动填充
- OTP 管理支持 TOTP/HOTP、二维码导入和 SSH 自动填充
- 支持 SOCKS5、HTTP 和 ProxyCommand 代理配置；支持带环路阻止的 SSH 跳板机；支持本地/远程/动态隧道
- 支持 SSH X11 转发，可配合本地 X server 使用
- 支持锁屏、空闲应用锁定、主密码、诊断设置、本地日志管理和诊断包导出

## 同步、备份与迁移

- 通过 WebDAV、S3 兼容存储和 GitHub Gist 进行加密云同步与备份
- 启用同步、备份、加密导入导出或定时加密备份前必须设置主密码
- 支持启动同步检查、本地变更后的防抖自动推送、详细状态更新和定时备份保留策略
- 支持手动测试、推送、拉取、备份、远程备份恢复和快照级冲突处理
- 支持从 Xshell、MobaXterm、WindTerm 和 NyaTerm JSON 定义导入会话
- 支持完整 NyaTerm 配置的加密 `.nya` 导入导出

---

<a name="截图"></a>
# 截图

## 工作区

在同一个标签页和分屏工作区中管理 SSH、本地 Shell、Telnet 和串口会话。

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./docs-site/static/img/home/overview-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="./docs-site/static/img/home/overview-light.png">
    <img alt="NyaTerm 工作区概览" src="./docs-site/static/img/home/overview-light.png">
  </picture>
</p>

## 外观与背景图

为主窗口设置本地壁纸，并通过 `Image Sizing`、`Image Opacity` 和 `Background Content Opacity` 平衡视觉效果与面板可读性。

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./docs-site/static/img/home/cover-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="./docs-site/static/img/home/cover-light.png">
    <img alt="NyaTerm 背景图自定义" src="./docs-site/static/img/home/cover-light.png">
  </picture>
</p>

## 终端增强

在终端中使用命令历史、搜索、翻译、动作链接、时间戳、关键词高亮和大输出保护。

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./docs-site/static/img/home/terminal-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="./docs-site/static/img/home/terminal-light.png">
    <img alt="NyaTerm 终端功能" src="./docs-site/static/img/home/terminal-light.png">
  </picture>
</p>

## 远程文件

在终端旁浏览 SFTP 文件、管理传输队列，并把本地编辑器保存的修改回传到远端路径。

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./docs-site/static/img/home/files-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="./docs-site/static/img/home/files-light.png">
    <img alt="NyaTerm SFTP 文件工作流" src="./docs-site/static/img/home/files-light.png">
  </picture>
</p>

## 安全与网络工具

在专用面板中管理凭据、OTP、known hosts、代理、跳板机和 SSH 隧道。

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./docs-site/static/img/home/security-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="./docs-site/static/img/home/security-light.png">
    <img alt="NyaTerm 安全与网络工具" src="./docs-site/static/img/home/security-light.png">
  </picture>
</p>

## 同步与备份

通过 WebDAV 或 S3 兼容存储同步加密的可移植配置快照，并按需恢复远程备份。

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./docs-site/static/img/home/sync-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="./docs-site/static/img/home/sync-light.png">
    <img alt="NyaTerm 同步与备份" src="./docs-site/static/img/home/sync-light.png">
  </picture>
</p>

---

<a name="支持平台"></a>
# 支持平台

| 系统 | 支持情况 |
| :--- | :--- |
| **Windows** | Windows 10/11，x64 / arm64 |
| **macOS** | macOS 12+，Intel / Apple Silicon |
| **Linux** | Ubuntu 20.04+、Fedora 36+、Arch Linux 及类似发行版 |

基于 Ubuntu 20.04、使用 **glibc 2.31** 的定制 x86-64 桌面，请使用专门的
[focal 离线兼容构建](scripts/compat/linux-focal/README.md)。普通 Linux 发布包在
更新的发行版上编译，可能要求更高版本的 glibc。兼容包包含全部辅助程序及 ABI
报告，默认优先系统 GPU 驱动，并保留显式的独立 OpenGL CPU 回退入口；目标机需可用的 X11 会话。

可从 [nyaterm.app](https://nyaterm.app) 或 [Releases](https://github.com/nyakang/nyaterm/releases) 页面下载安装包。

---

<a name="支持的会话类型"></a>
# 支持的会话类型

| 类型 | 典型场景 | 说明 |
|------|----------|------|
| SSH | Linux / Unix 远程服务器 | 支持 SFTP、OTP、资源监控、代理、跳板机和隧道 |
| 本地终端 | 本地 Shell 工作流 | 使用本机 Shell 路径和工作目录 |
| Telnet | 旧设备或实验环境 | 轻量终端会话，不包含 SSH 专属能力，并支持 `Backspace Mode` 在 `Ctrl+H (BS)` 与 `DEL (0x7F)` 间切换 |
| 串口 | 路由器、开发板、嵌入式设备 | 可配置端口、波特率、数据位、校验位、停止位和 `Backspace Mode` |

---

<a name="快速开始"></a>
# 快速开始

## 下载

Windows、macOS、Linux 的快照与正式构建发布在本分支的 [Releases](https://github.com/houxianghui/nyaterm/releases) 页面。

| 平台 | 格式 |
|------|------|
| Windows | `.exe` / 便携版 `.zip` |
| macOS | `.dmg` |
| Linux | `.deb` / `.AppImage` / `.rpm` |

Windows 便携版解压后运行 `NyaTerm.exe`，数据保存在旁边的 `data/`。GPUI 的 **Help → 检查更新** 使用签名更新清单，支持适用安装方式的下载和安装；Homebrew、deb/rpm 用户通过包管理器更新。

**从 1.x 迁移：Windows 便携版和旧 MSI 必须手动升级。** 先退出并备份完整数据及密钥；便携版完整覆盖程序文件并保留 `data/`，MSI 备份后卸载旧版再安装 NSIS。不能只复制主程序，也不承诺所有旧安装方式都能自动升级。详见 [2.0 迁移指南](https://nyaterm.app/docs/getting-started/migration-v2)。

### macOS

macOS 用户可以通过 Homebrew 安装 NyaTerm：

```bash
brew install nyakang/nyaterm/nyaterm
```

该命令会使用 [`nyakang/homebrew-nyaterm`](https://github.com/nyakang/homebrew-nyaterm) tap，并安装 `nyaterm` cask。也可以从 [nyaterm.app](https://nyaterm.app) 或 [Releases](https://github.com/nyakang/nyaterm/releases) 下载 `.dmg` 安装包，然后将 NyaTerm 拖入 `/Applications`。

NyaTerm 目前还没有使用 Apple Developer 证书签名。安装后如果 macOS 提示应用已损坏或无法打开，可以移除 quarantine 属性后再打开：

```bash
sudo xattr -cr /Applications/NyaTerm.app
```

### Arch Linux / AUR

Arch Linux 用户可以通过 AUR 安装 NyaTerm：

```bash
yay -S nyaterm-bin
```

或使用 `paru`：

```bash
paru -S nyaterm-bin
```

也可以手动从 AUR 构建安装：

```bash
git clone https://aur.archlinux.org/nyaterm-bin.git
cd nyaterm-bin
makepkg -si
```

AUR 软件包：[`nyaterm-bin`](https://aur.archlinux.org/packages/nyaterm-bin)

> `nyaterm-bin` 为社区维护的 AUR 二进制包。如果 AUR 版本暂未同步到最新发布版，可以从 [Releases](https://github.com/nyakang/nyaterm/releases) 下载官方安装包。


## 开发环境要求

- 通过 [rustup](https://rustup.rs/) 安装 Rust stable
- GPUI 和操作系统所需的平台依赖

## 本地开发

```bash
git clone https://github.com/nyakang/nyaterm.git
cd nyaterm
cargo run -p nyaterm-app --bin nyaterm
```

## 项目结构

```text
├── crates/
│   ├── nyaterm-app/             # 原生 GPUI 应用入口和资源
│   ├── nyaterm-desktop/         # GPUI 应用组合与功能状态
│   ├── nyaterm-core/            # 与 UI 无关的领域模型和策略
│   ├── nyaterm-terminal/        # 终端状态机和解析
│   ├── nyaterm-terminal-gpui/   # GPUI 终端布局、输入和绘制
│   ├── nyaterm-transport/       # SSH、PTY、SFTP、Telnet、串口和传输
│   ├── nyaterm-store/           # 持久化和数据库兼容层
│   ├── nyaterm-ui/              # 共享 GPUI 控件和主题集成
│   ├── nyaterm-remote-desktop/  # 与 UI 无关的 RDP/VNC 会话和类型化 IPC
│   ├── nyaterm-rdp-helper/      # 隔离的 IronRDP 协议 helper 进程
│   ├── nyaterm-vnc-helper/      # 隔离的 VNC 协议 helper 进程
│   └── nyaterm-otp/             # OTP 兼容实现
├── docs-site/                   # Docusaurus 文档站点
├── Cargo.toml                   # Rust workspace 定义
└── Cargo.lock
```

---

<a name="鸣谢"></a>
# 鸣谢
感谢以下项目和库使 NyaTerm 成为可能：
- [WindTerm](https://github.com/kingToolbox/WindTerm) - 启发了 NyaTerm 的设计和功能
- [tabby](https://github.com/Eugeny/tabby) - 一个优秀的跨平台终端，提供了很多设计灵感
- [Alacritty](https://github.com/alacritty/alacritty) - NyaTerm 使用的终端模拟组件
- [russh](https://github.com/warp-tech/russh) - SSH 客户端和服务端 Rust 库

---

<a name="license"></a>
# 许可证

本项目基于 [Apache License 2.0](LICENSE) 开源。
