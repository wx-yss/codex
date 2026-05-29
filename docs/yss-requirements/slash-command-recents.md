# TUI slash 命令最近使用记录

Codex TUI 应把最近使用过的 slash 命令记录到 `$CODEX_HOME/slash_command_recents.json`。

只有命令真正被使用后才记录，不能因为它出现在 slash 弹窗里就记录。内置 slash 命令使用命令名作为 key，例如
`model` 或 `review`。用户 prompt 命令使用完整 prompt 命令名，例如 `prompt:opsx-apply`。

最近使用文件是一个 LRU 列表。记录已存在命令时，将它移动到最前面；记录新命令时，将它插入到最前面；列表最多
保留 50 条。

slash 命令弹窗打开时，TUI 读取一次最近使用文件，并只用它调整当前已经匹配到的候选顺序。最近使用记录不能让
原本不匹配当前输入的命令出现在弹窗里。

最近使用文件读取、解析或写入失败时，不应阻塞命令执行。
