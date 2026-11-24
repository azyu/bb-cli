use std::io::{Read, Write};

use reqwest::Method;
use serde_json::{Value, json};

use crate::context;
use crate::error::CliError;
use crate::render;
use crate::{
    CommitReportAction, CommitReportRequest, CommitRequest, CommitStatusAction,
    CommitStatusRequest, ListOutput, WriteOutput,
};

use super::api::read_api_input_body_from;
use super::support::{
    STATUS_JSON_FIELDS, client_from_profile, collect_query, optional_trimmed, parse_json_fields,
    parse_list_output, parse_write_output, print_json_list, print_json_object, required_string,
    set_optional_string,
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
        CommitRequest::StatusHelp => {
            write!(stdout, "{}", render::commit_status_usage()).map_err(CliError::from)
        }
        CommitRequest::Status(request) => handle_commit_status(request, stdin, stdout),
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
        _ => Some(validate_path_segment("--id", request.id.as_deref())?),
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

fn handle_commit_status<R: Read, O: Write>(
    request: &CommitStatusRequest,
    stdin: &mut R,
    stdout: &mut O,
) -> Result<(), CliError> {
    // Validate every local input before touching config or the network.
    let commit = validate_commit(request.commit.as_deref())?;
    let base = |workspace: &str, repo: &str| {
        format!("/repositories/{workspace}/{repo}/commit/{commit}/statuses")
    };

    match request.action {
        CommitStatusAction::List => {
            let output = parse_list_output(&request.output)?;
            let json_fields = parse_json_fields(
                request.json_fields.as_deref(),
                output == ListOutput::Json,
                "bb commit status list",
                STATUS_JSON_FIELDS,
            )?;
            let (workspace, repo) = context::resolve_repo_target(
                request.workspace.as_deref(),
                request.repo.as_deref(),
                true,
            )?;
            let client = client_from_profile(request.profile.as_deref())?;
            let path = base(&workspace, &repo);
            let query = collect_query([
                ("q", request.q.as_deref()),
                ("sort", request.sort.as_deref()),
                ("fields", request.fields.as_deref()),
            ]);
            let values = if request.all {
                client.get_all_values(&path, &query)?
            } else {
                client.get_page(&path, &query)?.0
            };
            match output {
                ListOutput::Json => print_json_list(stdout, &values, json_fields.as_deref()),
                ListOutput::Table => {
                    write!(stdout, "{}", render::render_pr_statuses_table(&values))
                        .map_err(CliError::from)
                }
            }
        }
        CommitStatusAction::Get => {
            let output = parse_write_output(&request.output)?;
            let json_fields = parse_json_fields(
                request.json_fields.as_deref(),
                output == WriteOutput::Json,
                "bb commit status get",
                STATUS_JSON_FIELDS,
            )?;
            let key = validate_path_segment("--key", request.key.as_deref())?;
            let (workspace, repo) = context::resolve_repo_target(
                request.workspace.as_deref(),
                request.repo.as_deref(),
                true,
            )?;
            let client = client_from_profile(request.profile.as_deref())?;
            let query = collect_query([("fields", request.fields.as_deref())]);
            let value = client.request_value(
                Method::GET,
                &format!("{}/build/{key}", base(&workspace, &repo)),
                &query,
                None,
            )?;
            match output {
                WriteOutput::Json => print_json_object(stdout, &value, json_fields.as_deref()),
                WriteOutput::Text => write_status_text(stdout, &value, None),
            }
        }
        CommitStatusAction::Create => {
            let output = parse_write_output(&request.output)?;
            let body = build_status_write_body(request, true, stdin)?;
            let (workspace, repo) = context::resolve_repo_target(
                request.workspace.as_deref(),
                request.repo.as_deref(),
                true,
            )?;
            let client = client_from_profile(request.profile.as_deref())?;
            let value = client.request_value(
                Method::POST,
                &format!("{}/build", base(&workspace, &repo)),
                &[],
                Some(body),
            )?;
            match output {
                WriteOutput::Json => render::print_json(stdout, &value),
                WriteOutput::Text => write_status_text(stdout, &value, Some("Created")),
            }
        }
        CommitStatusAction::Update => {
            let output = parse_write_output(&request.output)?;
            let key = validate_path_segment("--key", request.key.as_deref())?;
            let body = build_status_write_body(request, false, stdin)?;
            let (workspace, repo) = context::resolve_repo_target(
                request.workspace.as_deref(),
                request.repo.as_deref(),
                true,
            )?;
            let client = client_from_profile(request.profile.as_deref())?;
            let value = client.request_value(
                Method::PUT,
                &format!("{}/build/{key}", base(&workspace, &repo)),
                &[],
                Some(body),
            )?;
            match output {
                WriteOutput::Json => render::print_json(stdout, &value),
                WriteOutput::Text => write_status_text(stdout, &value, Some("Updated")),
            }
        }
    }
}

fn build_status_write_body<R: Read>(
    request: &CommitStatusRequest,
    include_key: bool,
    stdin: &mut R,
) -> Result<Value, CliError> {
    if optional_trimmed(request.input.as_deref()).is_some() {
        let has_field_flags = (include_key && optional_trimmed(request.key.as_deref()).is_some())
            || optional_trimmed(request.state.as_deref()).is_some()
            || optional_trimmed(request.url.as_deref()).is_some()
            || optional_trimmed(request.name.as_deref()).is_some()
            || optional_trimmed(request.description.as_deref()).is_some()
            || optional_trimmed(request.refname.as_deref()).is_some();
        if has_field_flags {
            return Err(CliError::InvalidInput(
                "--input cannot be combined with status field flags".to_string(),
            ));
        }
        return read_api_input_body_from(request.input.as_deref(), stdin)?
            .ok_or_else(|| CliError::InvalidInput("--input is required".to_string()));
    }

    let state = normalize_status_state(request.state.as_deref())?;
    let url = required_string(
        "--url is required (or pass --input)",
        request.url.as_deref(),
    )?;
    let mut body = json!({
        "state": state,
        "url": url,
    });
    if include_key {
        let key = required_string(
            "--key is required (or pass --input)",
            request.key.as_deref(),
        )?;
        body["key"] = Value::String(key.to_string());
    }
    set_optional_string(&mut body, "name", request.name.as_deref());
    set_optional_string(&mut body, "description", request.description.as_deref());
    set_optional_string(&mut body, "refname", request.refname.as_deref());
    Ok(body)
}

fn normalize_status_state(value: Option<&str>) -> Result<String, CliError> {
    let value = required_string("--state is required (or pass --input)", value)?;
    let normalized = value.to_ascii_uppercase();
    match normalized.as_str() {
        "INPROGRESS" | "SUCCESSFUL" | "FAILED" | "STOPPED" => Ok(normalized),
        _ => Err(CliError::InvalidInput(
            "--state must be one of INPROGRESS, SUCCESSFUL, FAILED, STOPPED".to_string(),
        )),
    }
}

fn write_status_text<O: Write>(
    stdout: &mut O,
    value: &Value,
    action: Option<&str>,
) -> Result<(), CliError> {
    let key = render::string_field(value, &["key"]).unwrap_or("-");
    let state = render::string_field(value, &["state"]).unwrap_or("-");
    let name = render::string_field(value, &["name"]).filter(|name| !name.trim().is_empty());
    match (action, name) {
        (Some(action), Some(name)) => writeln!(stdout, "{action} status {key} ({state}): {name}")?,
        (Some(action), None) => writeln!(stdout, "{action} status {key} ({state})")?,
        (None, Some(name)) => writeln!(stdout, "Status {key} ({state}): {name}")?,
        (None, None) => writeln!(stdout, "Status {key} ({state})")?,
    }
    if let Some(url) = render::string_field(value, &["url"]).filter(|url| !url.trim().is_empty()) {
        writeln!(stdout, "URL: {url}")?;
    }
    if let Some(description) =
        render::string_field(value, &["description"]).filter(|desc| !desc.trim().is_empty())
    {
        writeln!(stdout, "Description: {description}")?;
    }
    if let Some(refname) =
        render::string_field(value, &["refname"]).filter(|refname| !refname.trim().is_empty())
    {
        writeln!(stdout, "Ref: {refname}")?;
    }
    Ok(())
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

/// Report ids and status keys are URL path segments, and reqwest::Url treats '\' as '/' and
/// resolves dot segments, so a blocklist can be bypassed (`..\..\widgets` would retarget a
/// request at the repository). Allow only characters that cannot change the path.
fn validate_path_segment<'a>(flag_name: &str, value: Option<&'a str>) -> Result<&'a str, CliError> {
    let id = required_string(&format!("{flag_name} is required"), value)?;
    let allowed =
        |ch: char| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '~' | '{' | '}');
    if !id.chars().all(allowed) || id == "." || id == ".." {
        return Err(CliError::InvalidInput(format!(
            "{flag_name} may only contain ASCII letters, digits, '-', '_', '.', '~', '{{', '}}' and must not be '.' or '..': {id} (use bb api for other values)"
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
    fn validate_path_segment_rejects_url_changing_characters() {
        assert_eq!(
            validate_path_segment("--id", Some("pr-risk-tier-trial")).unwrap(),
            "pr-risk-tier-trial"
        );
        assert_eq!(
            validate_path_segment("--id", Some("{3f2a-11}")).unwrap(),
            "{3f2a-11}"
        );
        for bad in [
            "a/b",
            "a?b",
            "a#b",
            "a%2Fb",
            "a b",
            r"..\..\widgets",
            r"a\b",
            ".",
            "..",
            "a:b",
        ] {
            assert!(
                validate_path_segment("--key", Some(bad)).is_err(),
                "{bad} should fail"
            );
        }
        assert!(validate_path_segment("--key", Some("  ")).is_err());
    }

    #[test]
    fn normalize_status_state_accepts_valid_states_case_insensitively() {
        assert_eq!(
            normalize_status_state(Some("successful")).unwrap(),
            "SUCCESSFUL"
        );
        assert_eq!(
            normalize_status_state(Some(" INPROGRESS ")).unwrap(),
            "INPROGRESS"
        );
        assert_eq!(normalize_status_state(Some("failed")).unwrap(), "FAILED");
        assert_eq!(normalize_status_state(Some("STOPPED")).unwrap(), "STOPPED");
        assert!(normalize_status_state(Some("PENDING")).is_err());
        assert!(normalize_status_state(None).is_err());
    }
}
