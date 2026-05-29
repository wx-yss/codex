# 旧分支功能复刻计划

## 原则

- 以当前分支实现和本目录内产品需求文档为准，不 cherry-pick 旧提交。
- 旧提交只作为行为参考；如果旧实现和当前代码结构冲突，按当前结构重做。
- 旧提交 commit id 用于查行为和参考实现，不作为可直接套用的补丁。
- 优先小步复刻，先恢复可见行为，再补充清理和测试。
- 阶段一验证只运行 `cd codex-rs && just fmt` 和 `git diff --check`；模块测试等用户手动验收通过后再执行。
- 一个功能一个功能做，做完一个、用户验收一个、再提交一个。
- 开发执行时采用 `$superpowers:subagent-driven-development` 模式；但该模式内的测试、提交节奏仍以本文件和 AGENTS.md 的 YSS 规则为准。

## 建议顺序

1. 状态栏调整：中偏低
2. 双击 ESC 中断：中
3. spawn_agent 元数据展示：中
4. agent 和 resume 状态标记：中
5. MCP Server 管理弹窗：中
6. 用户 prompt slash 命令：高

## 状态栏调整

对应需求：`docs/yss-requirements/statusline-adjustments.md`

旧提交参考：`84280ddee522001c3e486523e10255ccfcbe1776`

难度：中偏低

### 需要改的

- `codex-rs/tui/src/chatwidget/status_surfaces.rs`
  - `context-remaining` 从 `Context {n}% left` 改成 `{n}%`。
  - 为 `current-dir` 增加紧凑路径显示：
    - 最后一级目录完整保留；
    - 中间目录按同级目录最短唯一前缀缩短；
    - `~` 开头路径保留 `~`；
    - Windows 保持原完整路径；
    - 读取同级目录失败或无法确认唯一前缀时保留原段。
  - 增加 current-dir 显示缓存，避免状态栏频繁刷新时反复扫描文件系统。
- `codex-rs/tui/src/chatwidget.rs`
  - 增加 current-dir 显示缓存字段。
- `codex-rs/tui/src/chatwidget/constructor.rs`
  - 初始化缓存字段。
- `codex-rs/tui/src/chatwidget/session_flow.rs`
  - 会话 cwd 更新时清空缓存。
- `codex-rs/tui/src/chatwidget/settings.rs`
  - thread settings 同步 cwd 时清空缓存。
- `codex-rs/tui/src/bottom_pane/status_surface_preview.rs`
  - 状态栏配置预览中的 `context-remaining` 占位文案改为 `{n}%`。
- `codex-rs/tui/src/chatwidget/tests/status_and_layout.rs`
  - 更新状态栏文案测试。
  - 补充 current-dir 缩短纯函数测试。

### 验收标准

- 状态栏启用 `context-remaining` 时，只显示类似 `100%`，不再显示 `Context 100% left`。
- `/status` 详情卡片里的 Context window 文案不受影响。
- 状态栏启用 `current-dir` 时，`~/Documents/project/remote/codex` 可显示为类似 `~/Doc/p/r/codex`。
- 最后一级目录必须完整显示。
- 无法确认唯一前缀时，该路径段保持完整名称。
- Windows 下仍显示原完整路径。
- cwd 改变后，状态栏路径缓存能刷新，不显示旧目录。

## 双击 ESC 中断

对应需求：`docs/yss-requirements/double-esc-interrupt.md`

旧提交参考：`94055c5e537f0ca5885bafd797b5668c50881aac`

难度：中

### 需要改的

- `codex-rs/tui/src/esc_interrupt_armer.rs`
  - 新增 ESC 双击确认状态机。
  - 确认窗口 200ms。
  - 只有 `KeyEventKind::Press` 可以 arm 或 confirm。
  - `Repeat` 和 `Release` 只能被消费，不能触发中断。
- `codex-rs/tui/src/lib.rs`
  - 注册新模块。
- `codex-rs/tui/src/bottom_pane/mod.rs`
  - 运行中状态行的 ESC 中断路径接入双击确认。
  - 如果用户把中断键重映射为非 ESC，例如 F12，保持单击中断。
  - popup、Vim insert escape、`/agent` 编辑等本地 UI 行为优先级不变。
- `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
  - request-user-input 弹层的 ESC 中断路径接入双击确认。
  - notes UI 展开时，第一次 ESC 仍先收起 notes，不进入中断确认。
  - 非 ESC 中断键继续单击生效。
- `codex-rs/tui/src/chatwidget.rs`
  - 增加 pending steer 路径所需的 ESC armer 字段。
- `codex-rs/tui/src/chatwidget/constructor.rs`
  - 初始化 armer 字段。
- `codex-rs/tui/src/chatwidget/interaction.rs`
  - pending steer 中断路径接入双击 ESC。
- 相关测试：
  - `codex-rs/tui/src/chatwidget/tests/composer_submission.rs`
  - `codex-rs/tui/src/chatwidget/tests/review_mode.rs`
  - `codex-rs/tui/src/chatwidget/tests/slash_commands.rs`
  - `codex-rs/tui/src/bottom_pane/mod.rs`
  - `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`

### 验收标准

- agent 运行中，单次 ESC 不发送 interrupt。
- 200ms 内第二次 ESC 才发送 interrupt。
- ESC 超过 200ms 后再按，只重新进入待确认状态。
- 长按 ESC 产生的 `Repeat` 不会被识别为双击。
- ESC `Release` 不会触发中断。
- pending steer 场景下，也必须双击 ESC 才中断并立即发送排队消息。
- request-user-input 弹层中，双击 ESC 才中断当前 turn。
- request-user-input 的 notes UI 展开时，第一次 ESC 只收起 notes。
- 中断键重映射为 F12 后，F12 单击中断，ESC 不再触发中断。

## spawn_agent 元数据展示

对应需求：`docs/yss-requirements/spawn-agent-metadata.md`

旧提交参考：`f3c58bd9ae74cb0410fae8cffc881ba955a5e3fe`

难度：中

范围口径：只处理当前 v1 `spawn_agent` 调起子代理的链路，在 TUI 中增加子代理参数显示。这里的 v1 指当前实际使用的 `multi_agents` 子代理模式；不复刻、不依赖、不改变 MultiAgentV2。

### 需要改的

- `codex-rs/core/src/tools/handlers/multi_agents/spawn.rs`
  - 解析 v1 `spawn_agent` 原始 JSON 参数，区分 `fork_context` 是省略还是显式传入 `false`。
  - 在 spawn begin/end 事件中携带显式请求的 `agent_type` 和 `fork_context`。
- `codex-rs/protocol/src/protocol.rs`
  - `CollabAgentSpawnBeginEvent` 和 `CollabAgentSpawnEndEvent` 增加可选字段。
- `codex-rs/app-server-protocol/src/protocol/v2/item.rs`
  - `ThreadItem::CollabAgentToolCall` 增加可选 `agent_type`、`fork_context` 字段。
  - 注意：这里的 `protocol/v2` 是 app-server protocol 的版本目录，不代表 MultiAgentV2；本需求仍只处理 v1 `spawn_agent`。
- `codex-rs/app-server-protocol/src/protocol/event_mapping.rs`
  - core event 到 v2 thread item 的映射透传元数据。
- `codex-rs/app-server-protocol/src/protocol/thread_history.rs`
  - 历史重建时保留元数据。
- `codex-rs/tui/src/chatwidget/protocol.rs`
  - app-server thread item 转 TUI item 时透传字段。
- `codex-rs/tui/src/chatwidget/replay.rs`
  - replay 历史时透传字段。
- `codex-rs/tui/src/chatwidget/tool_lifecycle.rs`
  - spawn begin/end 缓存的 `SpawnRequestSummary` 增加字段。
- `codex-rs/tui/src/multi_agents.rs`
  - `SpawnRequestSummary` 增加字段。
  - `Spawned` 历史单元详情中，在 prompt 预览后追加元数据行。
- 相关测试和快照：
  - `codex-rs/tui/src/chatwidget/tests/app_server.rs`
  - `codex-rs/tui/src/app/tests.rs`
  - `codex-rs/tui/src/snapshots/codex_tui__multi_agents__tests__collab_agent_transcript.snap`
  - 必要时更新 app-server / exec 相关事件映射测试。

### 验收标准

- v1 `spawn_agent` 显式传 `agent_type` 时，`Spawned` 详情显示 `agent_type=...`。
- v1 `spawn_agent` 显式传 `fork_context=false` 时，详情显示 `fork_context=false`。
- 请求省略 `fork_context` 时，不合成展示 `fork_context=false`。
- 请求省略 `agent_type` 时，不展示 `agent_type`。
- 标题仍显示 agent label、实际 model、reasoning effort。
- prompt 预览仍保留在详情中，元数据行在 prompt 预览之后。
- v2 `spawn_agent` 不作为本次目标，不因本改动改变 v2 行为。
- MultiAgentV2 相关命名、参数和行为不纳入本次验收。

## agent 和 resume 状态标记

对应需求：`docs/yss-requirements/tui-agent-resume-status-markers.md`

旧提交参考：`9e78fddc41f0dad837d3775c10e5b281267d0a78`

难度：中

### 需要改的

- `codex-rs/tui/src/multi_agents.rs`
  - 新增 `AgentPickerStatus`，表示 `running`、`completed`、`closed`。
  - `AgentPickerThreadEntry` 从 `is_closed: bool` 调整为状态枚举。
  - 保留现有状态点显示，新增可见状态描述。
- `codex-rs/tui/src/app/agent_navigation.rs`
  - `upsert`、`mark_closed` 和相关测试改用状态枚举。
- `codex-rs/tui/src/app/session_lifecycle.rs`
  - 打开 `/agent` 前刷新 thread liveness 并推导状态。
  - `/agent` 行 description 优先显示生命周期状态。
  - 当前行继续由通用选择器显示 `(current)`，description 不重复显示 current。
- `codex-rs/tui/src/app/thread_routing.rs`
  - 新增或调整 resume picker 当前 thread 判定，优先使用 `chat_widget.thread_id()`，必要时再回退 `active_thread_id`。
- `codex-rs/tui/src/app/event_dispatch.rs`
  - 从已有会话打开 `/resume` 时传入当前正在查看的 thread id。
  - 启动阶段打开 resume picker 时不传当前 thread id。
- `codex-rs/tui/src/resume_picker.rs`
  - picker state 增加 `current_thread_id`。
  - comfortable 列表在元信息行显示 `current`。
  - dense 列表增加固定状态列，当前会话显示 `current`，其他会话留空。
- 相关测试：
  - `codex-rs/tui/src/app/tests.rs`
  - `codex-rs/tui/src/app/tests/startup.rs`
  - `codex-rs/tui/src/resume_picker.rs`

### 验收标准

- `/agent` 中仍有未完成 turn 的代理显示 `running`。
- 最近 turn 已结束且线程仍可查看的代理显示 `completed`。
- 线程已关闭或不可继续的代理显示 `closed`。
- `/agent` 当前行由 `(current)` 标记，不在 description 中重复显示 current。
- `/agent` 原有状态点和搜索能力保留。
- `/agent` 搜索仍能通过 thread id 命中。
- 从已有 TUI 会话打开 `/resume`，当前正在查看的会话显示 `current`。
- 启动阶段 resume picker 不显示 current。
- comfortable resume 列表的 current 位于会话元信息行。
- dense resume 列表的 current 位于固定状态列，不挤压标题对齐。

## MCP Server 管理弹窗

对应需求：`docs/yss-requirements/tui-mcp-server-management.md`

旧提交参考：`0df0c6aab1d2912b325060c1f75888ec8f89a81c`

难度：中

范围口径：第一轮接受旧的 MCP inventory history cell 代码暂留，只要求 `/mcp` 新入口不再触发旧的 S 区清单输出。旧代码清理可以在功能稳定后单独处理。

### 需要改的

- `codex-rs/tui/src/chatwidget/mcp_management.rs`
  - 新增 MCP server 管理弹窗模块。
  - 合并 `config.mcp_servers` 和 app-server `mcpServerStatus/list` 状态。
  - 以配置中的 server 为准生成列表，status 只补充工具数和 Auth 状态。
- `codex-rs/tui/src/chatwidget.rs`
  - 增加最近一次 MCP status 缓存字段，用于刷新弹窗。
- `codex-rs/tui/src/chatwidget/constructor.rs`
  - 初始化缓存字段。
- `codex-rs/tui/src/chatwidget/slash_dispatch.rs`
  - `/mcp` 改为打开管理弹窗。
  - `/mcp verbose` 不再支持，统一提示 `Usage: /mcp`。
- `codex-rs/tui/src/slash_command.rs`
  - 更新 `/mcp` 描述，改为管理 MCP server。
- `codex-rs/tui/src/app_event.rs`
  - 新增管理弹窗加载和启停写入事件。
- `codex-rs/tui/src/app/background_requests.rs`
  - 新增 MCP 管理状态加载请求。
  - 新增 `enabled` 配置写入请求。
  - 写入只更新 `mcp_servers.<server>.enabled` 叶子字段。
  - 写入成功后调用 `config/mcpServer/reload`。
- `codex-rs/tui/src/app/event_dispatch.rs`
  - 处理加载完成、写入完成、连续切换排队。
- `codex-rs/tui/src/app.rs`
  - 增加 pending MCP server enabled 写入队列字段。
- `codex-rs/tui/src/app/test_support.rs`
  - 补齐测试构造初始化。
- 相关测试：
  - `codex-rs/tui/src/chatwidget/tests/slash_commands.rs`
  - `codex-rs/tui/src/app/tests.rs`

### 验收标准

- 输入 `/mcp` 时打开 C 区 MCP server 管理弹窗，不在 S 区输出 MCP 清单。
- 允许旧 S 区 MCP 清单相关代码暂留，但 `/mcp` 入口不得再走旧清单路径。
- 弹窗列出 `config.mcp_servers` 中全部 server，包括已禁用 server。
- 列表支持搜索。
- 每行显示启用状态、server 名称和工具数量。
- 选中行显示 Auth 状态和“下一轮生效”提示。
- 空格切换当前选中 server 的启用状态。
- 启停只修改对应 server 的 `enabled` 字段，不覆盖 command、args、cwd、env、headers 等其他配置。
- 写入成功后排队刷新 MCP 配置，让下一轮使用新的启用状态。
- 当前连接不立即重启、不主动断开、不立即重连。
- `/mcp verbose` 显示 `Usage: /mcp`，不再输出 verbose 清单。
- 连续切换同一个 server 时，最终配置以最后一次选择为准。

## 用户 prompt slash 命令

对应需求：`docs/yss-requirements/user-prompts.md`

旧提交参考：`1467bc39d916d967b85615adf7736d4dd005fcab`

难度：高

### 需要改的

- `codex-rs/tui/src/user_prompts.rs`
  - 新增用户 prompt 加载模块。
  - 从 `$CODEX_HOME/prompts/*.md` 读取 prompt metadata。
  - 支持 desktop prompt frontmatter：
    - `description` 用于 slash 弹窗描述；
    - `argument-hint` 接受但暂不使用；
    - 正文为 frontmatter 后的内容。
  - 校验 prompt 文件名，避免路径穿越。
  - 提供按名称加载 prompt 正文的接口。
- `codex-rs/tui/src/lib.rs`
  - 注册新模块。
- `codex-rs/tui/src/bottom_pane/slash_commands.rs`
  - `SlashCommandItem` 增加 `UserPrompt` 分支。
  - `commands_for_input`、`find_slash_command`、`has_slash_command_prefix` 接收 prompt metadata。
  - `/prompt:<name>` 和内置命令、service tier 命令共享候选匹配链路。
- `codex-rs/tui/src/bottom_pane/command_popup.rs`
  - `CommandItem` 增加 `UserPrompt` 分支。
  - 弹窗显示 `/prompt:<name>` 和 description。
  - 支持大小写不敏感前缀匹配。
- `codex-rs/tui/src/bottom_pane/chat_composer.rs`
  - composer 初始化时接收并持有一次性加载的 prompt metadata。
  - slash popup 选择 prompt 时只插入 `/prompt:<name> `，不立即提交。
  - 历史记录和 atomic slash element 与现有 slash 命令保持一致。
- `codex-rs/tui/src/bottom_pane/chat_composer/slash_input.rs`
  - 按当前新结构接入 prompt 类型。
  - bare command、inline command、queued parse 都能识别 prompt。
- `codex-rs/tui/src/chatwidget.rs`
  - 增加用户 prompt metadata 字段。
- `codex-rs/tui/src/chatwidget/constructor.rs`
  - 初始化时从 `$CODEX_HOME/prompts` 加载一次 prompt metadata。
- `codex-rs/tui/src/chatwidget/slash_dispatch.rs`
  - live 提交 `/prompt:<name>` 时加载 prompt 正文并作为用户消息发送。
  - live 提交 `/prompt:<name> <args>` 时发送“prompt 正文 + 空行 + 参数”。
  - queued slash 解析同样支持 prompt。
- `codex-rs/tui/src/chatwidget/user_messages.rs`
  - 暴露文本拼接时 rebasing text elements 的 helper，供 prompt 参数拼接复用。
- 相关测试：
  - `codex-rs/tui/src/bottom_pane/command_popup.rs`
  - `codex-rs/tui/src/bottom_pane/chat_composer.rs`
  - `codex-rs/tui/src/bottom_pane/chat_composer/slash_input.rs`
  - `codex-rs/tui/src/chatwidget/tests/slash_commands.rs`

### 验收标准

- TUI 启动时，从 `$CODEX_HOME/prompts` 读取 `<name>.md` 文件。
- 启动后新增、删除、重命名 prompt 文件，不影响当前 TUI 会话。
- `/` 弹窗能显示 `/prompt:<name>`。
- prompt 候选与内置 slash 命令、service tier 命令共享匹配和渲染链路。
- prompt 的 `description` 显示在 slash 弹窗里。
- `argument-hint` 可以存在，但 TUI 暂不展示、不使用。
- 从 slash 弹窗选择 prompt 时，只插入 `/prompt:<name> `，不立即提交。
- 提交 `/prompt:<name>` 时，发送内容为 prompt 正文。
- 提交 `/prompt:<name> <args>` 时，发送内容为 prompt 正文、一个空行、参数内容。
- 提交 `/prompt:<name>` 后跟多行内容时，多行内容作为参数拼接到 prompt 正文后。
- prompt 参数中的 text elements、mention bindings、图片等现有提交能力不被破坏。
- 无效 prompt 名称不会读取 `$CODEX_HOME/prompts` 之外的文件。
- prompt 文件为空时，TUI 给出明确错误，不发送空消息。

## 阶段验收

### 阶段一：AI 编码完成

- 只运行：
  - `cd codex-rs && just fmt`
  - `git diff --check`
- 不运行 `cargo build`、`cargo test`、`just test`、`just fix`。
- 告知用户目标工作区路径，等待用户手动测试。

### 阶段二：用户手动测试通过并明确允许提交

- 运行本次变更对应模块测试：
  - `cd codex-rs && env RUST_MIN_STACK=16777216 cargo test -p codex-tui --lib`
- 如果涉及 core/protocol/app-server 协议链路，再补充对应 crate 的 focused 测试。
- 提交时只 `git add` 本轮需求相关文件，禁止 `git add .` 或 `git add -A`。
