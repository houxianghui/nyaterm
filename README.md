<p align="center">
  <img src="./src-tauri/icons/128x128.png" alt="NyaTerm" width="128" height="128">
</p>

<h1 align="center">NyaTerm — macOS Visual Fork</h1>

<p align="center">
  <strong>A UI-focused fork of <a href="https://github.com/nyakang/nyaterm">NyaTerm</a> with a full macOS-style visual overhaul.</strong>
</p>

<p align="center">
  <a href="./README.md">English</a> · <a href="./README.zh-CN.md">简体中文</a>
</p>

---

## About This Fork

This fork tracks [`nyakang/nyaterm`](https://github.com/nyakang/nyaterm) and merges upstream changes regularly. Everything upstream ships — SSH, local shells, Telnet, Serial, RDP, VNC, SFTP, tunnels, OTP, AI assistance, encrypted sync — works here exactly as it does upstream.

What this fork changes is the **look and feel**: a complete macOS-inspired redesign of the default appearance, an interface refresh across the workspace, and a new app icon. For the full feature list, see the [upstream README](https://github.com/nyakang/nyaterm#readme).

---

## Differences from Upstream

### macOS Visual Overhaul (default appearance)

- **macOS neutral-gray default themes** (dark + light) replacing the GitHub-palette defaults, with the macOS system blue `#0a84ff` accent
- **Mac-soft terminal palettes** — low-saturation 16-color ANSI palettes for both dark and light themes
- **Larger, coherent corner radius** — 10px base scale (14px containers, 10px cards, 8px inputs, 6px small elements)
- **Soft layered shadows** — low-opacity, wide-blur diffusion shadows instead of hard elevation shadows
- **SF Pro-first typography** — system font stack led by SF Pro, Inter fallback on Windows
- **Restrained motion** — 150ms float animations with softer zoom/ease on dialogs, popovers, and menus; optional interface-animation toggle in settings

### Interface Redesign

- **Safari-style floating capsule tabs** in the tab strip
- **Edge-style hub menu** replacing the classic menu bar
- **Acrylic menu surfaces** — translucent, blurred menus and overlays
- **Capsule input bars** throughout the UI
- **Finder-style selection** in the saved-connections list
- **macOS-style activity bar and panel headers** with blue menu highlights
- **Immersive header** with inset floating panel cards and tighter panel gaps
- **Hover-only split resize lines**; rounded, transparent child windows
- **Windows polish** — DWM non-client border removed on transparent windows for a clean frameless look
<img width="1920" height="1032" alt="image" src="https://github.com/user-attachments/assets/726dc223-c06a-4e0c-b55c-9db5b3490b31" />
<img width="800" height="560" alt="image" src="https://github.com/user-attachments/assets/f8da9b06-7c45-4b19-8fb1-6e7be7efa746" />

<img width="1920" height="1032" alt="image" src="https://github.com/user-attachments/assets/9d90ca1b-376b-43ec-b781-2b18d229d7fa" />
<img width="800" height="560" alt="image" src="https://github.com/user-attachments/assets/ad2f73b4-6aea-45b7-8e3e-326eaf3099ff" />
### Branding

- **New terminal app icon** — OS bundle icons (Windows/macOS/Linux/Android) and the in-app logo

### Build / CI

- Main snapshot builds are triggered manually (`workflow_dispatch`); the snapshot release tag is updated via REST API
- Fork-specific Tauri updater signing keypair
- Windows 7 build target dropped

---

## Download

Snapshot and release builds for Windows, macOS, and Linux are published on this fork's [Releases](https://github.com/houxianghui/nyaterm/releases) page.

---

## Development

```bash
git clone https://github.com/houxianghui/nyaterm.git
cd nyaterm
pnpm install
pnpm tauri dev
```

Requirements: Node.js 18+, pnpm, and Rust stable via [rustup](https://rustup.rs/).

---

## Credits

- [NyaTerm](https://github.com/nyakang/nyaterm) — the upstream project this fork is based on; all core functionality is upstream's work
- [WindTerm](https://github.com/kingToolbox/WindTerm), [tabby](https://github.com/Eugeny/tabby), [xterm.js](https://xtermjs.org/), [russh](https://github.com/warp-tech/russh) — upstream's inspirations and foundations

---

<a name="license"></a>
# License

This project is licensed under the [MIT License](LICENSE).
