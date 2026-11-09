use bb_core::{CommitReportAction, CommitReportRequest, CommitRequest, Request};
use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub(super) enum CommitCommands {
    /// Manage Code Insights reports on a commit
    Report {
        #[command(subcommand)]
        command: Option<ReportCommands>,
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

#[derive(Debug, Args)]
pub(super) struct ReportTarget {
    #[arg(long)]
    pub(super) workspace: Option<String>,
    #[arg(long)]
    pub(super) repo: Option<String>,
    /// Commit hash the reports belong to
    #[arg(long, value_name = "SHA")]
    pub(super) commit: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct ReportListArgs {
    #[command(flatten)]
    pub(super) target: ReportTarget,
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
    pub(super) target: ReportTarget,
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
    pub(super) target: ReportTarget,
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
    pub(super) target: ReportTarget,
    /// Report id (external id or UUID)
    #[arg(long)]
    pub(super) id: Option<String>,
    #[arg(long, default_value = "text")]
    pub(super) output: String,
}

pub(super) fn map_request(command: Option<CommitCommands>, profile: Option<String>) -> Request {
    let Some(CommitCommands::Report { command }) = command else {
        return Request::Commit(CommitRequest::Help);
    };
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
