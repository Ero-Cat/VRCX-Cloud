<div align="center">

# <img src="images/logo.svg" alt="VRCX-Cloud logo" width="110"> VRCX-Cloud

### 你的 VRChat 社交生活，由你自己的服务器提供。

**自托管、单账号的 VRChat 网页伴侣** —— 在浏览器中获得完整的 VRCX-0
体验；由一个 Rust 二进制驱动，它与桌面版 VRCX-0 并行运行，并通过
PostgreSQL 同步网格保持数据一致。

> 🛠️ 本项目由 **[VRCX-0](https://github.com/Map1en/VRCX-0)**（作者
> [Map1en](https://github.com/Map1en)）魔改而来 —— 底层运行时与同步
> 引擎的功劳全部归于上游。
>
> 🔗 与本项目联动的桌面客户端（客户端数据库同步到远程线上数据库）为
> **[Ero-Cat/vrcx-0](https://github.com/Ero-Cat/vrcx-0)**。

[![CI](https://img.shields.io/github/actions/workflow/Ero-Cat/VRCX-Cloud/ci.yml?branch=master&style=flat-square&label=CI&logo=github)](https://github.com/Ero-Cat/VRCX-Cloud/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Ero-Cat/VRCX-Cloud?style=flat-square&color=blue&label=version)](https://github.com/Ero-Cat/VRCX-Cloud/releases)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/backend-Rust%201-dea584?style=flat-square&logo=rust)](Cargo.toml)
[![Frontend](https://img.shields.io/badge/frontend-React%2019-61dafb?style=flat-square&logo=react)](package.json)
[![PostgreSQL](https://img.shields.io/badge/sync-PostgreSQL-336791?style=flat-square&logo=postgresql)](.env.example)

**[快速开始](#-快速开始) · [功能](#-功能) · [安装](#-安装) · [使用手册](#-使用手册) · [设计理念](#-为什么再造一个-vrcx) · [常见问题](#-常见问题)**

[English](README.md) · **简体中文**

</div>

---

## 📑 目录

- [这是什么？](#-这是什么)
- [✨ 功能](#-功能)
- [🚀 快速开始](#-快速开始)
- [📦 安装](#-安装)
    - [Docker Compose（推荐）](#docker-compose推荐)
    - [裸机](#裸机)
    - [配置参考](#%EF%B8%8F-配置参考)
- [📖 使用手册](#-使用手册)
    - [首次登录](#首次登录)
    - [配对你的桌面版 VRCX-0](#配对你的桌面版-vrcx-0)
    - [桌面感知实时切换](#桌面感知实时切换)
    - [日常使用](#日常使用)
- [🎥 演示](#-演示)
- [🏗 架构](#%EF%B8%8F-架构)
- [🤔 为什么再造一个 VRCX？](#-为什么再造一个-vrcx)
- [❓ 常见问题](#-常见问题)
- [🔒 安全说明](#-安全说明)
- [🤝 致谢](#-致谢)

---

## 🌥 这是什么？

```
 ┌──────────┐  HTTP/WS   ┌────────────────┐            ┌────────────┐
 │ Browser  │───────────▶│ vrcx-0-server  │──sync─────▶│ PostgreSQL │
 │ (anywhere│            │  · VRChat live │◀──sync─────│   (yours)  │
 │  on LAN) │            │  · full data   │            └─────▲──────┘
 └──────────┘            └────────────────┘                  │ sync
                                        ┌───────────────────┴─────┐
                                        │  Desktop VRCX-0 (games, │
                                        │  logs, screenshots)     │
                                        └─────────────────────────┘
```

VRCX-Cloud 将**完整的 VRCX 运行时作为服务器**运行：它维持自己的
VRChat 实时会话，提供整个 VRCX-0 Web UI，并通过内置的
SQLite↔PostgreSQL 协议与桌面版 VRCX-0 同步每一行数据。PC 上记录的
游戏日志会出现在网页上；你在网页上写的备忘录也会落到桌面端 —— 一个
同步周期内完成。

## ✨ 功能

|     | 功能                         | 说明                                                                                                    |
| --- | ---------------------------- | ------------------------------------------------------------------------------------------------------- |
| 🖥   | **浏览器中的完整 VRCX-0 UI** | 动态（feed）、好友、游戏日志历史、收藏、通知、统计图表、AI 助手 —— 同一个 React 应用，由服务器提供      |
| ⚡  | **服务端 VRChat 会话**       | 在浏览器登录一次（含 2FA）；服务器维持实时 websocket，桌面端关机时网页依然在线                          |
| 🔁  | **双向同步网格**             | op-log 协议 + HLC 冲突仲裁、幂等推送、重启后续传 —— 与桌面版 VRCX-0 共享                                |
| 🤝  | **桌面感知切换**             | 桌面版 VRCX-0 活跃同步时，服务器暂停自己的 VRChat 会话让桌面端采集 —— 不会产生双倍 API 流量、双倍动态行 |
| 🧠  | **344 条命令 API**           | 与桌面端完全一致的命令面，经 `POST /api/invoke` + WebSocket 事件提供                                    |
| 🔐  | **单密码网页认证**           | cookie 会话、登录限流、显式局域网开放模式                                                               |
| 🐳  | **单个容器**                 | docker compose 启动 server + PostgreSQL；裸机部署为单个二进制                                           |
| 🧩  | **优雅降级**                 | 桌面专属功能（启动游戏、VR overlay、托盘）自动报告为不支持并隐藏                                        |

## 🚀 快速开始

```bash
git clone https://github.com/Ero-Cat/VRCX-Cloud.git
cd VRCX-Cloud

cp .env.example .env      # 将 VRCX_SYNC_* 指向你的远程 PostgreSQL
docker compose up -d --build

open http://localhost:8800   # 默认免登录（可信局域网）
```

你会看到登录门 → 登录后是 VRChat 登录页。输入你的 VRChat 凭据（支持
2FA），服务器接管会话。就这么简单 —— 随时可以去
[配对桌面端](#配对你的桌面版-vrcx-0)。

## 📦 安装

### Docker Compose（推荐）

```bash
cp .env.example .env      # 设置 VRCX_WEB_PASSWORD（必填）和 VRCX_PG_PASSWORD
docker compose up -d --build
```

启动的服务：

| 服务       | 地址                             | 用途                  |
| ---------- | -------------------------------- | --------------------- |
| `app`      | `http://<host>:8800`             | VRCX-Cloud Web 服务器 |
| `postgres` | `127.0.0.1:5432`（仅本机可访问） | 同步数据库            |

### 裸机

环境要求：Rust 1.85+、Node 24、一台可达的 PostgreSQL。

```bash
npm ci && npm run build                    # 前端 -> dist/
cargo build --release -p vrcx-0-server

VRCX_CLOUD_DATA_DIR=/var/lib/vrcx-cloud \
VRCX_CLOUD_WEB_PASSWORD=... \
VRCX_CLOUD_SYNC_HOST=127.0.0.1 VRCX_CLOUD_SYNC_USER=vrcx \
VRCX_CLOUD_SYNC_PASSWORD=... VRCX_CLOUD_SYNC_DATABASE=vrcx \
./target/release/vrcx-0-server
```

### ⚙️ 配置参考

环境变量（或 `server.toml`，路径由 `VRCX_CLOUD_CONFIG` 指定）：

| 环境变量                                           | TOML                      | 默认值                   | 含义                                                   |
| -------------------------------------------------- | ------------------------- | ------------------------ | ------------------------------------------------------ |
| `VRCX_CLOUD_DATA_DIR`                              | `[server] data_dir`       | `<config>/VRCX-0-Server` | SQLite 配置与图片缓存                                  |
| `VRCX_CLOUD_LISTEN`                                | `[server] listen_addr`    | `0.0.0.0:8800`           | HTTP 监听地址                                          |
| `VRCX_CLOUD_WEB_PASSWORD`                          | `[web] password`          | —（开放）                | 网页登录密码                                           |
| `VRCX_CLOUD_WEB_AUTH_DISABLED`                     | `[web] auth_disabled`     | `false`                  | 关闭认证（可信局域网）                                 |
| `VRCX_CLOUD_DIST_DIR`                              | `[web] dist_dir`          | `./dist`                 | 前端静态文件目录                                       |
| `VRCX_CLOUD_SYNC_HOST/PORT/USER/PASSWORD/DATABASE` | `[sync] …`                | —                        | 远程同步 DSN，启动时注入                               |
| `VRCX_CLOUD_SYNC_INTERVAL_SEC`                     | `[sync] interval_sec`     | `60`                     | 同步周期（5–3600 秒）                                  |
| `VRCX_CLOUD_SYNC_ALLOW_PLAINTEXT`                  | `[sync] allow_plaintext`  | `false`                  | 允许无加密 PG（局域网）                                |
| `VRCX_CLOUD_REALTIME_MODE`                         | `[realtime] mode`         | `auto`                   | `auto`：桌面端活跃时暂停服务端会话；`always`：始终开启 |
| `VRCX_CLOUD_FEED_LOGGING`                          | `[realtime] feed_logging` | `true`                   | `false`：服务端永不记录动态（由桌面端记录）            |

## 📖 使用手册

### 首次登录

1. 打开 `http://<server>:8800`（未设置网页密码则无需登录）。
2. 在 VRChat 登录页用你的账号登录（支持 TOTP / 邮件 OTP）。服务器会将
   会话加密存储。
3. 应用打开后进入动态页。仅存在于桌面端的数据会在
   [配对](#配对你的桌面版-vrcx-0)后到达。

### 配对你的桌面版 VRCX-0

与本项目的桌面版本为 **[Ero-Cat/vrcx-0](https://github.com/Ero-Cat/vrcx-0)**
—— 一个内置相同同步引擎的 VRCX-0 分支，其本地数据库会与本服务端共用
的远程 PostgreSQL 收敛：

1. 桌面版 VRCX-0 → **设置 → 数据同步（Data Sync）**。
2. 输入你的服务器所用的 PostgreSQL 连接：主机 `<server>`、端口
   `5432`、用户/密码/数据库来自你的 `.env`。在没有 TLS 的可信局域网
   勾选 _允许无加密连接_。
3. 点击 **测试连接**，然后启用。首次同步会做完整引导；之后每个周期
   双向收敛。

> 顺序无关：服务端先启动或桌面端先启动都会收敛。

### 桌面感知实时切换

在 `[realtime] mode = auto`（默认）下，服务器会监视同步设备活动：

- **桌面端活跃**（约 90 秒内有推/拉）→ 服务器关闭自己的 VRChat
  websocket 并闲置。零重复会话、零重复动态；网页靠同步数据继续工作。
- **桌面端安静** → 服务器重连自己的会话，网页恢复完全实时（比如桌面机
  夜间关机）。

若希望服务端会话始终开启，可设 `mode = "always"`；若桌面端应永远是
记录者，可设 `feed_logging = false`。

### 日常使用

- **动态 / 好友 / 历史** —— 服务端会话开启时为实时；否则为同步延迟
  （≤ 你的同步周期）。
- **游戏日志**（加入/离开、视频播放、进图玩家列表）由桌面端从 VRChat
  的日志文件记录，并出现在网页上。
- **收藏、邀请、通知** —— 网页通过服务端会话直接调用 VRChat API。
- **数据同步面板**（设置 → 同步）显示每台设备、其最近推送/拉取时间，
  以及 _立即同步_ 按钮 —— 网页与桌面端都有。

## 🎥 演示

首次启动全流程（登录门 → VRChat 登录 → 动态页）：

```
$ docker compose up -d --build
 ✔ Container vrcx-cloud-postgres-1  Healthy
 ✔ Container vrcx-cloud-app-1        Started

$ curl -s localhost:8800/healthz
{"ok":true,"phase":"Running","authStatus":"Authenticated",...}

浏览器 → http://localhost:8800
 ├─ 输入网页密码登录
 ├─ VRChat 登录（2FA）… 完成 —— 重启后会话自动恢复
 ├─ 设置 → 数据同步 → 桌面设备 "PC-Win11" 最近拉取：12 秒前
 └─ 桌面端进图 → 15 秒后网页动态显示
```

> 真实录屏将随首个 tag 版本发布。

## 🏗 架构

| 组成部分                                          | 是什么                                                                                                                            |
| ------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `crates/server`                                   | axum 二进制：认证、`POST /api/invoke` 分发器（344 条命令）、`/api/events` WebSocket、`/api/img` 缓存、SPA 托管、实时门 supervisor |
| `crates/runtime-host-server`                      | 组合根之上的服务器运行时门面（`RuntimeHostProfile::Server`）                                                                      |
| `crates/application-sync`                         | 与桌面版 VRCX-0 共享的 op-log 同步引擎                                                                                            |
| `crates/*`（application、realtime、persistence…） | VRCX 领域核心，与 VRCX-0 保持一致                                                                                                 |
| `src/`                                            | VRCX-0 React 应用；`webTransport.ts` 将 invoke/事件切换为浏览器内的 HTTP/WS                                                       |

桌面专属子系统（游戏日志监视、VR overlay、托盘、更新器、注册表备份、
TTS）是被移除而非禁用 —— 服务器代码树中不含桌面死代码。

## 🧹 磁盘清理

Rust 的 `target/` 增长很快（本工作区开发高峰期的 debug 产物达到过
25 GB）。注意控制：

```bash
npm run rust:clean:debug        # 清掉 target/debug（收益最大，随时可做）
scripts/cargo-target-hygiene.sh # 增量缓存 + cargo-sweep 清理 30 天以上
npm run rust:clean              # 完整 cargo clean
```

工作区已使用 `debug = "line-tables-only"` 编译并跳过依赖的
debuginfo；如果你在多个 Rust 仓库之间工作，可以把 `CARGO_TARGET_DIR`
指向同一个共享目录，避免各项目重复占用磁盘。

## 🤔 为什么再造一个 VRCX？

- **VRCX 和 VRCX-0 都是桌面应用。** 电脑关机 —— 或你只是想用手机看看
  好友在哪个世界 —— 此前无解。VRCX-Cloud 让 _服务器_ 成为常开设备。
- **一个账号，多块屏幕。** 这刻意 _不是_ 多租户 SaaS：你的数据、你的
  机器、你的局域网（或 tailnet）。没有账号服务、没有分析埋点、设计上
  不对外暴露。
- **不与桌面端为敌 —— 与之结盟。** VRCX-Cloud 没有把 VRCX 的存储移植
  到 PostgreSQL，而是复用 VRCX-0 自己的同步引擎，桌面端与服务器永远是
  同一网格中的对等节点，更重的桌面专属职责（游戏日志、截图）留在
  原处。
- **全栈 Rust。** 整个后端就是久经考验的 VRCX-0 代码库（约 25 万行）
  加一个新的 axum 外壳 —— 不是重写。

## ❓ 常见问题

**电脑关机时网页还能用吗？**
能 —— 这正是它的意义。服务器维持自己的 VRChat 会话
（`realtime mode = auto` 在桌面端活跃时让位，其余时间接管）。

**两个会话会让我的 VRChat 账号被标记吗？**
有可能 —— 任何自动化都有风险。保持 `[realtime] mode = auto`，游玩时
让服务器闲置；若服务器 IP 与你常用 IP 差异很大，考虑出站代理。新 IP
首次登录可能遇到验证码（无自动求解）。

**我的数据放在哪？**
服务器数据目录（SQLite + 图片缓存，VRChat 会话加密存储）和你的
PostgreSQL。数据不会离开你的网络；遥测已被编译移除。

**多个 VRChat 账号？**
一台服务器 = 一个账号，设计如此。第二个账号再跑一套 compose。

**能暴露到公网吗？**
在 VPN（Tailscale/WireGuard）之后可以。直接暴露？请别 —— 网页层背后
是你完整的 VRChat 会话。

**如何升级？**
`git pull && docker compose up -d --build`。同步协议有版本守卫；桌面端
与服务端版本可以短暂不一致。

## 🔒 安全说明

- 网页认证：单密码、恒定时间比较、会话 cookie（`HttpOnly`、
  `SameSite=Lax`）、登录限流。局域网定位；需要时经反向代理加 TLS。
- PostgreSQL：每次部署独立凭据，可信局域网可选 TLS，compose 文件不会
  公开暴露。
- VRChat 会话 cookie：以机器派生密钥静态加密。
- 保持服务器与桌面端 NTP 时间同步 —— 同步仲裁使用混合逻辑时钟。

## 🤝 致谢

- **[VRCX-0](https://github.com/Map1en/VRCX-0)**（作者 Map1en）——
  本项目魔改自它的 Rust 重写版，双方共享同一套同步引擎。
- **[Ero-Cat/vrcx-0](https://github.com/Ero-Cat/vrcx-0)** —— 与本
  服务端在同步网格上配对的桌面客户端（桌面数据库 ⇄ 远程
  PostgreSQL）。
- **[VRCX](https://github.com/vrcx-team/VRCX)** —— 原版 Electron 伴侣
  应用及其社区。
- VRChat 是 VRChat Inc. 的商标。本项目与 VRChat Inc. 无隶属或背书
  关系。

---

<div align="center">

**[⬆ 回到顶部](#-目录)**

GPL-3.0 · 魔改自 VRCX-0 ❤️

</div>
