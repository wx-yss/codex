# TUI 用户 prompt

Codex TUI 应在 chat composer 初始化时，从 `$CODEX_HOME/prompts` 加载用户 prompt 文件。初始化后新增、删除或
重命名的 prompt 文件，不需要同步反映到当前 TUI 会话里。

prompt 文件是命名为 `<name>.md` 的 Markdown 文件。命令名由文件名 stem 推导，并暴露为 `/prompt:<name>`，
例如 `opsx-apply.md` 对应 `/prompt:opsx-apply`。

slash 命令弹窗打开时，TUI 将内置 slash 命令和已加载的 prompt metadata 合并成一个候选列表。弹窗是否显示以及
实际渲染哪些候选，都基于这个统一列表做大小写不敏感的前缀匹配。

prompt 文件可以使用桌面端 prompt frontmatter 格式：

```md
---
description: 显示在 slash 弹窗里的描述
argument-hint: 命令参数
---

Prompt 正文...
```

`description` 字段显示在 slash 弹窗里。prompt 正文是 frontmatter 块之后的内容。`argument-hint` 作为文件格式的一
部分被接受，但 TUI 暂时不使用它。

从 slash 命令弹窗选择某个 prompt 时，只把 `/prompt:<name> ` 插入 composer，不应立即提交。这样用户可以继续
输入 inline 参数后再发送。

提交 `/prompt:<name>` 时，将 prompt 正文作为用户消息发送。提交 `/prompt:<name> <args>`，或提交
`/prompt:<name>` 后面跟着后续行内容时，发送内容应为 prompt 正文、一个空行、以及对应参数内容。

实现时应把用户 prompt 作为 slash 命令候选的一类，和内置 slash 命令、动态服务档位命令共享候选匹配与弹窗渲染
链路，避免不同入口的可见性、补全和提交行为不一致。
