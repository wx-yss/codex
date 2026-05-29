# TUI 显示 yss 本地构建版本

## 需求

当 yss 使用本地构建产物运行 Codex TUI 时，状态显示里的版本信息需要在官方 CLI 版本后额外展示一段 yss 本地构建版本。

显示格式如下：

- `v<官方版本> - <YSS_VERSION>`

示例：

- `v0.130.0 - 20260510-001`

## 生效范围

只在 TUI 的状态显示中生效：

- 手动执行 `/status` 生成的 status 卡
- 启动自动展示的 status 卡
- 新会话 header 和 `/clear` 后重绘的 header

## 非目标

以下位置继续保持官方版本，不拼接 yss 本地构建版本：

- `codex --version`
- 更新检查和版本比较逻辑
- 会话记录、埋点、协议字段里的 `cli_version`
- 其他不属于状态显示的内部版本使用点

## 版本来源

官方版本继续使用编译时的 `CARGO_PKG_VERSION`。

yss 本地构建版本使用新的编译时环境变量 `YSS_VERSION`。

当 `YSS_VERSION` 未设置时，状态显示退化为仅展示官方版本，不显示额外分隔符或占位文本。

## 当前实现落点

TUI 展示版本统一由 `codex-rs/tui/src/version.rs` 的状态展示版本函数生成。

以下入口使用状态展示版本：

- `/status` 卡片标题
- 启动自动展示的 session header
- session 配置完成前的 placeholder header
- `/clear` 和 Ctrl-L 重绘的 header

其他版本用途继续直接使用官方版本常量。
