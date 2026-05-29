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

## 踩坑记录

- v2 `spawn_agent` 当前不作为本需求复刻目标。测试时发现 v2 工具链虽然能暴露 `fork_turns` 并在父线程显示
  metadata，但子代理首条任务投递行为不稳定：`message` 会走 `InterAgentCommunication` 路径，子代理界面里不像普通
  `UserInput` 一样执行原始 prompt，容易先响应 AGENTS/系统注入内容。因此当前需求按 v1 `spawn_agent` 落地，字段保持
  `agent_type` / `fork_context`。
