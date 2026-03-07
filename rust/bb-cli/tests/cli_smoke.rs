use std::fs;
use std::process::Command;

use httpmock::Method::GET;
use httpmock::MockServer;
use serde_json::json;
use tempfile::tempdir;

fn bb_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_bb"))
}

#[test]
fn root_help_prints_commands() {
    let output = bb_command().output().expect("command should run");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("bb - Bitbucket CLI"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("auth"));
    assert!(stdout.contains("completion"));
}

#[test]
fn version_prints_metadata() {
    let output = bb_command()
        .arg("version")
        .output()
        .expect("command should run");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains(&format!("bb version {}", env!("CARGO_PKG_VERSION"))));
    assert!(stdout.contains("commit:"));
    assert!(stdout.contains("built:"));
}

#[test]
fn auth_status_without_login_writes_error_to_stderr() {
    let temp = tempdir().unwrap();
    let output = bb_command()
        .args(["auth", "status"])
        .env("BB_CONFIG_PATH", temp.path().join("config.json"))
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("not logged in"));
}

#[test]
fn repo_list_json_reads_config_and_calls_server() {
    let server = MockServer::start();
    let repos = server.mock(|when, then| {
        when.method(GET).path("/2.0/repositories/acme");
        then.json_body(json!({
            "values": [
                {"slug": "one", "full_name": "acme/one"}
            ]
        }));
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    fs::write(
        &config_path,
        format!(
            "{{\n  \"current\": \"default\",\n  \"profiles\": {{\n    \"default\": {{\n      \"base_url\": \"{}/2.0\",\n      \"token\": \"token-123\",\n      \"username\": \"\"\n    }}\n  }}\n}}\n",
            server.base_url()
        ),
    )
    .unwrap();

    let output = bb_command()
        .args(["repo", "list", "--workspace", "acme", "--output", "json"])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body[0]["full_name"], "acme/one");
    repos.assert();
}

#[test]
fn completion_bash_prints_script() {
    let output = bb_command()
        .args(["completion", "bash"])
        .output()
        .expect("command should run");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("complete -F _bb_complete bb"));
}
