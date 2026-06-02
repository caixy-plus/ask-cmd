# ask-cmd

Natural language → shell command. Cross-platform CLI written in Rust.

**仅通过本机 [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) 调用，不支持自行接入 API Key。**

Works in **bash**, **zsh**, **fish**, **PowerShell**, and any terminal that can run a subprocess.

```bash
$ ask -n create empty file hello.txt
touch hello.txt

$ ask create empty file hello.txt
Suggested:  touch hello.txt
Run? [Y/n/c(opy)/q]:
```

## 前置条件

安装并登录 Claude Code：

```bash
npm install -g @anthropic-ai/claude-code
claude login
claude -p hello    # 验证可用
```

未安装时 `ask` 会提示安装步骤。

## Install

```bash
git clone https://github.com/caixy-plus/ask-cmd.git
cd ask-cmd
cargo install --path .
ask-cmd install          # adds `ask()` to your shell rc
```

Restart the terminal, or `source ~/.zshrc` (or your shell rc).

## Usage

```bash
ask 创建空文件 test.txt       # Claude → confirm → run
ask -n 怎么创建文件 test.txt  # print command only
ask-cmd install --shell fish
ask-cmd install --shell powershell   # Windows
```

Confirm: **Y** run · **n** cancel · **c** copy · **q** quit

## Shell integration

| Shell | File |
|-------|------|
| bash | `~/.bashrc` |
| zsh | `~/.zshrc` |
| fish | `~/.config/fish/conf.d/ask-cmd.fish` |
| PowerShell | `$PROFILE` |

Manual snippets live in `integrations/`.

## Test

```bash
./scripts/test.sh
```

## License

MIT
