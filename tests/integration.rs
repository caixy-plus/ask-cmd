use ask_cmd::suggestions::parse_suggestions;
use ask_cmd::validate::{looks_like_command, validate_command};

#[test]
fn pipeline_mock_response() {
    let raw = "touch /tmp/demo.txt";
    let cmd = ask_cmd::extract::extract_command(raw).unwrap();
    validate_command(&cmd).unwrap();
    assert!(looks_like_command(&cmd));
}

#[test]
fn parses_multiple_suggestions() {
    let raw = r#"["touch a.txt", "echo '' > a.txt", "ls"]"#;
    let cmds = parse_suggestions(raw).unwrap();
    assert!(cmds.len() >= 2);
}

#[test]
fn rejects_chinese_placeholder() {
    let raw = r#"["touch 文件名"]"#;
    assert!(parse_suggestions(raw).is_err());
}

#[test]
fn claude_hint_is_helpful() {
    let hint = ask_cmd::claude_install_hint();
    assert!(hint.contains("claude login"));
    assert!(hint.contains("Claude Code"));
}
