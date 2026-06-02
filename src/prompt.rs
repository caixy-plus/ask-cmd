pub fn system_prompt() -> String {
    let shell_hint = if cfg!(target_os = "windows") {
        "Windows PowerShell or cmd.exe"
    } else if cfg!(target_os = "macos") {
        "macOS zsh/bash"
    } else {
        "Linux bash/sh"
    };

    format!(
        "You convert natural-language requests into one {shell_hint} command.\n\
         Reply with ONLY the command itself — one line, no markdown, no backticks, no explanation.\n\
         Use real filenames when given; never use placeholders like 文件名 or filename.\n\
         If the request is vague, pick the simplest command (touch/New-Item for create file, ls/Get-ChildItem for list)."
    )
}

pub fn user_prompt(query: &str) -> String {
    format!("Convert to ONE executable shell command. Request: {query}")
}
