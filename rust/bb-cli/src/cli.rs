use std::ffi::OsString;

use bb_core::{
    ApiRequest, AuthLoginRequest, AuthLogoutRequest, AuthRequest, AuthStatusRequest,
    IssueCreateRequest, IssueListRequest, IssueRequest, IssueUpdateRequest, PipelineListRequest,
    PipelineRequest, PipelineRunRequest, PrCreateRequest, PrListRequest, PrMergeRequest, PrRequest,
    RepoListRequest, RepoRequest, Request, WikiGetRequest, WikiListRequest, WikiPutRequest,
    WikiRequest,
};
use clap::{Args, Parser, Subcommand};

const STDIN_TOKEN_SENTINEL: &str = bb_core::runtime::STDIN_TOKEN_SENTINEL;

#[derive(Debug, Parser)]
#[command(
    name = "bb",
    disable_version_flag = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[arg(short = 'v', long = "version", global = true)]
    pub version: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Auth {
        #[command(subcommand)]
        command: Option<AuthCommands>,
    },
    Api(ApiArgs),
    Repo {
        #[command(subcommand)]
        command: Option<RepoCommands>,
    },
    Pr {
        #[command(subcommand)]
        command: Option<PrCommands>,
    },
    Pipeline {
        #[command(subcommand)]
        command: Option<PipelineCommands>,
    },
    Issue {
        #[command(subcommand)]
        command: Option<IssueCommands>,
    },
    Wiki {
        #[command(subcommand)]
        command: Option<WikiCommands>,
    },
    Completion(CompletionArgs),
    Version,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    Login(AuthLoginArgs),
    Status(AuthStatusArgs),
    Logout(AuthLogoutArgs),
}

#[derive(Debug, Subcommand)]
pub enum RepoCommands {
    List(RepoListArgs),
}

#[derive(Debug, Subcommand)]
pub enum PrCommands {
    List(PrListArgs),
    Create(PrCreateArgs),
    Merge(PrMergeArgs),
}

#[derive(Debug, Subcommand)]
pub enum PipelineCommands {
    List(PipelineListArgs),
    Run(PipelineRunArgs),
}

#[derive(Debug, Subcommand)]
pub enum IssueCommands {
    List(IssueListArgs),
    Create(IssueCreateArgs),
    Update(IssueUpdateArgs),
}

#[derive(Debug, Subcommand)]
pub enum WikiCommands {
    List(WikiListArgs),
    Get(WikiGetArgs),
    Put(WikiPutArgs),
}

#[derive(Debug, Args)]
pub struct CompletionArgs {
    pub shell: Option<String>,
}

#[derive(Debug, Args)]
pub struct AuthLoginArgs {
    #[arg(long, default_value = "default")]
    pub profile: String,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long)]
    pub username: Option<String>,
    #[arg(long)]
    pub with_token: bool,
    #[arg(long)]
    pub base_url: Option<String>,
}

#[derive(Debug, Args)]
pub struct AuthStatusArgs {
    #[arg(long)]
    pub profile: Option<String>,
}

#[derive(Debug, Args)]
pub struct AuthLogoutArgs {
    #[arg(long)]
    pub profile: Option<String>,
}

#[derive(Debug, Args)]
pub struct ApiArgs {
    #[arg(long, default_value = "GET")]
    pub method: String,
    #[arg(long)]
    pub paginate: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub q: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Args)]
pub struct RepoListArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long, default_value = "table")]
    pub output: String,
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub q: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
}

#[derive(Debug, Args)]
pub struct PrListArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value = "table")]
    pub output: String,
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub state: Option<String>,
    #[arg(long)]
    pub q: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
}

#[derive(Debug, Args)]
pub struct PrCreateArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub source: Option<String>,
    #[arg(long)]
    pub destination: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub close_branch: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct PrMergeArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<String>,
    #[arg(long)]
    pub message: Option<String>,
    #[arg(long)]
    pub strategy: Option<String>,
    #[arg(long)]
    pub close_branch: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct PipelineListArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value = "table")]
    pub output: String,
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
}

#[derive(Debug, Args)]
pub struct PipelineRunArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub branch: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct IssueListArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long, default_value = "table")]
    pub output: String,
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub q: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
}

#[derive(Debug, Args)]
pub struct IssueCreateArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub content: Option<String>,
    #[arg(long)]
    pub state: Option<String>,
    #[arg(long)]
    pub kind: Option<String>,
    #[arg(long)]
    pub priority: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct IssueUpdateArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub id: Option<u64>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub content: Option<String>,
    #[arg(long)]
    pub state: Option<String>,
    #[arg(long)]
    pub kind: Option<String>,
    #[arg(long)]
    pub priority: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct WikiListArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "table")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct WikiGetArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub page: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

#[derive(Debug, Args)]
pub struct WikiPutArgs {
    #[arg(long)]
    pub workspace: Option<String>,
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub page: Option<String>,
    #[arg(long)]
    pub content: Option<String>,
    #[arg(long)]
    pub file: Option<String>,
    #[arg(long)]
    pub message: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, default_value = "text")]
    pub output: String,
}

pub fn parse_from<I, T>(args: I) -> Result<Request, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let normalized = normalize_args(args.into_iter().map(Into::into).collect());
    if normalized.len() == 2 && normalized[1].to_string_lossy() == "help" {
        return Ok(Request::RootHelp);
    }

    let cli = Cli::try_parse_from(normalized)?;
    Ok(map_request(cli))
}

fn normalize_args(args: Vec<OsString>) -> Vec<OsString> {
    let mut out = Vec::with_capacity(args.len());
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if index > 0 && arg.to_string_lossy() == "--token" {
            let next_is_value = args
                .get(index + 1)
                .map(|value| !value.to_string_lossy().starts_with('-'))
                .unwrap_or(false);
            if next_is_value {
                out.push(arg.clone());
            } else {
                out.push(OsString::from(format!("--token={STDIN_TOKEN_SENTINEL}")));
            }
            index += 1;
            continue;
        }
        out.push(arg.clone());
        index += 1;
    }
    out
}

fn map_request(cli: Cli) -> Request {
    if cli.version {
        return Request::Version;
    }

    match cli.command {
        None => Request::RootHelp,
        Some(Commands::Version) => Request::Version,
        Some(Commands::Completion(args)) => Request::Completion(args.shell),
        Some(Commands::Auth { command }) => Request::Auth(match command {
            None => AuthRequest::Help,
            Some(AuthCommands::Login(args)) => AuthRequest::Login(AuthLoginRequest {
                profile: args.profile,
                token: args.token,
                username: args.username,
                with_token: args.with_token,
                base_url: args.base_url,
            }),
            Some(AuthCommands::Status(args)) => AuthRequest::Status(AuthStatusRequest {
                profile: args.profile,
            }),
            Some(AuthCommands::Logout(args)) => AuthRequest::Logout(AuthLogoutRequest {
                profile: args.profile,
            }),
        }),
        Some(Commands::Api(args)) => Request::Api(ApiRequest {
            method: args.method,
            paginate: args.paginate,
            profile: args.profile,
            q: args.q,
            sort: args.sort,
            fields: args.fields,
            endpoint: args.endpoint,
        }),
        Some(Commands::Repo { command }) => Request::Repo(match command {
            None => RepoRequest::Help,
            Some(RepoCommands::List(args)) => RepoRequest::List(RepoListRequest {
                workspace: args.workspace,
                output: args.output,
                all: args.all,
                profile: args.profile,
                q: args.q,
                sort: args.sort,
                fields: args.fields,
            }),
        }),
        Some(Commands::Pr { command }) => Request::Pr(match command {
            None => PrRequest::Help,
            Some(PrCommands::List(args)) => PrRequest::List(PrListRequest {
                workspace: args.workspace,
                repo: args.repo,
                output: args.output,
                all: args.all,
                profile: args.profile,
                state: args.state,
                q: args.q,
                sort: args.sort,
                fields: args.fields,
            }),
            Some(PrCommands::Create(args)) => PrRequest::Create(PrCreateRequest {
                workspace: args.workspace,
                repo: args.repo,
                title: args.title,
                source: args.source,
                destination: args.destination,
                description: args.description,
                close_branch: args.close_branch,
                profile: args.profile,
                output: args.output,
            }),
            Some(PrCommands::Merge(args)) => PrRequest::Merge(PrMergeRequest {
                workspace: args.workspace,
                repo: args.repo,
                id: args.id,
                message: args.message,
                strategy: args.strategy,
                close_branch: args.close_branch,
                profile: args.profile,
                output: args.output,
            }),
        }),
        Some(Commands::Pipeline { command }) => Request::Pipeline(match command {
            None => PipelineRequest::Help,
            Some(PipelineCommands::List(args)) => PipelineRequest::List(PipelineListRequest {
                workspace: args.workspace,
                repo: args.repo,
                output: args.output,
                all: args.all,
                profile: args.profile,
                sort: args.sort,
                fields: args.fields,
            }),
            Some(PipelineCommands::Run(args)) => PipelineRequest::Run(PipelineRunRequest {
                workspace: args.workspace,
                repo: args.repo,
                branch: args.branch,
                profile: args.profile,
                output: args.output,
            }),
        }),
        Some(Commands::Issue { command }) => Request::Issue(match command {
            None => IssueRequest::Help,
            Some(IssueCommands::List(args)) => IssueRequest::List(IssueListRequest {
                workspace: args.workspace,
                repo: args.repo,
                output: args.output,
                all: args.all,
                profile: args.profile,
                q: args.q,
                sort: args.sort,
                fields: args.fields,
            }),
            Some(IssueCommands::Create(args)) => IssueRequest::Create(IssueCreateRequest {
                workspace: args.workspace,
                repo: args.repo,
                title: args.title,
                content: args.content,
                state: args.state,
                kind: args.kind,
                priority: args.priority,
                profile: args.profile,
                output: args.output,
            }),
            Some(IssueCommands::Update(args)) => IssueRequest::Update(IssueUpdateRequest {
                workspace: args.workspace,
                repo: args.repo,
                id: args.id,
                title: args.title,
                content: args.content,
                state: args.state,
                kind: args.kind,
                priority: args.priority,
                profile: args.profile,
                output: args.output,
            }),
        }),
        Some(Commands::Wiki { command }) => Request::Wiki(match command {
            None => WikiRequest::Help,
            Some(WikiCommands::List(args)) => WikiRequest::List(WikiListRequest {
                workspace: args.workspace,
                repo: args.repo,
                profile: args.profile,
                output: args.output,
            }),
            Some(WikiCommands::Get(args)) => WikiRequest::Get(WikiGetRequest {
                workspace: args.workspace,
                repo: args.repo,
                page: args.page,
                profile: args.profile,
                output: args.output,
            }),
            Some(WikiCommands::Put(args)) => WikiRequest::Put(WikiPutRequest {
                workspace: args.workspace,
                repo: args.repo,
                page: args.page,
                content: args.content,
                file: args.file,
                message: args.message,
                profile: args.profile,
                output: args.output,
            }),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_token_is_normalized_to_stdin_sentinel() {
        let request = parse_from(["bb", "auth", "login", "--token"]).expect("parse should succeed");
        let Request::Auth(AuthRequest::Login(request)) = request else {
            panic!("expected auth login");
        };
        assert_eq!(request.token.as_deref(), Some(STDIN_TOKEN_SENTINEL));
    }

    #[test]
    fn root_without_command_maps_to_root_help() {
        let request = parse_from(["bb"]).expect("parse should succeed");
        assert!(matches!(request, Request::RootHelp));
    }

    #[test]
    fn version_flag_maps_to_version_request() {
        let request = parse_from(["bb", "--version"]).expect("parse should succeed");
        assert!(matches!(request, Request::Version));
    }
}
