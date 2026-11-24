use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use crate::error::CliError;

pub fn resolve_repo_target(
    workspace_value: Option<&str>,
    repo_value: Option<&str>,
    require_repo: bool,
) -> Result<(String, String), CliError> {
    let mut workspace = match workspace_value {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(CliError::InvalidInput(
                    "--workspace must not be empty".to_string(),
                ));
            }
            trimmed.to_string()
        }
        None => String::new(),
    };
    let mut repo = match repo_value {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(CliError::InvalidInput(
                    "--repo must not be empty".to_string(),
                ));
            }
            trimmed.to_string()
        }
        None => String::new(),
    };

    if workspace.is_empty() || (require_repo && repo.is_empty()) {
        if let Ok((inferred_workspace, inferred_repo)) = infer_bitbucket_repo_from_git(None) {
            if workspace.is_empty() {
                workspace = inferred_workspace;
            }
            if repo.is_empty() {
                repo = inferred_repo;
            }
        }
    }

    if workspace.is_empty() {
        return Err(CliError::InvalidInput(
            "--workspace is required".to_string(),
        ));
    }
    validate_target_segment("--workspace", &workspace)?;
    if require_repo && repo.is_empty() {
        return Err(CliError::InvalidInput("--repo is required".to_string()));
    }
    if !repo.is_empty() {
        validate_target_segment("--repo", &repo)?;
    }

    Ok((workspace, repo))
}

/// Workspace and repository slugs/UUIDs are interpolated into URL paths, where reqwest::Url
/// treats '\' as '/' and resolves dot segments. Allow only characters that cannot alter the path.
fn validate_target_segment(flag_name: &str, value: &str) -> Result<(), CliError> {
    let allowed =
        |ch: char| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '~' | '{' | '}');
    if !value.chars().all(allowed) || value == "." || value == ".." {
        return Err(CliError::InvalidInput(format!(
            "{flag_name} may only contain ASCII letters, digits, '-', '_', '.', '~', '{{', '}}' and must not be '.' or '..': {value}"
        )));
    }
    Ok(())
}

pub fn infer_bitbucket_repo_from_git(dir: Option<&Path>) -> Result<(String, String), CliError> {
    let remote = run_git(dir, ["config", "--get", "remote.origin.url"])?;
    let remote = remote.trim();
    if remote.is_empty() {
        return Err(CliError::Git("remote.origin.url not set".to_string()));
    }

    parse_bitbucket_remote(remote)
        .ok_or_else(|| CliError::Git("origin remote is not a Bitbucket repository".to_string()))
}

pub fn parse_bitbucket_remote(remote: &str) -> Option<(String, String)> {
    let trimmed = remote.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.contains("://") {
        let url = reqwest::Url::parse(trimmed).ok()?;
        if url.host_str()?.eq_ignore_ascii_case("bitbucket.org") {
            return parse_bitbucket_path(url.path());
        }
        return None;
    }

    let (mut host, path) = trimmed.split_once(':')?;
    if let Some((_, rest)) = host.rsplit_once('@') {
        host = rest;
    }
    if !host.trim().eq_ignore_ascii_case("bitbucket.org") {
        return None;
    }
    parse_bitbucket_path(path)
}

pub fn parse_bitbucket_path(raw_path: &str) -> Option<(String, String)> {
    let trimmed = raw_path
        .trim()
        .trim_start_matches('/')
        .trim_end_matches('/');
    let mut parts = trimmed.split('/');
    let workspace = parts.next()?.trim();
    let repo = parts.next()?.trim().trim_end_matches(".git");
    if parts.next().is_some() || workspace.is_empty() || repo.is_empty() {
        return None;
    }
    Some((workspace.to_string(), repo.to_string()))
}

pub fn run_git<I, S>(dir: Option<&Path>, args: I) -> Result<String, CliError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new("git");
    command.arg("-c").arg("credential.helper=");
    command.args(args);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }

    let output = command
        .output()
        .map_err(|error| CliError::Git(error.to_string()))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let message = if stderr.is_empty() { stdout } else { stderr };
    Err(CliError::Git(message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_remote_supports_https_and_ssh() {
        assert_eq!(
            parse_bitbucket_remote("https://bitbucket.org/acme/app.git"),
            Some(("acme".to_string(), "app".to_string()))
        );
        assert_eq!(
            parse_bitbucket_remote("git@bitbucket.org:acme/app.git"),
            Some(("acme".to_string(), "app".to_string()))
        );
        assert_eq!(
            parse_bitbucket_remote("https://github.com/acme/app.git"),
            None
        );
    }

    #[test]
    fn resolve_repo_target_rejects_path_traversal_segments() {
        assert_eq!(
            resolve_repo_target(Some("acme"), Some("my.widgets_1"), true).unwrap(),
            ("acme".to_string(), "my.widgets_1".to_string())
        );
        assert_eq!(
            resolve_repo_target(Some("{ws-uuid}"), Some("{repo-uuid}"), true).unwrap(),
            ("{ws-uuid}".to_string(), "{repo-uuid}".to_string())
        );

        for bad in [
            "",
            "   ",
            "foo/../victim",
            r"foo\..\victim",
            "..",
            ".",
            "a?b",
            "a#b",
            "a%2Fb",
            "a b",
        ] {
            assert!(
                resolve_repo_target(Some("acme"), Some(bad), true).is_err(),
                "repo {bad} should be rejected"
            );
            assert!(
                resolve_repo_target(Some(bad), Some("widgets"), true).is_err(),
                "workspace {bad} should be rejected"
            );
        }
    }
}
