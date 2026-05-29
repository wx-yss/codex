# TUI 用户 prompt

Codex TUI 应在 chat composer 初始化时，从 `$CODEX_HOME/prompts` 加载用户 prompt 文件。初始化后新增、删除或
重命名的 prompt 文件，不需要同步反映到当前 TUI 会话里。

prompt 文件是命名为 `<name>.md` 的 Markdown 文件。命令名由文件名 stem 推导，并暴露为 `/prompt:<name>`，
例如 `opsx-apply.md` 对应 `/prompt:opsx-apply`。

slash 命令弹窗打开时，TUI 将内置 slash 命令和已加载的 prompt metadata 合并成一个候选列表。弹窗是否显示以及
实际渲染哪些候选，都基于这个统一列表做大小写不敏感匹配；先做 exact 和 prefix 匹配，只有没有 exact/prefix
候选时才用 fuzzy 兜底。fuzzy 使用字符顺序匹配，例如输入 `cpt` 可以候选出 `compact`。

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

## 当前分支复刻记录

- 新增 TUI 用户 prompt 加载模块：启动/ChatWidget 构造时一次性读取 `$CODEX_HOME/prompts/*.md` 的 metadata 和正文。
- slash 候选统一通过 `SlashCommandItem` 承载内置命令、service tier 命令和用户 prompt。
- slash 弹窗候选优先使用 exact/prefix，保留前缀输入的原有体验；无 exact/prefix 候选时再用 fuzzy 兜底，支持非连续字符顺序匹配。
- 用户 prompt 在弹窗中显示为 `/prompt:<name>`，描述来自 frontmatter `description`。
- 弹窗选择用户 prompt 只回填 `/prompt:<name> `，不立即提交。
- live 与 queued slash 解析都支持用户 prompt，并在提交时使用启动阶段已缓存的 prompt 正文，不重新扫描或读取 prompts 目录。
- prompt 有参数时，发送内容为 prompt 正文、空行、参数；多行参数同样拼接到正文后。
- prompt 参数继续复用现有 composer 提交流程，保留 text elements、mention bindings、图片等提交能力。
- prompt 文件为空时，TUI 输出明确错误并不发送空消息。

## 踩坑记录

- prompt 候选名必须使用完整命令 `prompt:<name>` 参与匹配和渲染；如果只用文件 stem，会出现弹窗显示 `/name` 或 `/prompt:` 前缀无法匹配的问题。
- prompt 正文加载不能直接接收用户输入路径，只能在启动阶段通过校验后的文件 stem 读取 `$CODEX_HOME/prompts/<name>.md`，避免路径穿越。
- queued slash 解析不能重新扫描 prompts 目录，否则启动后新增、删除、重命名文件会影响当前会话的可见命令集合。
- live inline 参数需要继续走 composer 的 inline args preparation，否则大段粘贴、text elements、mention bindings、图片附件会丢失或范围错位。
