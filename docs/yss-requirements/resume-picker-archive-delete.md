# Resume picker 删除归档

## 目标

在 `codex resume` 的会话选择器中，可以用键盘把不想继续出现在列表里的历史会话归档隐藏。

## 期望行为

- 只在 resume picker 生效；fork picker 不响应删除归档。
- 选中会话后按 `Del` 进入确认状态，状态行提示再次按 `Del` 归档该会话。
- 对同一个会话再次按 `Del` 后调用已有的 `thread/archive` 能力归档 thread。
- 归档成功后，该会话立即从当前 picker 列表中消失，选中位置保持在相邻可见会话上。
- 归档失败时，picker 保持打开，并在状态行展示错误；后续重试需要重新按两次 `Del`。
- 移动选择、搜索、切换排序或筛选、退出 picker 时，取消未确认的归档状态。

## 边界

- 该功能不物理删除 JSONL rollout 文件，只使用归档语义隐藏会话。
- 不在 picker 内提供恢复入口；恢复继续依赖已有 unarchive 能力。
- 缺少 thread metadata 的行不能归档，应展示错误而不是尝试从路径推断。
- 当前 picker 生命周期内已经归档的 thread 不能被较晚返回的分页结果重新加入列表。

## 当前分支实现备注

- 新版 resume/fork picker 共用 app-server 线程列表；删除归档只在 `SessionPickerAction::Resume` 下响应。
- 归档通过 app-server 既有 `thread/archive` 请求完成，picker 只负责二次确认、隐藏行和过滤晚返回分页结果。
