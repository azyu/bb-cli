use std::io::{BufRead, Write};

use crate::config;
use crate::config::Profile;
use crate::error::CliError;
use crate::render;
use crate::{
    AuthListRequest, AuthLoginRequest, AuthRequest, AuthStatusRequest, AuthSwitchRequest,
    ListOutput,
};

use super::STDIN_TOKEN_SENTINEL;
use super::support::parse_list_output;
use serde_json::{Value, json};

pub(super) fn handle_auth<R: BufRead, O: Write>(
    request: &AuthRequest,
    stdin: &mut R,
    stdout: &mut O,
) -> Result<(), CliError> {
    match request {
        AuthRequest::Help => write!(stdout, "{}", render::auth_usage()).map_err(CliError::from),
        AuthRequest::Login(request) => handle_auth_login(request, stdin, stdout),
        AuthRequest::Status(request) => handle_auth_status(request, stdout),
        AuthRequest::Logout(request) => handle_auth_logout(request, stdout),
        AuthRequest::Switch(request) => handle_auth_switch(request, stdout),
        AuthRequest::List(request) => handle_auth_list(request, stdout),
    }
}

fn handle_auth_switch<O: Write>(
    request: &AuthSwitchRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let mut config = config::load()?;
    let switched = config.switch_profile(&request.profile)?;
    config::save(&config)?;

    writeln!(stdout, "active profile: {switched:?}")?;
    Ok(())
}

fn handle_auth_list<O: Write>(request: &AuthListRequest, stdout: &mut O) -> Result<(), CliError> {
    let output = parse_list_output(&request.output)?;
    let config = config::load()?;
    if config.profiles.is_empty() {
        return Err(CliError::NotLoggedIn);
    }

    let rows: Vec<(&String, &Profile, bool, &'static str)> = config
        .profiles
        .iter()
        .map(|(name, profile)| {
            let mode = if profile.username.trim().is_empty() {
                "bearer"
            } else {
                "basic"
            };
            (name, profile, *name == config.current, mode)
        })
        .collect();

    match output {
        ListOutput::Json => {
            let values: Vec<Value> = rows
                .iter()
                .map(|(name, profile, active, mode)| {
                    json!({
                        "name": name,
                        "active": active,
                        "auth": mode,
                        "username": profile.username.trim(),
                        "base_url": profile.base_url,
                    })
                })
                .collect();
            render::print_json(stdout, &values)
        }
        ListOutput::Table => {
            let labels: Vec<String> = rows
                .iter()
                .map(|(_, profile, _, mode)| {
                    if *mode == "bearer" {
                        "bearer token".to_string()
                    } else {
                        format!("basic ({})", profile.username.trim())
                    }
                })
                .collect();
            let name_width = rows.iter().map(|(name, ..)| name.len()).max().unwrap_or(0);
            let auth_width = labels.iter().map(String::len).max().unwrap_or(0);

            for ((name, profile, active, _), auth) in rows.iter().zip(&labels) {
                let marker = if *active { "*" } else { " " };
                let base_url = &profile.base_url;
                writeln!(
                    stdout,
                    "{marker} {name:name_width$}  {auth:auth_width$}  {base_url}"
                )?;
            }
            Ok(())
        }
    }
}

fn handle_auth_login<R: BufRead, O: Write>(
    request: &AuthLoginRequest,
    stdin: &mut R,
    stdout: &mut O,
) -> Result<(), CliError> {
    let token = resolve_login_token(request, stdin)?;
    let username = request
        .username
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            std::env::var("BITBUCKET_USERNAME")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_default();

    let mut config = config::load()?;
    config.set_profile_with_auth(
        &request.profile,
        &username,
        &token,
        request.base_url.as_deref(),
    );
    config::save(&config)?;

    writeln!(stdout, "authenticated profile {:?}", request.profile)?;
    if username.is_empty() {
        writeln!(stdout, "auth mode: bearer token")?;
    } else {
        writeln!(stdout, "auth mode: basic ({username})")?;
    }
    Ok(())
}

fn handle_auth_status<O: Write>(
    request: &AuthStatusRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let config = config::load()?;
    let (profile, name) = config.active_profile(request.profile.as_deref())?;

    writeln!(stdout, "Profile: {name}")?;
    writeln!(stdout, "Base URL: {}", profile.base_url)?;
    if profile.username.trim().is_empty() {
        writeln!(stdout, "Auth: bearer token")?;
    } else {
        writeln!(stdout, "Auth: basic ({})", profile.username.trim())?;
    }
    if profile.token.trim().is_empty() {
        writeln!(stdout, "Token: not configured")?;
    } else {
        writeln!(stdout, "Token: configured")?;
    }
    Ok(())
}

fn handle_auth_logout<O: Write>(
    request: &crate::AuthLogoutRequest,
    stdout: &mut O,
) -> Result<(), CliError> {
    let mut config = config::load()?;
    if request
        .profile
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
        && config.current.trim().is_empty()
    {
        return Err(CliError::NotLoggedIn);
    }

    let (removed, ok) = config.remove_profile(request.profile.as_deref());
    if !ok {
        return if removed.trim().is_empty() {
            Err(CliError::NotLoggedIn)
        } else {
            Err(CliError::Config(format!("profile {:?} not found", removed)))
        };
    }

    config::save(&config)?;
    writeln!(stdout, "logged out profile {:?}", removed)?;
    if !config.current.trim().is_empty() {
        writeln!(stdout, "active profile: {:?}", config.current)?;
    }
    Ok(())
}
fn resolve_login_token<R: BufRead>(
    request: &AuthLoginRequest,
    stdin: &mut R,
) -> Result<String, CliError> {
    let token = request
        .token
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(token) = token {
        if token == STDIN_TOKEN_SENTINEL {
            return read_token_from_stdin(stdin);
        }
        return Ok(token.to_string());
    }
    if request.with_token {
        return read_token_from_stdin(stdin);
    }
    if let Ok(token) = std::env::var("BITBUCKET_TOKEN") {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }
    Err(CliError::InvalidInput(
        "token is required: use --token <value>, --with-token, or BITBUCKET_TOKEN".to_string(),
    ))
}

fn read_token_from_stdin<R: BufRead>(stdin: &mut R) -> Result<String, CliError> {
    let mut buffer = String::new();
    let bytes = stdin.read_line(&mut buffer)?;
    if bytes == 0 {
        return Err(CliError::InvalidInput(
            "no token provided on stdin".to_string(),
        ));
    }
    let token = buffer.trim().to_string();
    if token.is_empty() {
        return Err(CliError::InvalidInput(
            "no token provided on stdin".to_string(),
        ));
    }
    Ok(token)
}
