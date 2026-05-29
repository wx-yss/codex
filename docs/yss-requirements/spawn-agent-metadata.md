# TUI spawn_agent 元数据展示

当 `spawn_agent` 工具调用显式传入了额外请求参数时，Codex TUI 应在 `Spawned` 历史单元的详情里展示这些
元数据。

当前按 v1 `spawn_agent` 处理，展示字段为 `agent_type` 和 `fork_context`。

标题保持紧凑，继续展示被创建 agent 的 label、实际 model 和 reasoning effort。详情行先展示渲染后的 prompt
预览，再展示请求元数据，例如：

```text
agent_type=default fork_context=false
```

元数据详情行只展示请求里真实出现过的参数。例如请求省略了 `fork_context`，只传了 `agent_type` 时，详情行只应展示
`agent_type=...`，不能补出一个合成的 `fork_context=false`。

## 实现记录

- v1 `spawn_agent` 在解析原始 JSON 参数时保留请求里显式出现的 `agent_type` 和 `fork_context`，并通过
  `CollabAgentSpawnBeginEvent` / `CollabAgentSpawnEndEvent` 透传。
- app-server `CollabAgentToolCall` 保留这两个可选字段，实时事件映射和历史重建都不合成默认值。
- TUI 在 `Spawned` 历史单元中继续用实际 model / reasoning effort 渲染标题，并在 prompt 预览之后追加请求元数据行。

## 踩坑记录

- v2 `spawn_agent` 当前不作为本需求复刻目标。测试时发现 v2 工具链虽然能暴露 `fork_turns` 并在父线程显示
  metadata，但子代理首条任务投递行为不稳定：`message` 会走 `InterAgentCommunication` 路径，子代理界面里不像普通
  `UserInput` 一样执行原始 prompt，容易先响应 AGENTS/系统注入内容。因此当前需求按 v1 `spawn_agent` 落地，字段保持
  `agent_type` / `fork_context`。
- `fork_context` 的默认值在工具参数结构里是 `false`，不能直接从反序列化后的参数判断展示内容；必须先看原始 JSON
  是否包含该字段，否则会把省略字段误显示成 `fork_context=false`。
- TUI 会用 begin 阶段缓存补齐 completed item 中缺失的 spawn 请求摘要。合并时不能让缺少 model / reasoning effort
  的 completed item 覆盖 begin 缓存，否则 `Spawned` 标题会丢失模型与 reasoning 信息；请求元数据则按 begin/end
  中显式出现的字段合并。
- 同时也不能固定使用 begin 缓存里的 model / reasoning effort；end 阶段拿到的是子 agent 最终实际配置，存在时应优先用于
  标题。`reasoning_effort` 必须保留 Option，才能区分 end 缺字段和 end 显式给出的默认值。
- TUI tool lifecycle 处理 completed / failed spawn item 时，不能先把 completed 摘要写入 pending 缓存再 remove；
  否则会覆盖 begin 缓存，导致 end 缺少 model / reasoning effort 时无法回退。终态 item 应先 remove begin 缓存，再与当前
  item 的 fallback 摘要合并。
