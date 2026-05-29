# TUI agent 和 resume 状态标记

## 背景

`/agent` 弹框列出可切换的主会话和子代理，但只看列表时不容易判断哪些代理仍在处理、哪些已经结束。

`/resume` 会话列表从当前会话中打开时，不容易判断列表里的哪一项是进入列表前正在查看的会话。

## 需求

- `/agent` 弹框的每一行必须显示代理状态：
  - 仍有未完成 turn 的代理显示 `running`。
  - 最近 turn 已结束且线程仍可查看的代理显示 `completed`。
  - 线程已关闭或不可继续的代理显示 `closed`。
- `/agent` 当前行由通用选择器显示 `(current)`，状态描述里不重复显示 `current`。
- `/agent` 弹框继续保留已有的状态点和搜索能力；可见描述优先展示状态，线程 ID 可继续用于搜索。
- 从已有 TUI 会话打开 `/resume` 时，当前正在查看的会话必须在列表中标记为 `current`。
- 启动阶段打开的 resume picker 没有进入前的当前会话，不显示 `current`。
- `/resume` 的 `current` 标记应作为可扩展状态位置存在，未来可以继续添加其他状态。

## 展示规则

- comfortable resume 列表把 `current` 放在会话元信息行中。
- dense resume 列表保留固定状态列，当前会话在该列显示 `current`，其他会话留空。
- current 判定必须使用用户当前正在查看的 thread id，而不是只依赖内部切换过程中的临时 active 状态。

## 当前分支复刻记录

- `/agent` 使用明确生命周期状态：`running`、`completed`、`closed`。
- `/agent` 状态点继续保留；可见描述显示生命周期状态，当前行仍由通用选择器显示 `(current)`。
- 会话内 `/resume` 传入打开列表前正在查看的 thread id；启动阶段 resume picker 不传入当前 thread id。
- `/resume` 的 `current` 作为列表状态位渲染，comfortable 放入元信息行，dense 使用固定状态列。

## 踩坑记录

- `/resume` 的 `current` 判定不能直接复用 `current_displayed_thread_id()`；该函数优先取 `active_thread_id`，当内部 active 状态和用户正在查看的 ChatWidget 短暂不一致时，会把错误 thread id 传给 picker，导致列表中没有任何 row 命中 `current`。
- 会话内 `/resume` 必须优先使用 `chat_widget.thread_id()` 作为用户当前正在查看的 thread id，必要时再回退到 `active_thread_id`。
