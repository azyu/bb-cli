use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use httpmock::Method::{DELETE, GET};
use httpmock::MockServer;
use serde_json::json;
use tempfile::{TempDir, tempdir};

fn bb_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_bb"))
}

fn write_config(config_path: &std::path::Path, base_url: &str) {
    fs::write(
        config_path,
        format!(
            "{{\n  \"current\": \"default\",\n  \"profiles\": {{\n    \"default\": {{\n      \"base_url\": \"{base_url}\",\n      \"token\": \"token-123\",\n      \"username\": \"\"\n    }}\n  }}\n}}\n"
        ),
    )
    .unwrap();
}

fn run_git<I, S>(dir: &Path, args: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("git command should run");
    if !output.status.success() {
        panic!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    String::from_utf8(output.stdout).expect("git stdout should be utf-8")
}

struct CheckoutRepo {
    _temp: TempDir,
    worktree: PathBuf,
    feature_commit: String,
}

fn setup_checkout_repo(source_branch: &str) -> CheckoutRepo {
    let temp = tempdir().unwrap();
    let bare = temp.path().join("remote.git");
    let seed = temp.path().join("seed");
    let worktree = temp.path().join("worktree");

    run_git(temp.path(), ["init", "--bare", bare.to_str().unwrap()]);
    run_git(temp.path(), ["init", seed.to_str().unwrap()]);
    run_git(&seed, ["config", "user.name", "Test User"]);
    run_git(&seed, ["config", "user.email", "test@example.com"]);

    fs::write(seed.join("README.md"), "seed\n").unwrap();
    run_git(&seed, ["add", "README.md"]);
    run_git(&seed, ["commit", "-m", "initial"]);
    run_git(&seed, ["branch", "-M", "main"]);
    run_git(&seed, ["remote", "add", "origin", bare.to_str().unwrap()]);
    run_git(&seed, ["push", "-u", "origin", "main"]);

    run_git(&seed, ["checkout", "-b", source_branch]);
    fs::write(seed.join("feature.txt"), "feature\n").unwrap();
    run_git(&seed, ["add", "feature.txt"]);
    run_git(&seed, ["commit", "-m", "feature"]);
    let feature_commit = run_git(&seed, ["rev-parse", "HEAD"]).trim().to_string();
    run_git(&seed, ["push", "origin", source_branch]);

    run_git(
        temp.path(),
        [
            "clone",
            "-b",
            "main",
            bare.to_str().unwrap(),
            worktree.to_str().unwrap(),
        ],
    );

    let bitbucket_url = "https://bitbucket.org/acme/widgets.git";
    let rewritten_url = format!("file://{}", bare.canonicalize().unwrap().display());
    let rewrite_key = format!("url.{rewritten_url}.insteadOf");
    run_git(&worktree, ["remote", "set-url", "origin", bitbucket_url]);
    run_git(&worktree, ["config", rewrite_key.as_str(), bitbucket_url]);

    CheckoutRepo {
        _temp: temp,
        worktree,
        feature_commit,
    }
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
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

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
    assert!(stdout.contains("request-changes"));
    assert!(stdout.contains("remove-request-changes"));
    assert!(stdout.contains("checkout"));
}

#[test]
fn pr_help_lists_api_aligned_commands() {
    let output = bb_command()
        .args(["pr", "--help"])
        .output()
        .expect("command should run");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("get"));
    assert!(stdout.contains("update"));
    assert!(stdout.contains("request-changes"));
    assert!(stdout.contains("remove-request-changes"));
    assert!(stdout.contains("statuses"));
    assert!(stdout.contains("activity"));
    assert!(stdout.contains("checkout"));
}

#[test]
fn pr_get_json_reads_config_and_calls_server() {
    let server = MockServer::start();
    let pr = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pullrequests/42");
        then.json_body(json!({
            "id": 42,
            "state": "OPEN",
            "title": "Add widget support",
            "source": { "branch": { "name": "feature/widgets" } },
            "destination": { "branch": { "name": "main" } }
        }));
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pr",
            "get",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--id",
            "42",
            "--output",
            "json",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body["title"], "Add widget support");
    pr.assert();
}

#[test]
fn pr_diff_text_reads_config_and_calls_server() {
    let server = MockServer::start();
    let diff = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pullrequests/42/diff");
        then.body("diff --git a/src/lib.rs b/src/lib.rs\n");
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pr",
            "diff",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--id",
            "42",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(stdout, "diff --git a/src/lib.rs b/src/lib.rs\n");
    diff.assert();
}

#[test]
fn pr_checkout_json_fetches_and_checks_out_source_branch() {
    let server = MockServer::start();
    let pr = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pullrequests/42");
        then.json_body(json!({
            "id": 42,
            "state": "OPEN",
            "title": "Add widget support",
            "source": {
                "branch": { "name": "feature/pr-42" },
                "repository": { "full_name": "acme/widgets" }
            },
            "destination": {
                "branch": { "name": "main" },
                "repository": { "full_name": "acme/widgets" }
            }
        }));
    });

    let repo = setup_checkout_repo("feature/pr-42");
    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pr",
            "checkout",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--id",
            "42",
            "--output",
            "json",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .current_dir(&repo.worktree)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body["id"], 42);
    assert_eq!(body["branch"], "feature/pr-42");
    assert_eq!(body["source_branch"], "feature/pr-42");
    assert_eq!(body["ref"], "refs/bb/pr/42");
    assert_eq!(body["forced"], false);
    assert_eq!(
        run_git(&repo.worktree, ["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        "feature/pr-42"
    );
    assert_eq!(
        run_git(&repo.worktree, ["rev-parse", "HEAD"]).trim(),
        repo.feature_commit
    );
    pr.assert();
}

#[test]
fn pr_checkout_without_force_rejects_conflicting_local_branch() {
    let server = MockServer::start();
    let pr = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pullrequests/42");
        then.json_body(json!({
            "id": 42,
            "state": "OPEN",
            "title": "Add widget support",
            "source": {
                "branch": { "name": "feature/pr-42" },
                "repository": { "full_name": "acme/widgets" }
            }
        }));
    });

    let repo = setup_checkout_repo("feature/pr-42");
    run_git(&repo.worktree, ["checkout", "-b", "feature/pr-42"]);
    run_git(&repo.worktree, ["checkout", "main"]);

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pr",
            "checkout",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--id",
            "42",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .current_dir(&repo.worktree)
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("rerun with --force"));
    pr.assert();
}

#[test]
fn pr_checkout_force_replaces_conflicting_local_branch() {
    let server = MockServer::start();
    let pr = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pullrequests/42");
        then.json_body(json!({
            "id": 42,
            "state": "OPEN",
            "title": "Add widget support",
            "source": {
                "branch": { "name": "feature/pr-42" },
                "repository": { "full_name": "acme/widgets" }
            }
        }));
    });

    let repo = setup_checkout_repo("feature/pr-42");
    run_git(&repo.worktree, ["checkout", "-b", "feature/pr-42"]);
    run_git(&repo.worktree, ["checkout", "main"]);

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pr",
            "checkout",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--id",
            "42",
            "--force",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .current_dir(&repo.worktree)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Checked out PR #42 to branch feature/pr-42"));
    assert_eq!(
        run_git(&repo.worktree, ["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        "feature/pr-42"
    );
    assert_eq!(
        run_git(&repo.worktree, ["rev-parse", "HEAD"]).trim(),
        repo.feature_commit
    );
    pr.assert();
}

#[test]
fn pr_checkout_fork_error_uses_json_envelope() {
    let server = MockServer::start();
    let pr = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pullrequests/42");
        then.json_body(json!({
            "id": 42,
            "state": "OPEN",
            "title": "Add widget support",
            "source": {
                "branch": { "name": "feature/pr-42" },
                "repository": { "full_name": "fork/widgets" }
            }
        }));
    });

    let repo = setup_checkout_repo("feature/pr-42");
    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pr",
            "checkout",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--id",
            "42",
            "--output",
            "json",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .current_dir(&repo.worktree)
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body["error"]["code"], "invalid_input");
    assert_eq!(
        body["error"]["message"],
        "fork pull requests are not supported by bb pr checkout yet"
    );
    pr.assert();
}

#[test]
fn pr_unapprove_json_emits_synthetic_envelope() {
    let server = MockServer::start();
    let unapprove = server.mock(|when, then| {
        when.method(DELETE)
            .path("/2.0/repositories/acme/widgets/pullrequests/42/approve");
        then.status(204);
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pr",
            "unapprove",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--id",
            "42",
            "--output",
            "json",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body["id"], 42);
    assert_eq!(body["action"], "Removed approval from");
    assert_eq!(body["ok"], true);
    unapprove.assert();
}

#[test]
fn pr_statuses_json_reads_config_and_calls_server() {
    let server = MockServer::start();
    let statuses = server.mock(|when, then| {
        when.method(GET)
            .path("/2.0/repositories/acme/widgets/pullrequests/42/statuses");
        then.json_body(json!({
            "values": [
                {
                    "key": "build",
                    "state": "SUCCESSFUL",
                    "name": "CI"
                }
            ]
        }));
    });

    let temp = tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    write_config(&config_path, &format!("{}/2.0", server.base_url()));

    let output = bb_command()
        .args([
            "pr",
            "statuses",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--id",
            "42",
            "--output",
            "json",
        ])
        .env("BB_CONFIG_PATH", &config_path)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body[0]["key"], "build");
    assert_eq!(body[0]["state"], "SUCCESSFUL");
    statuses.assert();
}

#[test]
fn pr_comment_missing_content_emits_json_error() {
    let output = bb_command()
        .args([
            "pr",
            "comment",
            "--workspace",
            "acme",
            "--repo",
            "widgets",
            "--id",
            "42",
            "--output",
            "json",
        ])
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be json");
    assert_eq!(body["error"]["code"], "invalid_input");
    assert_eq!(body["error"]["message"], "--content is required");
}
