# Antigravity Tools 🚀

<div align="center">
  <img src="public/icon.png" alt="Antigravity Logo" width="120" height="120" style="border-radius: 24px; box-shadow: 0 10px 30px rgba(0,0,0,0.15);">

  <h3>Professional Account Management for AI Services</h3>
  <p>Manage your Gemini / Claude accounts with ease. Unlimited Possibilities.</p>
  
  <p>
    <a href="https://github.com/lbjlaq/Antigravity-Manager">
      <img src="https://img.shields.io/badge/Version-2.0.0-blue?style=flat-square" alt="Version">
    </a>
    <img src="https://img.shields.io/badge/Tauri-v2-orange?style=flat-square" alt="Tauri">
    <img src="https://img.shields.io/badge/React-18-61DAFB?style=flat-square" alt="React">
    <img src="https://img.shields.io/badge/License-CC--BY--NC--SA--4.0-lightgrey?style=flat-square" alt="License">
  </p>

  <p>
    <a href="#-downloads">📥 Download</a> • 
    <a href="#-features">✨ Features</a> • 
    <a href="#-comparison">🆚 Comparison</a>
  </p>
  
  <p>
    <a href="./README.md">🇨🇳 简体中文</a> | 
    <strong>🇺🇸 English</strong>
  </p>
</div>

---

<div align="center">
  <img src="docs/images/accounts-dark.png" alt="Antigravity Dark Mode" style="border-radius: 12px; box-shadow: 0 20px 40px rgba(0,0,0,0.2); width: 100%; max-width: 800px;">
  <p><i>(Deep Dark Mode: Increased productivity)</i></p>
</div>

## 🎨 Gallery

<div align="center">

| **Light Mode** | **Dark Mode** |
| :---: | :---: |
| <img src="docs/images/dashboard-light.png" width="100%" style="border-radius: 8px;"> | <img src="docs/images/accounts-dark.png" width="100%" style="border-radius: 8px;"> |
| **Dashboard** | **Accounts** |

| <img src="docs/images/accounts-light.png" width="100%" style="border-radius: 8px;"> | <img src="docs/images/settings-dark.png" width="100%" style="border-radius: 8px;"> |
| **List View** | **Settings** |

</div>

---

**Antigravity Tools** is a **modern account management tool** built for AI developers and power users.

As the 2.0 rewrite of [Antigravity Manager](https://github.com/lbjlaq/Antigravity-Manager), it leverages the high-performance **[Tauri v2](https://v2.tauri.app/)** + **[React](https://react.dev/)** stack, evolving from a heavy Python GUI into a lightweight, blazing-fast native application.

It helps you effortlessly manage dozens of **Google Gemini** and **Claude 3.5** accounts, monitoring Quotas in real-time, and intelligently switching accounts when quotas are exhausted, enabling an "unlimited" AI experience.

> ⚠️ **Note**: The project repository URL remains unchanged at [lbjlaq/Antigravity-Manager](https://github.com/lbjlaq/Antigravity-Manager).
>
> **Looking for v1.0?**
> The source code for v1.0 (Python/Flet) has been archived in the [v1 branch](https://github.com/lbjlaq/Antigravity-Manager/tree/v1). Switch branches to view or maintain the legacy version.

## 🆚 Why v2.0? (Comparison)

| Feature | 🐢 v1.0 (Legacy) | 🚀 v2.0 (New) | Improvement |
| :--- | :--- | :--- | :--- |
| **Core Tech** | Python + Flet | **Rust (Tauri)** + **React** | **Performance Leap** |
| **Bundle Size** | ~80 MB | **~10 MB** | **87% Smaller** |
| **Startup Time** | Slow (Interpreted) | **Instant** (Native Binary) | **Blazing Fast** |
| **Memory Usage** | High (>200MB) | **Tiny** (<50MB) | **Efficient** |
| **UI/UX** | Basic Material | **Modern Glassmorphism** | **Beautiful** |
| **Security** | Plain text / Simple obfuscation | **Local JSON Storage** | **Transparent & Controllable** |
| **Extensibility** | Hard (Python dependency hell) | **Easy** (Standard Web Tech) | **Rich Ecosystem** |

## ✨ Key Features

### 📊 Dashboard
- **Global Overview**: Real-time display of total accounts, average quota for each model, health status at a glance.
- **Smart Recommendation**: Automatically filters the "Best Account" with the most available quota, supports one-click switching to always use optimal resources.
- **Status Monitoring**: Real-time highlighting of accounts with low quota alerts to avoid development interruptions.

### 👥 Account Management
- **Multi-channel Import**:
    - 🔥 **OAuth Authorization**: Supports browser-based Google login authorization to automatically acquire Tokens (Recommended).
    - 📋 **Manual Add**: Supports direct pasting of Refresh Tokens.
    - 📂 **V1 Migration**: Supports scanning and batch importing old data from v1 version (`~/.antigravity-agent`).
    - 🔄 **Local Sync**: Supports automatically reading and importing currently logged-in accounts from IDE (Cursor/Windsurf) local database.
- **Batch Operations**: Batch refresh quota, batch export backup (JSON), batch delete.
- **Search & Filter**: Supports fast retrieval by email keywords, managing dozens of accounts with ease.

### 🔄 Quota Sync
- **Auto Refresh**: Configurable background automatic polling for latest quota information of all accounts.
- **Token Keep-alive**: Built-in Token auto-refresh mechanism, auto-renew upon expiration to ensure connection validity.
- **Precise Display**: Clearly displays specific remaining percentage and reset time for Gemini / Claude models.

### 🛠️ System Integration
- **Tray Resident**: Minimized to system tray, saving taskbar space, running silently in background.
- **Quick Actions**: Tray menu supports one-click viewing of current quota and quick switching to next available account.
- **Secure Storage**: Uses local JSON format storage, all Token data is saved only on user device, never uploaded to cloud.

### ⚙️ Settings
- **Internationalization**: Native support for **Simplified Chinese** / **English** real-time switching.
- **Theme Adaptation**: Perfectly adapts to system Dark / Light mode, eye-friendly for night use.
- **Data Management**: Supports custom data export path and one-click log cache cleaning.

## 🛠️ Tech Stack

Built with cutting-edge modern tech stack, ensuring high performance and maintainability:

| Module | Tech Stack | Description |
| :--- | :--- | :--- |
| **Frontend** | React 18 + TypeScript | UI Construction & Logic |
| **UI Framework** | TailwindCSS + DaisyUI | Modern Atomic CSS Library |
| **Backend** | Tauri v2 (Rust) | High-performance System Interaction |
| **Storage** | Local JSON | Local Config & Data Storage |
| **State** | Zustand | Lightweight Global State Management |
| **Network** | Reqwest (Async) | Async Network Requests |

## 📦 Installation & Run

### 📥 Download

Visit the [Releases Page](https://github.com/lbjlaq/Antigravity-Manager/releases) to download the installer for your system:

- **macOS**: Supports Intel (`.dmg`) and Apple Silicon (`.dmg`)
- **Windows**: `.exe` Installer
- **Linux**: `.deb` or `.AppImage` *(Theoretical support, untested, feedback welcome)*

### 💻 Development

If you're a developer and want to contribute:

```bash
# 1. Clone project
git clone https://github.com/lbjlaq/antigravity-tools.git

# 2. Install dependencies
npm install

# 3. Start dev server (Frontend + Backend)
npm run tauri dev
```

### 🏗️ Build

```bash
# Build Universal macOS App (Intel & Apple Silicon)
npm run build:universal
```

## 👤 Author

**Ctrler**

- 💬 WeChat: `Ctrler`
- 🐙 GitHub: [@lbjlaq](https://github.com/lbjlaq)

## 📄 License

Copyright © 2025 Antigravity. All rights reserved.

This project is licensed under the **[CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/)** license.
Commercial use of this project or its derivatives is strictly prohibited.
