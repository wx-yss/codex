# TUI 共识文件预览命令

## 需求

Codex TUI 应提供一个本地 slash 命令，用于在 S区直接查看最近一次共识落盘文件内容。

命令名：

- `/consensus`

用户执行 `/consensus` 后，TUI 从当前会话的 `App::transcript_cells` 倒序查找最近出现的共识文件路径。路径中必须包含固定片段：

- `/.agents/consensus-compactions/`

找到最近的 `.md` 文件后，TUI 本地读取文件内容，并将内容插入 S区显示。

## 行为边界

`/consensus` 是本地 UI 命令，不发送给 AI，不触发模型请求。

展示内容只进入 TUI 的 S区显示，不应作为用户消息、assistant 消息或 tool result 注入模型上下文。

如果路径前面有空格、Markdown 链接包装或反引号包装，应能通过 `trim` 和简单边界识别提取出真实文件路径。

路径匹配不能硬编码用户名或用户主目录，例如不能只匹配 `/Users/yss/...`。提取路径时应以 `/.agents/consensus-compactions/` 为锚点，并保留锚点前面的用户目录部分，得到完整绝对路径。

如果当前 `transcript_cells` 中没有共识文件路径，S区显示提示：

- `No consensus compaction file found in this session.`

如果文件不存在、不是普通文件或读取失败，S区显示对应错误提示。

## 显示要求

成功读取文件后，S区先显示文件路径标题，再显示 Markdown 原文内容。

标题格式：

- `Consensus file: <path>`

## 踩坑记录

不要通过监听共识目录自动展示文件内容。用户可能同时打开多个 TUI，全局目录监听会导致多个 TUI 同时显示同一份内容。

不要让 AI 读取文件后再输出内容。这样会让共识文件内容重复进入后续模型上下文。

不要从 core `Session::clone_history()` 查找路径。该功能面向当前 TUI 的 S区展示，应以 `App::transcript_cells` 为准。
