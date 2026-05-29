# TUI 日志按天归档

## 需求

TUI 主日志使用本机本地时区记录时间。如果本机时区为北京时间，日志时间应显示为北京时间。

当天日志仍写入原路径：

- `$CODEX_HOME/log/codex-tui.log`

跨天后，上一天的 `codex-tui.log` 归档到日志目录内的 `history` 子目录：

- `$CODEX_HOME/log/history/codex-tui.log.YYYY-MM-DD`

归档后，新一天继续在原路径创建并写入 `codex-tui.log`。

## 行为边界

归档按本机日期判断，不使用 UTC 日期。

如果同一天的归档文件已经存在，新归档内容追加到已有文件末尾，不覆盖旧内容。

日志文件继续保持仅当前用户可读写的权限。

`CODEX_TUI_RECORD_SESSION` 生成的 session JSONL 日志不受本需求影响。
