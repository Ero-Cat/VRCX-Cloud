# VRCX-jirai 核心功能融入 VRCX-Cloud 实施计划

## 背景与结论

VRCX-jirai（原版 VRCX 的地雷系魔改分支，已停更，MIT 许可）代码栈为 Electron/.NET/Vue，与本项目（Rust axum + React）零兼容，**只能移植功能设计，不能移植代码**。评估结论：7 个核心功能中 6 个可融入，1 个（多账号）不建议做。用户已选定全部 4 个方向：分析三件套、关系网增强、非好友追踪、交流密度时间轴。

本项目的相对优势：`feed_gps/feed_status/feed_bio/feed_online_offline` 由 realtime WS 事件流落库且全部在同步目录（GSet），分析功能天然跨设备可用——这是 jirai 做不到的。

## 总体原则

- **复用三套现成模板**：①activity 聚合（`crates/persistence/src/activity/`，区间分段+派生视图）；②feed 查询（`crates/persistence/src/feed/query.rs`，按 user_id 过滤+keyset 分页）；③profile_bio 扫描器三件套（`application/src/social/profile_bio/` + `social_maintenance.rs` + `composition/src/state/background_ticks/`，轮询+diff+写 feed）。
- **新表自动进同步网格**：`persistence/src/sync.rs` 的 `refresh_capture` 会自动为目录外新表装 trigger（默认 LWW 语义）；只有需要 GSet 等非默认语义的表才往 `SYNC_TABLE_CATALOG`（`crates/contracts/src/sync.rs`）加条目。
- **新命令链路固定六处**：`contracts/src/<feature>.rs`（类型）→ `persistence/src/<feature>/`（查询）→ `runtime-host-server/src/local_data.rs`（方法）→ `server/src/commands/`（注册）→ `src/platform/native/bindings.ts`（**手工维护**，无 codegen）→ 可选 `src/repositories/`。
- 派生分析 v1 做成**按需即时计算**（单用户事件量小），不做缓存表；activity 那种全量缓存模式留作性能不足时的后手。

## Phase 1：分析三件套（纯增量，数据全就绪）

### 1a. 简介 Diff 视图（最小，先做）

- **后端**：`crates/persistence/src/feed/` 下新增 bio 历史查询（`feed_bio` 已同时存 `bio` + `previous_bio`）。新命令 `app__feed_bio_history {userId, targetUserId, dateFrom, dateTo, limit}`。24h 内多次修改的合并逻辑放查询侧：窗口内取最早 `previous_bio` 为基准、最新 `bio` 为结果，输出 `{createdAt, bio, previousBio, displayName}` 行。
- **前端**：`src/components/dialogs/user-dialog/userDialogViewData.ts` 的 `buildUserDialogTabs()` 加 `bio` tab → `UserDialogTabsSection.tsx` 加内容 → `useUserDialogTabData.ts` 加懒加载。Diff 渲染为行级 LCS 红删绿增组件（自写小工具函数放 feature model 文件，配 `.test.ts`，不加依赖）。
- i18n：`src/localization/en.json` + `zh.json` 加 key。

### 1b. 灯色分布图

- **后端**：新模块 `crates/persistence/src/status_stats/`（照 activity 的 mod/types/view 结构）。算法：读目标用户的 `{p}_feed_status`（状态变更）+ `{p}_feed_online_offline`（在线区间），相邻事件差分求各状态（active/joinme/busy/askme）时长，排除离线区间；输出总量占比 + 按天分桶序列。命令 `app__status_stats_view {ownerUserId, targetUserId, rangeDays}`，走 `spawn_blocking`。
- **前端**：user dialog 加 `status` tab，echarts 水平堆叠条（占比）+ 按天堆叠柱（`BarChart` 已注册，无需改 `lib/echarts.ts`）。自己视角的数据源（`self_profile_log` field=status）作为可选补充，v1 以好友视角为主。

### 1c. 共同实例查询（含"双向奔赴"）

- **后端**：新模块 `crates/persistence/src/mutual_instances/`。算法：
    1. 对两人各建"位置区间流"：`feed_online_offline`（Online 事件自带 location）+ `feed_gps`（每次变更开新段，直至下个变更/离线事件），沿用 `activity/view.rs` 的 15 分钟间隔成段模式；
    2. location 字符串相等（可切换按世界级匹配：剥掉 `:instanceId` 后缀）的区间求交；
    3. 每个共同段输出：世界名、起止时间、**双向奔赴标记**（两人入段起点差 ≤3 分钟）、**自己是否在场**（owner 自己的 gps/gamelog_location 区间相交）、**峰值好友数**（owner 全部好友在同一实例有重叠段的最大计数，近似值）。
    - 命令 `app__mutual_instances_query {ownerUserId, userIdA, userIdB, rangeDays, worldLevelMatch, includeSelf}`。
- **前端**：新对话框组件（user dialog 加"共同实例"入口 action + `src/components/search/FriendMultiSelectList.tsx` 限选 2 人），结果表格 + 双向奔赴 badge；参考 `useGameLogPreviousInstancesDialog.ts` 的一次性对话框模式。

## Phase 2：关系网增强（手动连线 + 非好友节点）

- **新表**（DDL 加进 `crates/persistence/src/database/schema.rs` 的 `ensure_user_store_statements`）：
    - `{p}_mutual_graph_manual_links (friend_id, mutual_id, note, created_at, PRIMARY KEY(friend_id, mutual_id))` → `SYNC_TABLE_CATALOG` 加 **GSet** 条目（追加型，避免被覆盖）；
    - `{p}_mutual_graph_external_users (user_id PRIMARY KEY, display_name, avatar_url, added_at)` → 加 **Lww** 条目。
    - 关键：现有 `mutual_graph_snapshot_commit`（`persistence/src/mutual_graph.rs`）是全量替换式刷新，但只动 API 派生三表，手动数据放独立表天然不被冲掉。
- **命令**：`app__mutual_graph_manual_link_add/remove`、`app__mutual_graph_external_user_add/remove`；扩展 `app__mutual_graph_snapshot_get` 输出附带 `manualLinks` + `externalUsers`（前端单次拉取）。
- **前端**：`src/lib/mutual-friends/` 扩展——`mutualFriendsTypes.ts` 加 ManualLink/ExternalNode 类型；`buildMutualFriendsBaseGraph` 合并手动边与外部节点（`ensureNode` 目前丢弃 snapshot 外节点，需放开）；`mutualFriendsSigmaGraph.ts` 给手动边加虚线/独立色样式、外部节点用空心样式（参考 `NodeHollowProgram`）；UI 入口：节点右键"连线到…"选择器 + SettingsSheet"按 UID 添加外部用户"；改完 bump `src/state/mutualGraphRevisionStore.ts`。

## Phase 3：非好友追踪（默认关闭，带隐私说明）

- **新表**：`{p}_watched_users (user_id PRIMARY KEY, display_name, added_at, last_polled_at, last_status, last_bio)`，Lww 进同步目录（关注列表跨设备同步；产生的 feed 事件本身就在 feed_* GSet 表里自动同步）。
- **轮询器**（照抄 profile_bio 三件套）：
    1. 纯逻辑 `crates/application/src/social/profile_watch/mod.rs`：`scan_next_profile_watch(deps, now)`——取最陈旧的 watched 用户 → `profile_get_input`（`vrchat-client/src/users.rs:29`，非好友可用）→ diff last_status/last_bio → 发 `FeedLiveEntry::Status/Bio`；429/5xx 退避复用 `ProfileBioScanPacer` 模式；轮询间隔常量（如 5 分钟量级、逐用户错峰）。
    2. 调度 `social_maintenance.rs` 的 `SocialMaintenanceSchedule` 加 `next_profile_watch` 字段 + plan 分支。
    3. 执行体 `composition/src/state/background_ticks/profile_watch.rs`。
    4. 开关：configs 表 key `profileWatchEnabled`，**默认 off**（与 `profileBioScanEnabled` 同模式，前端可切换）。
- **结构性缺口修复**：`publish_friend_feed_entry`（`application-realtime/.../host/friend_feed_entry.rs`）绑定好友 roster（generation 校验），非好友会被拒。需在 `RealtimeHostRuntime` 加 `publish_external_feed_entry` 变体：跳过 roster 匹配、直接构造 `RealtimeFriendOutput` 走同一条 `apply_friend_output_owned` → `write_realtime_batch` → `emit_feed_entries` 链（前端 feed 零改动自动水合）。
- **前端**：关注列表管理 UI（feed 页设置区或独立设置面板：按 UID/用户名添加、列表、移除）；被关注用户的简介 Diff / 灯色分布 / 状态历史 tab 直接复用 Phase 1（查询按 user_id 键控，不校验好友关系）。
- **文档**：README 隐私一节注明此功能默认关闭、数据来自公开 API、对方黄灯/红灯时 status 不可见（jirai 作者亦承认此失效条件）。

## Phase 4：交流密度时间轴（依赖 Phase 1c 的区间机械）

- **后端**：新模块复用 mutual_instances 的区间求交机械，命令 `app__social_density_view {ownerUserId, targetUserId, rangeDays, bucketBy}`：按天分桶输出（a）同时在线时长（两人 `feed_online_offline` presence 区间交）与（b）共同在房时长（与 owner 自己位置区间交，owner 自身位置来自桌面同步的 `gamelog_location`）。**明确边界**：纯服务器部署（无桌面同步）只有 (a) 没有 (b)。
- **前端**：charts 下新子页"交流密度"（`src/features/charts/social-density/`，照 mutual-friends 的 hooks/components 组织）；`src/lib/echarts.ts` 需注册 `LineChart`；user dialog 加入口。路由/导航三处注册：`src/app/routes.tsx`、`src/shared/constants/ui.ts` navDefinitions、`navMenuModel.ts` routePathByName。

## 横切事项

- **bindings.ts 手工维护**：每个新命令都要在 `src/platform/native/bindings.ts` 手写 TS 类型 + generatedCommands 方法（camelCase 方法 → `app__snake_case` 命令名）。
- **i18n**：en + zh 双语 key（其余语言文件按现有策略补齐或回退）。
- **测试**：Rust 单测跟随仓库模式（模块内 `tests` 子模块，如 `mutual_graph/tests`）；前端 model 纯函数配同目录 `.test.ts`。
- **实施顺序**：Phase 1a → 1b → 1c → 2 → 3 → 4（从纯读查询到 schema 到后台子系统，风险递增；1c 的区间机械是 4 的前置）。

## 明确不做

- **多账号共同登录**：认证/realtime 会话/同步网格（`_sync_devices`）/UI 全部假设单账号，属颠覆式重构，与"自托管单账号服务"定位冲突。
- **自动跟随**：jirai 自己也未实现且不再计划。
