use ask_cmd::config::Config;
use ask_cmd::extract::extract_command;
use ask_cmd::validate::{looks_like_command, validate_command};

#[test]
fn pipeline_mock_response() {
    let raw = "touch /tmp/demo.txt";
    let cmd = extract_command(raw).unwrap();
    validate_command(&cmd).unwrap();
    assert!(looks_like_command(&cmd));
}

#[test]
fn rejects_chinese_placeholder() {
    let raw = "touch 文件名";
    assert!(extract_command(raw).is_err() || validate_command("touch 文件名").is_err());
}

#[test]
fn default_config_loads() {
    let cfg = Config::default();
    assert_eq!(cfg.provider, "claude");
}
