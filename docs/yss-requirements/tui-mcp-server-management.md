# TUI MCP Server 管理弹窗

## 背景

之前的 `yss-custom` 分支基于 `v0.128.0`，其中 `/mcp` 会打开可搜索、可切换的 MCP server 管理弹窗。`rust-v0.130.0` 中 `/mcp` 改为把 MCP 清单直接渲染到消息滚动区，无法直接启停 server。

## 需求

输入 `/mcp` 时，打开 MCP server 管理弹窗，而不是在消息滚动区输出清单。

期望行为：

- 弹窗列出 `config.mcp_servers` 中配置的全部 server。
- 空格切换当前选中 server 的启用状态。
- 已禁用的 server 仍然必须出现在弹窗里，方便重新启用。
- 列表支持搜索。
- 每行展示启用状态、server 名称和工具数量。
- 选中行展示 Auth 状态和“下一轮生效”的提示。
- 启停只改变对应 server 的 `enabled` 字段；写入时必须保留 command、args、cwd、env 等其他配置。
- 启停写入后不立即重启、断开或重新连接当前 MCP server；实际启停在下一轮生效。
- 启停写入成功后，需要排队刷新 MCP server 配置，让下一轮使用新的启用状态。

## 边界

- 不支持 `/mcp verbose`。
- 不维护 `/mcp` 的第二套消息滚动区清单展示路径。
- 如果以后需要展示更多 MCP 信息，优先扩展这个弹窗，而不是恢复单独的 verbose 输出。

## 实现备注

- 当前分支复刻时直接基于 app-server 的 `mcpServerStatus/list` 获取工具数量与 Auth 状态。
- 启停配置写入只更新 `mcp_servers.<server>.enabled` 叶子字段，避免覆盖 command、args、cwd、env、headers 等配置。
- 写入成功后调用 `config/mcpServer/reload` 排队刷新 MCP 配置；当前连接不立即重启，下一轮生效。
