# 条件式 Skill Root Provider

## 背景

当前 Codex 会默认加载通用用户技能目录 `$HOME/.agents/skills`。这个目录适合放个人通用 skills。

用户同时存在公司和个人在家的使用环境。公司专用 skills 不期望在个人环境下被加载，也不希望把公司目录或个人判断逻辑写死到 Codex 源码里。

官方 `config.toml` 不应加入个人定制字段。原因是官方二进制可能不认识这些字段，未来存在解析失败或兼容性风险。

## 目标

保留 `$HOME/.agents/skills` 作为通用用户 skills 目录。

新增一种本地定制扩展点，用于在特定条件下动态追加额外 skill root。Codex 只负责提供应用上下文、执行 provider、读取返回结果，不内置公司/个人环境的具体判断逻辑。

## 推荐方案

使用外部脚本协议实现 skill root provider。

Codex 定制版扫描固定目录：

```text
$HOME/.agents/codex/skill-root-providers.d/
```

目录里的可执行文件都视为 provider。Codex 向 provider 的 stdin 写入上下文 JSON，provider 从 stdout 返回额外 skill roots。

这个目录不属于官方 Codex 配置。官方二进制不会读取它，也不会因为个人字段导致配置解析失败。

## Provider 输入

输入 JSON 先保持小而稳定，例如：

```json
{
  "schema_version": 1,
  "cwd": "/Users/yss/Documents/project/company/foo",
  "home": "/Users/yss",
  "codex_home": "/Users/yss/.codex",
  "session_source": "tui"
}
```

后续如确有需要，再扩展字段。Provider 不应依赖 Codex 内部 Rust 类型。

## Provider 输出

输出 JSON 示例：

```json
{
  "roots": [
    {
      "path": "/Users/yss/.agents/company-skills",
      "enabled": true,
      "label": "company"
    }
  ]
}
```

`enabled = false` 的 root 不加载。`label` 仅用于调试或展示，不参与扫描路径判断。

Provider 可以基于 cwd、环境变量、网络状态、本地文件、公司 VPN 等任意用户逻辑决定是否返回公司 skills 目录。

## 加载行为

Provider 返回的有效 roots 作为 `User` scope skill root 追加到现有 roots 中。

现有加载顺序和能力保持不变：

- `$HOME/.agents/skills` 仍然作为通用用户 skills 目录加载。
- `$CODEX_HOME/skills` 继续作为旧用户目录兼容。
- repo 内 `.agents/skills`、system skills、admin skills、plugin skills 保持现有行为。
- Provider 只追加额外 roots，不替换现有 roots。

Provider 非 0 退出、超时、stdout 非法 JSON、返回路径非法时，不加载该 provider 的 roots。Codex 记录 warning，但不阻塞启动和正常对话。

Provider 执行应有较短超时，避免每轮加载 skills 时明显拖慢 TUI。初始建议 800ms 到 1000ms。

## Watcher 边界

不专门监听 provider 脚本目录。

Provider 脚本新增、删除或逻辑变更后，用户重启 TUI 生效即可。

如果 provider 返回的 skill root 被加入现有 `skill_roots_for_config` 结果，则现有 skills watcher 会自然注册这些 root。无需额外开发 provider watcher。

本需求不要求 provider 条件变化实时生效。例如 VPN 状态变化、cwd 外部条件变化等场景，可以通过重启 TUI 或现有刷新路径处理。

## 接入点参考

主要接入点预计在 `core-skills` 的 skill roots 构建阶段。

实现时应优先把 provider roots 纳入统一的 roots 构建结果，让后续扫描、缓存、skills/list 和现有 watcher 尽量复用原流程。

缓存 key 需要包含 provider 返回后的有效 roots，避免不同条件下复用错误的 skills 结果。

## 实现记录

当前实现将 provider 协议放在 `core-skills` 内部，不扩展官方 `config.toml` schema。

Provider 目录固定为：

```text
$HOME/.agents/codex/skill-root-providers.d/
```

Codex 会执行该目录下的普通文件，并通过 stdin 传入 JSON 上下文。当前协议版本为 `schema_version = 1`，包含 `cwd`、`home`、`codex_home` 和 `session_source`。

Provider 必须在 stdout 输出 JSON：

```json
{
  "roots": [
    {
      "path": "/absolute/path/to/skills-root",
      "enabled": true,
      "label": "optional"
    }
  ]
}
```

只有 `enabled = true` 且 path 为绝对路径的 root 会被采纳。Provider 退出失败、超时、输出非法 JSON 或返回非法路径时，Codex 只记录 warning 并忽略该 provider 的结果。

Provider 执行超时当前为 1000ms。Provider 返回的 roots 会以 `User` scope 追加到现有 skill roots 中，并参与现有 roots-based cache key。

实现不专门监听 provider 脚本目录。Provider 脚本新增、删除或逻辑变更后，重启 TUI 生效即可。Provider 返回的 skill root 进入 `skill_roots_for_config` 后，会复用现有 skills watcher 的 root 监听路径。

## 边界

不向官方 `config.toml` schema 增加个人定制字段。

不引入 Rust trait 或动态库接口。Provider 使用进程级 JSON stdin/stdout 协议，方便用 shell、Python、Node 等脚本实现。

不保证 provider 脚本热更新。

不要求支持远程环境下的 provider 执行。初始目标是本机 TUI 使用场景。

## 踩坑记录

- 不能把个人专用配置放进官方 `config.toml`，否则官方二进制遇到未知字段时可能出现兼容性问题。
- Provider 脚本目录不需要额外 watcher；否则会增加实现复杂度，而当前使用场景中 provider 逻辑不频繁变更。
- 条件判断逻辑应留在 provider 内部，Codex 不应内置“公司/个人/在家”等具体概念。
