# 双击 ESC 中断

Codex TUI 在 agent 正在运行时，单次按下 ESC 不应立即中断任务。

对于通过 ESC 触发的中断路径，第一次按下 ESC 只进入 200ms 的待确认状态。在这个时间窗口内第二次按下
ESC，才确认中断并发送现有的 interrupt 操作。超过 200ms 后，待确认状态自然失效，下一次 ESC 会重新开始
一个确认窗口。

只有 `KeyEventKind::Press` 计入这套双击逻辑。Repeat 和 Release 事件不能进入待确认状态，也不能确认中断，
因此长按 ESC 不应被识别为双击。

这个需求适用于当前这些 ESC 中断路径：

- composer 上方的运行中任务状态行；
- pending steer 中断路径，也就是中断后立即发送已排队 steer 消息的场景；
- 使用 ESC 中断当前 turn 的 request-user-input 弹层。

其他 ESC 行为，例如关闭弹窗、退出 Vim insert mode、编辑上一条消息、在某个视图内做本地取消，都应继续保持为
本地 UI 行为，不能变成 agent 中断。

## 实现记录

- 2026-05-30：新增独立 ESC 中断确认状态机，并接入运行中状态行、pending steer、request-user-input 三条
  ESC 中断路径；非 ESC 中断键保持单击中断。

## 踩坑记录

- 当前分支里运行中状态行和 pending steer 路径曾接受 `KeyEventKind::Press | Repeat`，request-user-input
  路径也只排除了 `Release`。实现双击确认时必须统一收敛为只有 `Press` 能进入确认窗口，`Repeat` 和
  `Release` 只能被消费，不能 arm 或 confirm，否则长按 ESC 会被误识别为双击。
- request-user-input 的 notes 区有本地 ESC 行为：有选项且 notes UI 展开时，第一次 ESC 要先收起 notes，
  不能进入中断确认窗口。
- 第二次 ESC 确认成功后必须立即 disarm，不能续上新的 200ms 窗口；否则第三次 ESC 紧接着按下会被当作
  又一次确认，可能重复发送 interrupt。
