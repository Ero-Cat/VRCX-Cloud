# VRCX-Cloud

Self-hosted **web** companion for VRChat — a single-binary Rust server that
runs a complete VRCX runtime (VRChat realtime session, feed history,
favorites, notifications, stats) and serves the full VRCX-0 web UI from
your own machine. Built for **single-account, LAN-first private
deployment**.

```
浏览器 ──HTTP/WS──▶ vrcx-0-server ──直连──▶ PostgreSQL ◀──双向同步── 桌面端 VRCX-0
```

桌面端 [VRCX-0](https://github.com/Map1en/VRCX-0) 通过其内置的
SQLite↔PostgreSQL 数据同步与服务器收敛到同一个数据库——服务器就是同步
网格里的第二台设备：桌面端产生的游戏日志/实时 feed 经同步到达服务端，
服务端自有的 VRChat 会话提供真·实时的好友状态与 API 操作能力。

## 特性

- **完整 Web UI**：VRCX-0 前端原样运行（feed、好友、收藏、通知、
  游戏日志、统计图谱、AI 助手），桌面专属功能经主机能力门控自动隐藏。
- **服务端 VRChat 会话**：在网页里登录（含 2FA）后，服务端保持自己的
  实时 WebSocket 与 API 直连；桌面端离线时 Web 依然实时。
- **数据同步**：复用 VRCX-0 的 op-log 双向同步协议（HLC 仲裁、幂等、
  断线续传），与桌面端互为副本。
- **单密码 Web 认证**：cookie 会话 + 登录限流；可信内网可显式关闭。
- **轻量**：单个二进制 + 可选 docker-compose（含 PostgreSQL）。

## 快速开始

```bash
cp .env.example .env          # 设置 Web 与数据库密码
docker compose up -d --build  # http://<服务器IP>:8800
```

详见[部署指南](docs/DEPLOY.zh-CN.md)（配置项、桌面端同步接入、安全须知）。

## 架构

| 组件                         | 说明                                                                                                                   |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `crates/server`              | axum Web 服务：`/api/invoke` 命令分发、`/api/events` WebSocket 实时事件、`/api/img` 图片缓存、SPA 静态托管、单密码认证 |
| `crates/runtime-host-server` | 服务端运行时门面（composition `RuntimeHostState` + Server profile）                                                    |
| `crates/application-sync`    | SQLite↔PostgreSQL 双向同步引擎（与桌面端共用协议）                                                                     |
| `src/`                       | VRCX-0 React 前端，`src/platform/tauri/webTransport.ts` 在浏览器模式下自动切换 HTTP/WS 传输                            |

命令面与桌面端完全同名（`app__*`，camelCase 参数），桌面专属命令返回
结构化 `unsupportedOnWeb` 供前端优雅降级。

## 开发

```bash
npm ci && npm run build                    # 前端
cargo build -p vrcx-0-server               # 服务端
cargo test --workspace && npm run test     # 测试
```

本地运行：

```bash
VRCX_CLOUD_WEB_PASSWORD=dev VRCX_CLOUD_LISTEN=127.0.0.1:8800 \
VRCX_CLOUD_DIST_DIR=./dist ./target/debug/vrcx-0-server
```

## License

GPL-3.0（继承自 VRCX-0 / VRCX）。
