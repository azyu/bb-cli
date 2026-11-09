use std::io::{Read, Write};

use reqwest::Method;
use serde_json::{Value, json};

use crate::context;
use crate::error::CliError;
use crate::render;
use crate::{CommitReportAction, CommitReportRequest, CommitRequest, ListOutput, WriteOutput};

use super::api::read_api_input_body_from;
use super::support::{
    client_from_profile, collect_query, parse_list_output, parse_write_output, required_string,
};

pub(super) fn handle_commit<R: Read, O: Write>(
    request: &CommitRequest,
    stdin: &mut R,
    stdout: &mut O,
) -> Result<(), CliError> {
    match request {
        CommitRequest::Help => write!(stdout, "{}", render::commit_usage()).map_err(CliError::from),
        CommitRequest::ReportHelp => {
            write!(stdout, "{}", render::commit_report_usage()).map_err(CliError::from)
        }
        CommitRequest::Report(request) => handle_commit_report(request, stdin, stdout),
    }
}

fn handle_commit_report<R: Read, O: Write>(
    request: &CommitReportRequest,
    stdin: &mut R,
    stdout: &mut O,
) -> Result<(), CliError> {
    // Validate every local input before touching config or the network.
    let commit = validate_commit(request.commit.as_deref())?;
    let id = match request.action {
        CommitReportAction::List => None,
        _ => Some(validate_report_id(request.id.as_deref())?),
    };
    let body = match request.action {
        CommitReportAction::Put => Some(
            read_api_input_body_from(request.input.as_deref(), stdin)?
                .ok_or_else(|| CliError::InvalidInput("--input is required".to_string()))?,
        ),
        _ => None,
    };
    let (workspace, repo) =
        context::resolve_repo_target(request.workspace.as_deref(), request.repo.as_deref(), true)?;
    let base = format!("/repositories/{workspace}/{repo}/commit/{commit}/reports");
    let query = collect_query([("fields", request.fields.as_deref())]);

    match (request.action, id) {
        (CommitReportAction::List, _) => {
            let output = parse_list_output(&request.output)?;
            let client = client_from_profile(request.profile.as_deref())?;
            let values = if request.all {
                client.get_all_values(&base, &query)?
            } else {
                client.get_page(&base, &query)?.0
            };
            match output {
                ListOutput::Json => render::print_json(stdout, &values),
                ListOutput::Table => {
                    write!(stdout, "{}", render::render_commit_report_table(&values))
                        .map_err(CliError::from)
                }
            }
        }
        (CommitReportAction::Get, Some(id)) => {
            let output = parse_write_output(&request.output)?;
            let client = client_from_profile(request.profile.as_deref())?;
            let value = client.request_value(Method::GET, &format!("{base}/{id}"), &query, None)?;
            write_report(stdout, output, &value, None)
        }
        (CommitReportAction::Put, Some(id)) => {
            let output = parse_write_output(&request.output)?;
            let client = client_from_profile(request.profile.as_deref())?;
            let value = client.request_value(Method::PUT, &format!("{base}/{id}"), &[], body)?;
            write_report(stdout, output, &value, Some("Saved"))
        }
        (CommitReportAction::Delete, Some(id)) => {
            let output = parse_write_output(&request.output)?;
            let client = client_from_profile(request.profile.as_deref())?;
            client.request_text(Method::DELETE, &format!("{base}/{id}"), &[])?;
            match output {
                WriteOutput::Text => writeln!(stdout, "Deleted report {id} on commit {commit}")
                    .map_err(CliError::from),
                WriteOutput::Json => render::print_json(
                    stdout,
                    &json!({ "commit": commit, "id": id, "action": "deleted", "ok": true }),
                ),
            }
        }
        (_, None) => unreachable!("report id is validated for every non-list action"),
    }
}

fn write_report<O: Write>(
    stdout: &mut O,
    output: WriteOutput,
    value: &Value,
    action: Option<&str>,
) -> Result<(), CliError> {
    if output == WriteOutput::Json {
        return render::print_json(stdout, value);
    }
    let field = |key: &str| render::string_field(value, &[key]).unwrap_or("-");
    let id = render::string_field(value, &["external_id"]).unwrap_or_else(|| field("uuid"));
    match action {
        Some(action) => writeln!(stdout, "{action} report {id}: {}", field("title"))?,
        None => writeln!(stdout, "Report {id}: {}", field("title"))?,
    }
    writeln!(stdout, "Result: {}", field("result"))?;
    writeln!(stdout, "Type: {}", field("report_type"))?;
    writeln!(stdout, "Reporter: {}", field("reporter"))?;
    if let Some(link) = render::string_field(value, &["link"]).filter(|link| !link.is_empty()) {
        writeln!(stdout, "Link: {link}")?;
    }
    Ok(())
}

/// The commit is interpolated into the URL path, so only a hex hash is accepted.
fn validate_commit(value: Option<&str>) -> Result<&str, CliError> {
    let commit = required_string("--commit is required", value)?;
    if !commit.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(CliError::InvalidInput(format!(
            "--commit must be a hexadecimal commit hash: {commit}"
        )));
    }
    Ok(commit)
}

/// The report id is a path segment; reject characters that would change the request URL.
fn validate_report_id(value: Option<&str>) -> Result<&str, CliError> {
    let id = required_string("--id is required", value)?;
    if id
        .chars()
        .any(|ch| ch.is_whitespace() || matches!(ch, '/' | '?' | '#' | '%'))
    {
        return Err(CliError::InvalidInput(format!(
            "--id must not contain whitespace, '/', '?', '#', or '%': {id}"
        )));
    }
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_commit_accepts_hex_and_rejects_path_characters() {
        assert_eq!(validate_commit(Some(" 110daa97 ")).unwrap(), "110daa97");
        assert!(validate_commit(None).is_err());
        assert!(validate_commit(Some("main")).is_err());
        assert!(validate_commit(Some("abc/../x")).is_err());
    }

    #[test]
    fn validate_report_id_rejects_url_changing_characters() {
        assert_eq!(
            validate_report_id(Some("pr-risk-tier-trial")).unwrap(),
            "pr-risk-tier-trial"
        );
        for bad in ["a/b", "a?b", "a#b", "a%2Fb", "a b"] {
            assert!(validate_report_id(Some(bad)).is_err(), "{bad} should fail");
        }
        assert!(validate_report_id(Some("  ")).is_err());
    }
}
