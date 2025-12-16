# Antigravity Tools 🚀

<div align="center">
  <img src="public/icon.png" alt="Antigravity Logo" width="120" height="120" style="border-radius: 24px; box-shadow: 0 10px 30px rgba(0,0,0,0.15);">

  <h3>Professional Account Management for AI Services</h3>
  <p>Manage your Gemini / Claude accounts with ease. Unlimited Possibilities.</p>
  
  <p>
    <a href="https://github.com/lbjlaq/Antigravity-Manager">
      <img src="https://img.shields.io/badge/Version-2.1.2-blue?style=flat-square" alt="Version">
    </a>
    <img src="https://img.shields.io/badge/Tauri-v2-orange?style=flat-square" alt="Tauri">
    <img src="https://img.shields.io/badge/React-18-61DAFB?style=flat-square" alt="React">
    <img src="https://img.shields.io/badge/License-CC--BY--NC--SA--4.0-lightgrey?style=flat-square" alt="License">
  </p>

  <p>
    <a href="#-Downloads">📥 下载最新版 (macOS/Windows/Linux)</a> • 
    <a href="#-Features">✨ 核心特性</a> • 
    <a href="#-Comparison">🆚 版本对比</a>
  </p>

  <p>
    <strong>🇨🇳 简体中文</strong> | 
    <a href="./README_EN.md">🇺🇸 English</a>
  </p>
</div>

---

---

<div align="center">
  <img src="docs/images/accounts-dark.png" alt="Antigravity Dark Mode" style="border-radius: 12px; box-shadow: 0 20px 40px rgba(0,0,0,0.2); width: 100%; max-width: 800px;">
  <p><i>（Deep Dark: 沉浸式暗色模式，专注开发）</i></p>
</div>

## 🎨 界面预览 (Gallery)

<div align="center">

| **Light Mode (清爽明亮)** | **Dark Mode (深邃护眼)** |
| :---: | :---: |
| <img src="docs/images/dashboard-light.png" width="100%" style="border-radius: 8px;"> | <img src="docs/images/accounts-dark.png" width="100%" style="border-radius: 8px;"> |
| **仪表盘 Dashboard** | **账号管理 Accounts** |

| <img src="docs/images/accounts-light.png" width="100%" style="border-radius: 8px;"> | <img src="docs/images/settings-dark.png" width="100%" style="border-radius: 8px;"> |
| **列表视图 List View** | **全局设置 Settings** |

</div>

---

**Antigravity Tools** 是一款专为 AI 开发者和重度用户打造的 **现代化账号管理工具**。

作为 [Antigravity Manager](https://github.com/lbjlaq/Antigravity-Manager) 的 2.0 重构版本，它采用了高性能的 **[Tauri v2](https://v2.tauri.app/)** + **[React](https://react.dev/)** 技术栈，将原本笨重的 Python GUI 进化为轻量、极速的原生应用。

它可以帮助你轻松管理数十个 **Google Gemini**、**Claude 3.5** 等 AI 服务账号，实时监控配额（Quota），并在配额耗尽时智能切换，助你实现 "无限" 的 AI 调用体验。

> ⚠️ **注意**: 本项目仓库地址保持不变，继续沿用 [lbjlaq/Antigravity-Manager](https://github.com/lbjlaq/Antigravity-Manager)。
>
> **寻找 1.0 版本?**
> v1.0 (Python/Flet) 版本的完整源码已归档至 [v1 分支](https://github.com/lbjlaq/Antigravity-Manager/tree/v1)。如需查看或维护旧版，请切换分支查看。

## 🆚 为什么选择 2.0 ? (Comparison)

| 特性 Comparison | 🐢 v1.0 (Legacy) | 🚀 v2.0 (New) | 提升 |
| :--- | :--- | :--- | :--- |
| **技术核心** | Python + Flet | **Rust (Tauri)** + **React** | **性能质变** |
| **安装包大小** | ~80 MB | **~10 MB** | **体积减少 87%** |
| **启动速度** | 慢 (需加载 Python 解释器) | **秒开** (原生二进制) | **极速响应** |
| **内存占用** | 高 (>200MB) | **极低** (<50MB) | **更省资源** |
| **界面交互** | 基础 Material 风格 | **现代化 Glassmorphism** | **颜值正义** |
| **安全性** | 明文/简单混淆 | **本地 JSON 存储** | **透明可控** |
| **扩展性** | 难 (Python 依赖地狱) | **易** (标准 Web 技术栈) | **生态丰富** |

## ✨ 核心特性 (Features)

### 📊 仪表盘 (Dashboard)
- **全局概览**: 实时展示账号总数、各模型平均配额，健康度一目了然。
- **智能推荐**: 自动筛选当前配额最充足的 "最佳账号"，支持一键快速切换，始终使用最优资源。
- **状态监控**: 实时高亮显示低配额告警账号，避免开发中断。

### 👥 账号管理 (Account Management)
- **多渠道导入**:
    - 🔥 **OAuth 授权**: 支持拉起浏览器进行 Google 登录授权，自动获取 Token (推荐)。
    - 📋 **手动添加**: 支持直接粘贴 Refresh Token 进行添加。
    - 📂 **V1 迁移**: 支持从 v1 版本 (`~/.antigravity-agent`) 自动扫描并批量导入旧数据。
    - 🔄 **本地同步**: 支持从 IDE (Cursor/Windsurf) 本地数据库自动读取并导入当前登录账号。
- **批量操作**: 提供批量刷新配额、批量导出备份 (JSON)、批量删除功能。
- **搜索过滤**: 支持按邮箱关键字快速检索，管理数十个账号依然轻松。

### 🔄 配额同步 (Quota Sync)
- **自动刷新**: 可配置后台自动定时轮询所有账号的最新配额信息。
- **Token 保活**: 内置 Token 自动刷新机制，过期自动续期，确保连接时刻有效。
- **精准展示**: 清晰展示 Gemini / Claude 等不同模型的具体剩余百分比和重置时间。

### 🛠️ 系统集成 (System Integration)
- **托盘常驻**: 程序可最小化至系统托盘，不占用任务栏空间，后台静默运行。
- **快捷操作**: 托盘菜单支持一键查看当前账号配额、快速切换下一个可用账号。
- **安全存储**: 采用本地 JSON 格式存储，所有 Token 数据仅保存在用户设备，绝不上传云端。

### ⚙️ 个性化设置 (Settings)
- **国际化**: 原生支持 **简体中文** / **English** 实时切换。
- **主题适配**: 完美适配系统的深色 (Dark Mode) / 浅色模式，夜间使用更护眼。
- **数据管理**: 支持自定义数据导出路径，并提供日志缓存一键清理功能。

## 🛠️ 技术栈

本项目采用前沿的现代技术栈构建，确保了应用的高性能与可维护性：

| 模块 | 技术选型 | 说明 |
| :--- | :--- | :--- |
| **Frontend** | React 18 + TypeScript | UI 构建与逻辑处理 |
| **UI Framework** | TailwindCSS + DaisyUI | 现代化原子类样式库 |
| **Backend** | Tauri v2 (Rust) | 高性能、安全的系统底层交互 |
| **Storage** | Local JSON | 本地配置与数据存储 |
| **State** | Zustand | 轻量级全局状态管理 |
| **Network** | Reqwest (Async) | 异步网络请求处理 |

## 📦 安装与运行

### 📥 下载安装

前往 [Releases 页面](https://github.com/lbjlaq/Antigravity-Manager/releases) 下载对应系统的安装包：

- **macOS**: 支持 Intel (`.dmg`) 和 Apple Silicon (`.dmg`)
- **Windows**: `.exe` 安装包
- **Linux**: `.deb` 或 `.AppImage` *(理论支持，尚未经完整测试，欢迎反馈)*

### 💻 开发环境启动

如果您是开发者，想要贡献代码：

```bash
# 1. 克隆项目
git clone https://github.com/lbjlaq/antigravity-tools.git

# 2. 安装前端依赖
npm install

# 3. 启动开发模式 (Frontend + Backend)
npm run tauri dev
```

### 🏗️ 构建发布

```bash
# 构建通用 macOS 应用 (同时支持 Intel & Apple Silicon)
npm run build:universal
```

## ❓ 常见问题 (FAQ)

### ⚠️ 打开应用提示 "已损坏" 或 "无法打开"？

如果在 macOS 上打开应用时提示 **“Antigravity Tools 已损坏，无法打开”**，这是 macOS Gatekeeper 对未签名应用的默认拦截机制。

**解决方法：**

1. 打开终端 (Terminal)。
2. 复制并执行以下命令 (可能需要输入密码)：

```bash
sudo xattr -rd com.apple.quarantine "/Applications/Antigravity Tools.app"
```

> 注意：请根据实际安装位置调整路径，如果安装在“应用程序”目录，通常就是上面的路径。

## 📅 更新日志 (Changelog)

### v2.1.2 (2025-12-16)
- **🪟 Windows 兼容性**: 修复了在 Windows 下无法打开数据目录的问题。
- **📁 路径显示**: 设置页面现在显示数据目录的完整绝对路径，方便查找。

### v2.1.0 (2025-12-15)
- **🔥 OAuth 重构**:
  - 修复端口冲突问题 (改为随机端口)。
  - 新增 **"复制链接"** 功能，支持手动在浏览器完成验证。
  - 新增 **"取消授权"** 按钮，支持主动释放资源。
- **🎨 图标升级**:
  - 全新设计的 macOS 风格圆角图标 (Squircle)。
  - 优化托盘图标显示效果。
  - 修复旧版 macOS 下图标显示过大的问题。
- **📖 文档**: 新增常见问题 (FAQ) 指引。

## 👤 作者

**Ctrler**

- 💬 微信公众号: `Ctrler`
- 🐙 GitHub: [@lbjlaq](https://github.com/lbjlaq)

## 📄 版权说明

Copyright © 2025 Antigravity. All rights reserved.

本项目采用 **[CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/)** 协议许可。
禁止将本项目或其衍生作品用于任何商业用途。

