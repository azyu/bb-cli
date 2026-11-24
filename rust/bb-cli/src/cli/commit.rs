use bb_core::{
    CommitReportAction, CommitReportRequest, CommitRequest, CommitStatusAction,
    CommitStatusRequest, Request,
};
use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub(super) enum CommitCommands {
    /// Manage Code Insights reports on a commit
    Report {
        #[command(subcommand)]
        command: Option<ReportCommands>,
    },
    /// Manage build statuses on a commit
    Status {
        #[command(subcommand)]
        command: Option<StatusCommands>,
    },
}

#[derive(Debug, Subcommand)]
pub(super) enum ReportCommands {
    /// List reports on a commit
    List(ReportListArgs),
    /// Get a report
    Get(ReportGetArgs),
    /// Create or update a report from a JSON body
    Put(ReportPutArgs),
    /// Delete a report
    Delete(ReportDeleteArgs),
}

#[derive(Debug, Subcommand)]
pub(super) enum StatusCommands {
    /// List commit statuses
    List(StatusListArgs),
    /// Get a build status by key
    #[command(visible_alias = "view")]
    Get(StatusGetArgs),
    /// Create a build status on a commit
    Create(StatusCreateArgs),
    /// Update a build status by key
    #[command(visible_alias = "edit")]
    Update(StatusUpdateArgs),
}

#[derive(Debug, Args)]
pub(super) struct CommitTarget {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    /// Commit hash the resource belongs to
    #[arg(long, value_name = "SHA")]
    pub(super) commit: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct ReportListArgs {
    #[command(flatten)]
    pub(super) target: CommitTarget,
    #[arg(long, default_value = "table")]
    pub(super) output: String,
    #[arg(long)]
    pub(super) all: bool,
    #[arg(long)]
    pub(super) fields: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct ReportGetArgs {
    #[command(flatten)]
    pub(super) target: CommitTarget,
    /// Report id (external id or UUID)
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
    #[arg(long)]
    pub(super) fields: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct ReportPutArgs {
    #[command(flatten)]
    pub(super) target: CommitTarget,
    /// Report id (external id or UUID)
    #[arg(long)]
    pub(super) id: Option<String>,
    /// Report JSON body file ("-" for stdin)
    #[arg(long)]
    pub(super) input: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct ReportDeleteArgs {
    #[command(flatten)]
    pub(super) target: CommitTarget,
    /// Report id (external id or UUID)
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct StatusListArgs {
    #[command(flatten)]
    pub(super) target: CommitTarget,
    #[arg(long, default_value = "table")]
    pub(super) output: String,
    #[arg(long)]
    pub(super) all: bool,
    #[arg(long)]
    pub(super) q: Option<String>,
    #[arg(long)]
    pub(super) sort: Option<String>,
    #[arg(long)]
    pub(super) fields: Option<String>,
    #[arg(long)]
    pub(super) json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct StatusGetArgs {
    #[command(flatten)]
    pub(super) target: CommitTarget,
    /// Build status key
    #[arg(long)]
    pub(super) key: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
    #[arg(long)]
    pub(super) fields: Option<String>,
    #[arg(long)]
    pub(super) json_fields: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct StatusCreateArgs {
    #[command(flatten)]
    pub(super) target: CommitTarget,
    /// Build status key
    #[arg(long, conflicts_with = "input")]
    pub(super) key: Option<String>,
    /// Build status state: INPROGRESS, SUCCESSFUL, FAILED, or STOPPED
    #[arg(long, conflicts_with = "input")]
    pub(super) state: Option<String>,
    /// URL linking to the build result
    #[arg(long, conflicts_with = "input")]
    pub(super) url: Option<String>,
    /// Display name for the build status
    #[arg(long, conflicts_with = "input")]
    pub(super) name: Option<String>,
    /// Short description of the build status
    #[arg(long, conflicts_with = "input")]
    pub(super) description: Option<String>,
    /// Branch or ref name associated with the build
    #[arg(long, conflicts_with = "input")]
    pub(super) refname: Option<String>,
    /// Build status JSON body file ("-" for stdin)
    #[arg(long, conflicts_with_all = ["key", "state", "url", "name", "description", "refname"])]
    pub(super) input: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

#[derive(Debug, Args)]
pub(super) struct StatusUpdateArgs {
    #[command(flatten)]
    pub(super) target: CommitTarget,
    /// Build status key
    #[arg(long)]
    pub(super) key: Option<String>,
    /// Build status state: INPROGRESS, SUCCESSFUL, FAILED, or STOPPED
    #[arg(long, conflicts_with = "input")]
    pub(super) state: Option<String>,
    /// URL linking to the build result
    #[arg(long, conflicts_with = "input")]
    pub(super) url: Option<String>,
    /// Display name for the build status
    #[arg(long, conflicts_with = "input")]
    pub(super) name: Option<String>,
    /// Short description of the build status
    #[arg(long, conflicts_with = "input")]
    pub(super) description: Option<String>,
    /// Branch or ref name associated with the build
    #[arg(long, conflicts_with = "input")]
    pub(super) refname: Option<String>,
    /// Build status JSON body file ("-" for stdin)
    #[arg(long, conflicts_with_all = ["state", "url", "name", "description", "refname"])]
    pub(super) input: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

pub(super) fn map_request(command: Option<CommitCommands>, profile: Option<String>) -> Request {
    match command {
        None => Request::Commit(CommitRequest::Help),
        Some(CommitCommands::Report { command }) => map_report_request(command, profile),
        Some(CommitCommands::Status { command }) => map_status_request(command, profile),
    }
}

fn map_report_request(command: Option<ReportCommands>, profile: Option<String>) -> Request {
    let Some(command) = command else {
        return Request::Commit(CommitRequest::ReportHelp);
    };
    let (action, target, id, input, output, all, fields) = match command {
        ReportCommands::List(args) => (
            CommitReportAction::List,
            args.target,
            None,
            None,
            args.output,
            args.all,
            args.fields,
        ),
        ReportCommands::Get(args) => (
            CommitReportAction::Get,
            args.target,
            args.id,
            None,
            args.output,
            false,
            args.fields,
        ),
        ReportCommands::Put(args) => (
            CommitReportAction::Put,
            args.target,
            args.id,
            args.input,
            args.output,
            false,
            None,
        ),
        ReportCommands::Delete(args) => (
            CommitReportAction::Delete,
            args.target,
            args.id,
            None,
            args.output,
            false,
            None,
        ),
    };
    Request::Commit(CommitRequest::Report(CommitReportRequest {
        action,
        workspace: target.workspace,
        repo: target.repo,
        commit: target.commit,
        id,
        input,
        output,
        all,
        fields,
        profile,
    }))
}

fn map_status_request(command: Option<StatusCommands>, profile: Option<String>) -> Request {
    let Some(command) = command else {
        return Request::Commit(CommitRequest::StatusHelp);
    };
    let request = match command {
        StatusCommands::List(args) => CommitStatusRequest {
            action: CommitStatusAction::List,
            workspace: args.target.workspace,
            repo: args.target.repo,
            commit: args.target.commit,
            key: None,
            state: None,
            url: None,
            name: None,
            description: None,
            refname: None,
            input: None,
            output: args.output,
            all: args.all,
            q: args.q,
            sort: args.sort,
            fields: args.fields,
            json_fields: args.json_fields,
            profile,
        },
        StatusCommands::Get(args) => CommitStatusRequest {
            action: CommitStatusAction::Get,
            workspace: args.target.workspace,
            repo: args.target.repo,
            commit: args.target.commit,
            key: args.key,
            state: None,
            url: None,
            name: None,
            description: None,
            refname: None,
            input: None,
            output: args.output,
            all: false,
            q: None,
            sort: None,
            fields: args.fields,
            json_fields: args.json_fields,
            profile,
        },
        StatusCommands::Create(args) => CommitStatusRequest {
            action: CommitStatusAction::Create,
            workspace: args.target.workspace,
            repo: args.target.repo,
            commit: args.target.commit,
            key: args.key,
            state: args.state,
            url: args.url,
            name: args.name,
            description: args.description,
            refname: args.refname,
            input: args.input,
            output: args.output,
            all: false,
            q: None,
            sort: None,
            fields: None,
            json_fields: None,
            profile,
        },
        StatusCommands::Update(args) => CommitStatusRequest {
            action: CommitStatusAction::Update,
            workspace: args.target.workspace,
            repo: args.target.repo,
            commit: args.target.commit,
            key: args.key,
            state: args.state,
            url: args.url,
            name: args.name,
            description: args.description,
            refname: args.refname,
            input: args.input,
            output: args.output,
            all: false,
            q: None,
            sort: None,
            fields: None,
            json_fields: None,
            profile,
        },
    };
    Request::Commit(CommitRequest::Status(request))
}
