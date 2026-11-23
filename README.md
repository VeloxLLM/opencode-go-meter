<div align="center">

# OpenCode Go Meter

**A lightweight Windows tray meter for OpenCode Go usage.**

[![Rust](https://img.shields.io/badge/Rust-1.90%2B-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Windows](https://img.shields.io/badge/Platform-Windows%2010%20%2F%2011-0078D6?logo=windows&logoColor=white)](https://www.microsoft.com/windows)
[![License](https://img.shields.io/badge/License-MIT-2ea44f.svg)](LICENSE)
[![Native](https://img.shields.io/badge/UI-Native%20Win32-4b5563.svg)](#native-window)
[![Languages](https://img.shields.io/badge/UI%20Languages-8-4b5563.svg)](#language-support)

English · [中文](#中文)

[Features](#features) · [Quick start](#quick-start) · [How it works](#how-it-works) · [Build from source](#build-from-source) · [Security](#security)

</div>

---

OpenCode Go enforces three overlapping limits:

| Window | Share of monthly allowance | Resets |
|---|---:|---|
| 5-hour rolling | 20% | 5 hours after first use |
| Weekly | 50% | Monday 00:00 UTC |
| Monthly | 100% | Start of the paid period |

Checking the Console every time breaks focus. **OpenCode Go Meter** stays in the Windows notification area and keeps the numbers one glance away.

> **Native Win32 + Rust.** No Electron, no WebView, no HTML.

## Preview

![OpenCode Go Meter native window](assets/preview.png)

The tray icon is color-coded from the 5-hour window: **green** below 50%, **amber** from 50% to 79%, **red** at 80% or above, and **grey** when authentication fails.

## Language support

The UI language is auto-detected from Windows. Eight languages are built in:

English · 简体中文 · 日本語 · 한국어 · Español · Français · Deutsch · Русский

Force a language with:

```powershell
$env:OPENCODE_GO_LANG = "zh"   # en / zh / ja / ko / es / fr / de / ru
```

## Features

| | |
|---|---|
| **Tray-first** | Hover for `5h` and monthly percentages without opening anything. |
| **Right-click menu** | 5-hour, weekly, and monthly details; refresh; open Console; quit. |
| **Double-click window** | Native Win32 window with three progress bars, status, and reset times. |
| **Automatic refresh** | Polls once per minute; manual refresh from the menu or window. |
| **Multi-language UI** | Eight languages, auto-detected, overridable via `OPENCODE_GO_LANG`. |
| **Credential discovery** | Reads the active OpenCode Go credential from OpenCode Desktop's local database. |
| **Read-only by design** | Opens `opencode.db` with `mode=ro`; never writes to OpenCode's data. |
| **Single binary** | ~4 MB release executable; SQLite is bundled, no external runtime. |

## Quick start

### Option A — run the prebuilt executable

1. Download `opencode-go-tray.exe` from the latest release.
2. Place it in a permanent folder.
3. Double-click it. The icon appears in the notification area.

### Option B — build from source

```powershell
git clone https://github.com/VeloxLLM/opencode-go-meter.git
cd opencode-go-meter
.\run-tray.bat
```

`run-tray.bat` builds a release binary on first run, then starts it.

> **Prerequisite:** sign in to **OpenCode Go** in OpenCode Desktop at least once. The app reads the active credential that OpenCode Desktop stores locally.

If the icon is hidden, click the **^** overflow button in the notification area.

## Usage

| Action | Result |
|---|---|
| Hover the tray icon | Tooltip with 5-hour and monthly percentages |
| Right-click | Menu with 5h / weekly / monthly details, refresh, Console, quit |
| Double-click | Native usage window |
| `opencode-go-tray.exe --preview` | Open the usage window directly, without the tray |

The window can be closed normally; the tray process keeps running.

## How it works

```mermaid
flowchart LR
    A[OpenCode Desktop] -->|stores active key| B[(opencode.db)]
    B -->|read-only SQLite| C[OpenCode Go Meter]
    C -->|Bearer token| D[opencode.ai<br/>usage API]
    D -->|rolling / weekly / monthly| C
    C --> E[Tray icon + native window]
```

The app reads the active credential from:

```text
%USERPROFILE%\.local\share\opencode\opencode.db
table: credential
where: integration_id = 'opencode-go' AND active = 1
```

It then calls the official endpoint:

```http
GET https://opencode.ai/inference/go/v1/usage
Authorization: Bearer <token>
```

Response fields used:

```json
{
  "usage": {
    "rolling": { "status": "ok", "percent": 30, "resetsAt": "..." },
    "weekly":  { "status": "ok", "percent": 49, "resetsAt": "..." },
    "monthly": { "status": "ok", "percent": 35, "resetsAt": "..." }
  }
}
```

### Credential resolution order

1. `OPENCODE_API_KEY` environment variable
2. `OPENCODE_GO_API_KEY` environment variable
3. OpenCode Desktop's SQLite `credential` table

The legacy `auth.json` file is intentionally **not** used; older OpenCode builds stored a provider key there that the current usage endpoint rejects.

## Build from source

**Requirements**

- Windows 10 or 11
- Rust 1.90+ with the `x86_64-pc-windows-msvc` toolchain
- Visual Studio Build Tools with the MSVC linker

```powershell
cargo build --release
.\target\release\opencode-go-tray.exe
```

Run the test suite:

```powershell
cargo test
```

### Dependencies

| Crate | Purpose |
|---|---|
| `tray-icon` | Windows notification-area icon and context menu |
| `winit` | Event loop required by the tray backend |
| `ureq` + `rustls` | HTTPS usage requests |
| `rusqlite` (bundled) | Read-only access to OpenCode's local credential DB |
| `windows-sys` | Native Win32 window, progress bars, controls, language detection |
| `chrono` | RFC 3339 parsing and local reset times |
| `serde_json` | Usage response parsing |

## Autostart (optional)

Register a logon task so the meter starts with Windows:

```powershell
$exe = (Resolve-Path .\target\release\opencode-go-tray.exe).Path
$action = New-ScheduledTaskAction -Execute $exe
$trigger = New-ScheduledTaskTrigger -AtLogOn
Register-ScheduledTask -TaskName OpenCodeGoMeter -Action $action -Trigger $trigger
```

Remove it with:

```powershell
Unregister-ScheduledTask -TaskName OpenCodeGoMeter -Confirm:$false
```

## Security

- The SQLite database is opened with `SQLITE_OPEN_READ_ONLY`; the app cannot modify OpenCode data.
- The API key is held in memory only. It is never logged, written to disk, or sent anywhere except `opencode.ai`.
- No telemetry, analytics, or background network calls beyond the usage endpoint.
- The app does not read chat content, prompts, or session data; only the `credential` row.

## Project layout

```text
.
├─ assets/
│  └─ preview.png       # screenshot used in this README
├─ src/
│  ├─ i18n.rs           # language table and auto-detection
│  └─ main.rs           # tray, usage API, Win32 window
├─ Cargo.toml
├─ Cargo.lock
├─ run-tray.bat         # build-if-needed launcher
├─ LICENSE
└─ README.md            # bilingual: English + 中文 (this file)
```

## Roadmap

- [ ] Optional installer with autostart
- [ ] High-DPI polish and dark title bar
- [ ] Multi-workspace / multi-account switching
- [ ] Signed release binaries

## License

MIT © 2026 VeloxLLM

---

## 中文

OpenCode Go 有三层互相叠加的用量限制：

| 窗口 | 占月度额度比例 | 重置规则 |
|---|---:|---|
| 5 小时滚动窗口 | 20% | 首次使用后 5 小时 |
| 每周窗口 | 50% | 每周一 00:00 UTC |
| 月度额度 | 100% | 订阅周期开始时 |

每次都去 Console 查看会打断专注。**OpenCode Go Meter** 常驻 Windows 通知区域，用量一瞥即知。

> **原生 Win32 + Rust。** 没有 Electron、WebView，也没有 HTML。

### 多语言支持

界面语言自动跟随 Windows，内置 8 种语言：

English · 简体中文 · 日本語 · 한국어 · Español · Français · Deutsch · Русский

如需强制指定语言：

```powershell
$env:OPENCODE_GO_LANG = "zh"   # en / zh / ja / ko / es / fr / de / ru
```

### 预览

![OpenCode Go Meter 原生用量窗口](assets/preview.png)

托盘图标颜色随 5 小时窗口变化：**绿色** <50%，**橙色** 50–79%，**红色** ≥80%，认证异常时为**灰色**。

### 功能

| | |
|---|---|
| **托盘优先** | 悬停即可查看 5 小时与月度百分比。 |
| **右键菜单** | 5 小时 / 每周 / 月度明细、刷新、打开 Console、退出。 |
| **双击窗口** | 原生 Win32 窗口，带三条进度条、状态与重置时间。 |
| **自动刷新** | 每分钟轮询一次，支持手动刷新。 |
| **多语言界面** | 8 种语言自动检测，可用 `OPENCODE_GO_LANG` 覆盖。 |
| **自动发现凭据** | 从 OpenCode Desktop 本地数据库读取当前有效凭据。 |
| **只读设计** | 以 `mode=ro` 打开 `opencode.db`，绝不写入 OpenCode 数据。 |
| **单文件** | 约 4 MB 发布版可执行文件，SQLite 已打包。 |

### 快速开始

```powershell
git clone https://github.com/VeloxLLM/opencode-go-meter.git
cd opencode-go-meter
.\run-tray.bat
```

> **前提**：先在 OpenCode Desktop 中登录 OpenCode Go。程序读取的是 OpenCode Desktop 本地保存的当前有效凭据。

若托盘图标被隐藏，请点击通知区域的 **^** 展开按钮。

### 使用方式

| 操作 | 效果 |
|---|---|
| 悬停托盘图标 | 显示 5 小时与月度百分比 |
| 右键 | 5 小时 / 每周 / 月度明细、刷新、打开 Console、退出 |
| 双击 | 打开原生用量窗口 |
| `opencode-go-tray.exe --preview` | 直接打开用量窗口，不启动托盘 |

### 安全

- SQLite 数据库以 `SQLITE_OPEN_READ_ONLY` 打开，程序无法修改 OpenCode 数据。
- API Key 仅保存在内存中，不会被写入磁盘或发送到 `opencode.ai` 之外的任何地方。
- 无遥测、无统计上报，除用量接口外无其他后台请求。
- 不读取会话内容、提示词或聊天数据；只读 `credential` 表中的凭据。

### License

MIT © 2026 VeloxLLM
