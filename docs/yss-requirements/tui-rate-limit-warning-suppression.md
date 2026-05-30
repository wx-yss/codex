# TUI 低额度提醒隐藏

## 背景

用户本地定制需要减少 TUI 中低额度阈值提醒对日常使用的干扰。

## 期望行为

当账号额度使用率跨过低额度提醒阈值时，不在对话历史或界面中展示以下类型提醒：

- 5 小时额度剩余不足提醒
- 周额度剩余不足提醒
- 月额度或其他同类低额度阈值提醒

示例文案：

- `Heads up, you have less than 25% of your weekly limit left. Run /status for a breakdown.`
- `Heads up, you have less than 25% of your 5h limit left. Run /status for a breakdown.`

## 非目标

以下能力继续保留：

- `/status` 中的额度详情展示
- 状态栏中的额度信息展示
- 额度耗尽或请求触发限流后的错误处理与提示
- 切换模型等限流恢复交互

## 当前实现落点

低额度阈值提醒由 `codex-rs/tui/src/chatwidget/rate_limits.rs` 的 `RateLimitWarningState::take_warnings` 生成。

当前本地定制要求该方法对阈值提醒保持静默，返回空提醒列表。
