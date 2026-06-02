# ask-cmd 跨平台实现计划

> **Goal:** 将 Oh My Zsh 插件 `ai-cmd` 升级为跨平台 Rust CLI，支持 bash / zsh / fish / PowerShell 及任意终端。

**Architecture:** Rust 二进制 `ask-cmd` 仅调用本机 `claude` CLI；各 shell 保留薄包装 `ask()`。不支持 OpenAI/Ollama 等自接 API。

**Tech Stack:** Rust, clap, tokio (subprocess)

---

## 模块

| 模块 | 职责 |
|------|------|
| `extract` | 从 markdown / JSON / 纯文本提取命令 |
| `validate` | 拒绝说明文字、占位符、危险命令提示 |
| `prompt` | 按 OS 生成 system / user prompt |
| `providers` | 仅 claude CLI（未安装时友好提示） |
| `shell` | `install` 写入各 shell rc |
| `cli` | 子命令：默认查询、`-n` 仅输出、install |

## Shell 集成

- **bash** → `~/.bashrc`
- **zsh** → `~/.zshrc`
- **fish** → `~/.config/fish/conf.d/ask-cmd.fish`
- **PowerShell** → `$PROFILE` 追加 snippet

## 测试

- Rust 单元测试：`extract`, `validate`
- 集成测试：mock provider
- `scripts/test.sh`：编译 + 单元 + 可选 live claude

## 交付

- GitHub: `caixy-plus/ask-cmd`（或当前 gh 账号）
- README：安装、用法、配置、各 shell 说明
