# 状态栏调整需求

## 背景

用户本地定制需要让 TUI 状态栏更紧凑，减少路径和状态项占用的字符数。

## Current Dir 路径缩短

当 TUI 状态栏启用 `current-dir` 时，用紧凑形式展示当前工作目录。

期望行为：

- 最后一级路径段保留完整。
- 中间路径段按同级目录里的最短唯一前缀缩短。
- 用户家目录下的路径保留开头的 `~`。
- 没有中间路径段的短路径保持原样，例如 `~/codex`。
- Windows 下保持原有完整目录显示。

示例：

- 当 `Doc` 是 `Documents` 的最短唯一前缀时，`~/Documents/project/remote/codex` 变成 `~/Doc/p/r/codex`。
- `~/projects/codex` 变成 `~/p/codex`。

补充约束：

- 如果某个中间路径段在同级目录中找不到唯一前缀，则保留该路径段完整名称。
- 只有能通过同级目录确认最短唯一前缀时才缩短；读取同级目录失败或无法确认时，也保留完整名称。
- 状态栏刷新频繁，实现时应避免对同一个当前工作目录反复扫描文件系统。

## Context Remaining 紧凑显示

当 TUI 状态栏启用 `context-remaining` 时，只展示剩余百分比。

期望行为：

- `Context 100% left` 显示为 `100%`。
- 只影响状态栏/状态栏配置预览中的 `context-remaining` 文案。
- `/status` 详情卡片里的 `Context window` 描述保持原样。
