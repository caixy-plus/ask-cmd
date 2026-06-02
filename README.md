# ask-cmd

Natural language → shell command. Cross-platform CLI written in Rust.

Works in **bash**, **zsh**, **fish**, **PowerShell**, and any terminal that can run a subprocess.

```bash
$ ask -n create empty file hello.txt
touch hello.txt

$ ask create empty file hello.txt
Suggested:  touch hello.txt
Run? [Y/n/c(opy)/q]:
```

## Install

### From source

```bash
git clone https://github.com/caixy-plus/ask-cmd.git
cd ask-cmd
cargo install --path .
ask-cmd install          # adds `ask()` to your shell rc
ask-cmd init             # optional: ~/.config/ask-cmd/config.toml
```

Restart the terminal, or `source ~/.zshrc` (or your shell rc).

### Requirements

| Provider | Needs |
|----------|--------|
| `claude` (default) | [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) logged in |
| `openai` | API key in config or `OPENAI_API_KEY` |
| `ollama` | Local [Ollama](https://ollama.com) server |

## Usage

```bash
ask 创建空文件 test.txt       # AI → confirm → run
ask -n 怎么创建文件 test.txt  # print command only
ask-cmd install --shell fish
ask-cmd install --shell powershell   # Windows
```

Confirm: **Y** run · **n** cancel · **c** copy · **q** quit

## Config

`~/.config/ask-cmd/config.toml`:

```toml
provider = "claude"
timeout_secs = 45

# openai_api_key = "sk-..."
# openai_base_url = "https://api.openai.com/v1"
# openai_model = "gpt-4o-mini"

# ollama_url = "http://127.0.0.1:11434"
# ollama_model = "llama3.2"
```

## Shell integration

`ask-cmd install` writes a small wrapper:

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
