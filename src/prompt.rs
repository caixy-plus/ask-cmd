/// 运行时环境描述：OS（含 Linux 发行版）、架构、用户实际 shell。
/// 让 AI 按真实环境生成命令（BSD/GNU 工具差异、包管理器、shell 语法）。
fn environment_hint() -> String {
    let os = if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else if cfg!(target_os = "macos") {
        "macOS (BSD userland)".to_string()
    } else {
        linux_distro()
            .map(|d| format!("Linux ({d})"))
            .unwrap_or_else(|| "Linux".to_string())
    };

    let shell = if cfg!(target_os = "windows") {
        "PowerShell".to_string()
    } else {
        std::env::var("SHELL")
            .ok()
            .and_then(|s| s.rsplit('/').next().map(str::to_string))
            .unwrap_or_else(|| "sh".to_string())
    };

    format!("{os}, {} arch, shell: {shell}", std::env::consts::ARCH)
}

fn linux_distro() -> Option<String> {
    let content = std::fs::read_to_string("/etc/os-release").ok()?;
    content
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .map(|v| v.trim_matches('"').to_string())
}

pub fn suggestions_system_prompt() -> String {
    let shell_hint = environment_hint();

    format!(
        "You are a shell command assistant.\n\
         Target environment: {shell_hint}.\n\
         Commands MUST fit this exact environment (tool variants, package manager, shell syntax).\n\
         Your job: turn ANY user request into 5 concrete, runnable shell commands.\n\
         Always ask yourself: \"what terminal action would accomplish this?\"\n\
         Naming/brainstorming requests → pick concrete names, use mkdir/git init/touch.\n\
         Example: \"think of a project name\" → [\"mkdir nova-browser\", \"mkdir chromite\", ...]\n\
         Make concrete choices — never use placeholders.\n\
         If ambiguous, show 5 different interpretations as commands.\n\
         Only return [] for pure opinions/emotions with zero terminal relevance (e.g. 'I hate Mondays').\n\
         Return ONLY a JSON array of strings. No markdown, no explanation."
    )
}

pub fn suggestions_user_prompt(query: &str) -> String {
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    format!("Current directory: {cwd}\nRequest: {query}")
}

pub fn single_system_prompt() -> String {
    let shell_hint = environment_hint();

    format!(
        "You convert natural-language requests into one shell command.\n\
         Target environment: {shell_hint}. The command MUST fit this exact environment.\n\
         Reply with ONLY the command itself — one line, no markdown, no backticks, no explanation.\n\
         Use real filenames when given; never use placeholders like 文件名 or filename."
    )
}

pub fn single_user_prompt(query: &str) -> String {
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    format!("Current directory: {cwd}\nConvert to ONE executable shell command. Request: {query}")
}
