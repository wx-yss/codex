# Rust/codex-rs

In the `codex-rs` folder where the Rust code lives, follow the rules below.

---

# YSS Personal Rules

The following YSS sections describe user-specific workflow, interaction, and local customization preferences.

YSS rules have the highest priority for the user's local development tasks unless they directly conflict with safety,
correctness, or unavoidable project-wide constraints.

---

## YSS User Background

- 用户主要从事 Java 生态开发，对 Rust 与 TUI 相关技术暂无系统性背景。
- 回答 Rust/TUI 相关问题时，优先使用“整体逻辑、执行流程、问题定位、架构行为”层面的解释。
- 非必要情况下，不要过多展开：
  - Rust 语法细节
  - trait、生命周期、宏机制
  - TUI 内部实现
  - 函数名、方法名、底层 API 细节
- 更关注：
  - 为什么会这样
  - 整体是怎么工作的
  - 问题该怎么排查
  - 修改会产生什么影响
- 如有必要，可适当类比 Java 生态中的常见概念帮助理解。
- 用户具备较强的通用工程理解与逻辑能力，不需要过度科普或过度简化。
- 对于代码修改：
  - 更强调“改了什么、为什么改”
  - 少讲底层 Rust 语言机制
  - 优先给出可执行、可验证的结论
- 回答风格保持简洁、直接、实用。

## YSS TUI Terms

- C区：底部输入与状态区域，包含 composer、状态行、弹窗等现有交互承载区。
- S区：终端 scrollback 历史显示区域，由终端滚动缓冲承载历史消息。

---

## YSS Development Workflow

- 用户本地定制开发优先追求“快速可见反馈”与“快速恢复测试能力”。
- 优先小步修改，不要一开始进行大规模重构。
- 优先解决当前问题，不主动扩散修改范围。
- 非必要情况下，不要主动进行：
  - 大范围重构
  - 大规模代码清理
  - 与当前需求无关的优化
  - 过度抽象
- 测试、编译和验证节奏按 `YSS Build and Test Workflow` 执行。
- 更关注：
  - 用户是否能快速开始测试
  - 是否能快速定位问题
  - 是否能快速得到可见结果

## YSS Worktree Workflow

- 当用户明确要求隔离工作区或使用 worktree 能力时，在 Codex 源码仓库执行：`scripts/codex-worktree-dev <任务名>`。
- 命令会输出 `WORKTREE_PATH=...`、`INIT_PID=...`、`INIT_LOG=...`；后续命令必须切到 `WORKTREE_PATH` 执行。
- 初始化预热在后台执行，不要等待它完成才开始编码。
- worktree 任务完成并合回后，清理对应 worktree 和临时分支。

---

## YSS Build and Test Workflow

- 本节优先级高于 TDD / debugging / verification 等通用流程。
- 阶段一前禁止运行任何 `cargo test`、`cargo build`、`just test`、`just fix`。
- 可以写测试代码，但不能为了红灯/绿灯主动执行测试。
- 阶段一：AI 编码完成后，只运行：
  - `cd codex-rs && just fmt`
  - `git diff --check`
- 阶段一完成后，停止自动验证，明确告知用户可以手动测试，并输出目标工作区路径。
- 阶段二：用户手动测试通过，并明确说可以提交后，运行本次变更对应模块测试：
  - `cd codex-rs && env RUST_MIN_STACK=16777216 cargo test -p <crate> --lib`
- `codex-rs/tui` 对应命令固定为：
  - `cd codex-rs && env RUST_MIN_STACK=16777216 cargo test -p codex-tui --lib`
- 未进入阶段二前，禁止主动运行 `cargo build`、`cargo test`、`just test`、`just fix` 或 workspace 级验证。

---

## YSS Debugging Strategy

- 异常排查以“先恢复用户测试能力”为最高优先级。
- 对不确定问题：
  - 优先加调试日志定位
  - 不要一开始深度分析
- 调试日志规范：
  - 日志调用和级别：`tracing::info`
  - 日志统一前缀：`YSS_DEBUG_<日志标识>_`
  - 日志文件：`~/.codex/log/codex-tui.log`
- 加日志遵循“最小成本原则”：
  - 怎么快怎么来
  - 不做额外重构
- 加完日志后：
  - 立即停止
  - 不执行编译、测试、review 等耗时操作
- 直接通知用户测试。
- 用户反馈后，再由 AI 自行读取日志继续排查。

---

## YSS Custom Feature Rules

- `docs/yss-requirements/` 用于记录用户本地定制功能需求。
- 其中内容描述的是“目标行为与需求”，不是必须逐字复制的代码实现。
- 用户本地定制相关修改：
  - 必须同步更新对应 requirement 文件。
  - 如果属于新功能，先新增 requirement 文件。
  - 如果排查到可复发的实现坑、状态不一致或错误根因，必须写入对应 requirement 文件的“踩坑记录”小节。
- 对于用户本地定制功能：
  - 优先保持行为稳定。
  - 优先兼容现有个人使用习惯。
  - 不要轻易改变交互行为。
