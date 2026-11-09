use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use crate::error::CliError;

pub fn resolve_repo_target(
    workspace_value: Option<&str>,
    repo_value: Option<&str>,
    require_repo: bool,
) -> Result<(String, String), CliError> {
    let mut workspace = workspace_value.unwrap_or_default().trim().to_string();
    let mut repo = repo_value.unwrap_or_default().trim().to_string();

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
    if require_repo && repo.is_empty() {
        return Err(CliError::InvalidInput("--repo is required".to_string()));
    }

    Ok((workspace, repo))
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
}
