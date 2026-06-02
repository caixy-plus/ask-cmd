pub fn suggestions_system_prompt() -> String {
    let shell_hint = if cfg!(target_os = "windows") {
        "Windows PowerShell or cmd.exe"
    } else if cfg!(target_os = "macos") {
        "macOS zsh/bash"
    } else {
        "Linux bash/sh"
    };

    format!(
        "You suggest shell commands for {shell_hint}.\n\
         Given a user request, return exactly 5 different valid executable commands, from common to alternative approaches.\n\
         Return ONLY a JSON array of strings. No markdown fences. No explanation.\n\
         Use real filenames when given; never use placeholders like 文件名 or filename.\n\
         Each string must be one complete command."
    )
}

pub fn suggestions_user_prompt(query: &str) -> String {
    format!("Suggest 5 shell commands for: {query}")
}

pub fn single_system_prompt() -> String {
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
         Use real filenames when given; never use placeholders like 文件名 or filename."
    )
}

pub fn single_user_prompt(query: &str) -> String {
    format!("Convert to ONE executable shell command. Request: {query}")
}
