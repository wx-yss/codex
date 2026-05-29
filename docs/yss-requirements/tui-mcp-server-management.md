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

## 当前分支复刻记录

- `/mcp` 入口改为打开 C 区 MCP server 管理弹窗，并异步请求 `mcpServerStatus/list` 刷新工具数量和 Auth 状态；旧的 S 区 MCP inventory history cell 代码暂留，但 `/mcp` 不再走旧清单输出路径。
- 弹窗以当前 `config.mcp_servers` 为准生成 server 列表，所以已禁用 server 仍会显示；app-server status 只补充工具数量和 Auth 状态。
- `/mcp verbose` 和其他 `/mcp` 参数统一显示 `Usage: /mcp`，不再触发 verbose 清单。
- 启停写入使用 `config/batchWrite` 的 `mcp_servers."<server>".enabled` 叶子路径，并在成功后调用 `config/mcpServer/reload`。
- App 层按 server 名串行化启停写入；同一 server 连续切换时，后一次选择进入 pending 队列，最终落盘状态以最后一次选择为准。

## 踩坑记录

- 通用 `ListSelectionView` 在搜索输入非空时会把空格当作搜索字符处理，不能直接满足“搜索后空格切换选中 server”的交互；本功能使用 MCP 专用 view，交互模式参考 skills toggle。
- MCP server 名称可能包含 `.` 或引号，写配置时必须把 key path 的 server 段写成 quoted segment，避免误拆成多级路径。
- 异步 status 刷新可能晚于用户切换操作返回；弹窗刷新时必须叠加 pending enabled 覆盖，避免写入未完成时 UI 又显示回旧状态。
