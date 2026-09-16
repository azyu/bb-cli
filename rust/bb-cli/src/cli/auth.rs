use bb_core::{
    AuthListRequest, AuthLoginRequest, AuthLogoutRequest, AuthRequest, AuthStatusRequest,
    AuthSwitchRequest, Request,
};
use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub(super) enum AuthCommands {
    /// Save a Bitbucket Cloud authentication profile
    Login(AuthLoginArgs),
    /// Show authentication profile status
    Status,
    /// Remove a saved authentication profile
    Logout,
    /// Set the active authentication profile
    Switch,
    /// List saved authentication profiles
    List(AuthListArgs),
}

#[derive(Debug, Args)]
pub(super) struct AuthLoginArgs {
    #[arg(long)]
    /// API token value, or read from stdin when omitted
    pub(super) token: Option<String>,
    #[arg(long)]
    /// Bitbucket username used with API tokens
    pub(super) username: Option<String>,
    #[arg(long)]
    /// Read the API token from stdin
    pub(super) with_token: bool,
    #[arg(long)]
    /// Bitbucket API base URL
    pub(super) base_url: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct AuthListArgs {
    #[arg(long, default_value = "table")]
    /// Output format
    pub(super) output: String,
}

pub(super) fn map_request(command: Option<AuthCommands>, profile: Option<String>) -> Request {
    Request::Auth(match command {
        None => AuthRequest::Help,
        Some(AuthCommands::Login(args)) => AuthRequest::Login(AuthLoginRequest {
            profile: profile.unwrap_or_else(|| "default".to_string()),
            token: args.token,
            username: args.username,
            with_token: args.with_token,
            base_url: args.base_url,
        }),
        Some(AuthCommands::Status) => AuthRequest::Status(AuthStatusRequest { profile }),
        Some(AuthCommands::Logout) => AuthRequest::Logout(AuthLogoutRequest { profile }),
        Some(AuthCommands::Switch) => AuthRequest::Switch(AuthSwitchRequest {
            profile: profile.unwrap_or_default(),
        }),
        Some(AuthCommands::List(args)) => AuthRequest::List(AuthListRequest {
            output: args.output,
        }),
    })
}
