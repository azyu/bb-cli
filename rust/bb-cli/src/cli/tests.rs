use bb_core::{
    AuthRequest, CommitRequest, CommitStatusAction, PipelineRequest, PrDeclineRequest, PrRequest,
    PrRequestChangesRequest, PrStatusesRequest, PrUpdateRequest, Request,
};

use super::*;

#[test]
fn global_repository_selector_populates_repo_target() {
    let request = parse_from(["bb", "pr", "get", "42", "-R", "acme/widgets"])
        .expect("repository selector should parse");
    let Request::Pr(PrRequest::Get(request)) = request else {
        panic!("expected pr get");
    };
    assert_eq!(request.workspace.as_deref(), Some("acme"));
    assert_eq!(request.repo.as_deref(), Some("widgets"));
}

#[test]
fn global_repository_selector_rejects_invalid_or_conflicting_inputs() {
    assert!(parse_from(["bb", "pr", "get", "42", "-R", "invalid"]).is_err());
    assert!(parse_from(["bb", "pipeline", "stop", "123", "-R", "acme/ "]).is_err());
    assert!(parse_from(["bb", "pipeline", "stop", "123", "-R", " /widgets"]).is_err());
    assert!(parse_from(["bb", "pipeline", "stop", "123", "-R", "ac me/widgets"]).is_err());
    assert!(
        parse_from([
            "bb",
            "pr",
            "get",
            "42",
            "-R",
            "acme/widgets",
            "--workspace",
            "other",
        ])
        .is_err()
    );
    assert!(parse_from(["bb", "api", "-R", "acme/widgets", "user"]).is_err());
}

#[test]
fn bare_token_is_normalized_to_stdin_sentinel() {
    let request = parse_from(["bb", "auth", "login", "--token"]).expect("parse should succeed");
    let Request::Auth(AuthRequest::Login(request)) = request else {
        panic!("expected auth login");
    };
    assert_eq!(request.token.as_deref(), Some(STDIN_TOKEN_SENTINEL));
}

#[test]
fn profile_flag_is_accepted_before_or_after_the_subcommand() {
    let prefix = parse_from(["bb", "--profile", "work", "pr", "list"]).expect("prefix parses");
    let suffix = parse_from(["bb", "pr", "list", "--profile", "work"]).expect("suffix parses");

    for (request, label) in [(prefix, "prefix"), (suffix, "suffix")] {
        let Request::Pr(PrRequest::List(request)) = request else {
            panic!("expected pr list for {label} form");
        };
        assert_eq!(
            request.profile.as_deref(),
            Some("work"),
            "{label} form should carry the profile"
        );
    }

    // The global flag reaches every command group, not just `pr`.
    let request = parse_from(["bb", "--profile", "work", "api", "user"]).expect("api parses");
    let Request::Api(request) = request else {
        panic!("expected api");
    };
    assert_eq!(request.profile.as_deref(), Some("work"));

    // Omitting it still means "use the active profile".
    let request = parse_from(["bb", "pr", "list"]).expect("parse should succeed");
    let Request::Pr(PrRequest::List(request)) = request else {
        panic!("expected pr list");
    };
    assert_eq!(request.profile, None);
}

#[test]
fn global_profile_drives_auth_subcommands() {
    // `auth login` keeps its "default" fallback when the flag is absent.
    let request =
        parse_from(["bb", "auth", "login", "--token", "t"]).expect("parse should succeed");
    let Request::Auth(AuthRequest::Login(request)) = request else {
        panic!("expected auth login");
    };
    assert_eq!(request.profile, "default");

    let request = parse_from(["bb", "--profile", "work", "auth", "login", "--token", "t"])
        .expect("parse should succeed");
    let Request::Auth(AuthRequest::Login(request)) = request else {
        panic!("expected auth login");
    };
    assert_eq!(request.profile, "work");

    // `auth switch` accepts either position and still requires a value.
    for args in [
        vec!["bb", "auth", "switch", "--profile", "work"],
        vec!["bb", "--profile", "work", "auth", "switch"],
    ] {
        let request = parse_from(args.clone()).expect("parse should succeed");
        let Request::Auth(AuthRequest::Switch(request)) = request else {
            panic!("expected auth switch for {args:?}");
        };
        assert_eq!(request.profile, "work");
    }
}

#[test]
fn auth_switch_requires_profile_and_list_takes_none() {
    let request =
        parse_from(["bb", "auth", "switch", "--profile", "work"]).expect("parse should succeed");
    let Request::Auth(AuthRequest::Switch(request)) = request else {
        panic!("expected auth switch");
    };
    assert_eq!(request.profile, "work");

    assert!(
        parse_from(["bb", "auth", "switch"]).is_err(),
        "switch without --profile must be a parse error"
    );

    let request = parse_from(["bb", "auth", "list"]).expect("parse should succeed");
    let Request::Auth(AuthRequest::List(request)) = request else {
        panic!("expected auth list");
    };
    assert_eq!(request.output, "table", "list defaults to table output");

    let request =
        parse_from(["bb", "auth", "list", "--output", "json"]).expect("parse should succeed");
    let Request::Auth(AuthRequest::List(request)) = request else {
        panic!("expected auth list");
    };
    assert_eq!(request.output, "json");
}

#[test]
fn root_without_command_maps_to_root_help() {
    let request = parse_from(["bb"]).expect("parse should succeed");
    assert!(matches!(request, Request::RootHelp));
}

#[test]
fn root_help_flag_maps_to_root_help() {
    let request = parse_from(["bb", "--help"]).expect("parse should succeed");
    assert!(matches!(request, Request::RootHelp));
}

#[test]
fn version_flag_maps_to_version_request() {
    let request = parse_from(["bb", "--version"]).expect("parse should succeed");
    assert!(matches!(request, Request::Version));
}

#[test]
fn api_maps_input_flag() {
    let request = parse_from([
        "bb",
        "api",
        "--method",
        "POST",
        "--input",
        "-",
        "repositories/acme/widgets/pullrequests/42/comments",
    ])
    .expect("parse should succeed");
    let Request::Api(request) = request else {
        panic!("expected api request");
    };
    assert_eq!(request.method, "POST");
    assert_eq!(request.input.as_deref(), Some("-"));
    assert_eq!(
        request.endpoint.as_deref(),
        Some("repositories/acme/widgets/pullrequests/42/comments")
    );
}

#[test]
fn pr_list_maps_positive_limit_and_rejects_conflicts() {
    let request =
        parse_from(["bb", "pr", "list", "--limit", "25"]).expect("positive limit should parse");
    let Request::Pr(PrRequest::List(request)) = request else {
        panic!("expected pr list");
    };
    assert_eq!(request.limit, Some(25));

    assert!(parse_from(["bb", "pr", "list", "--limit", "0"]).is_err());
    assert!(parse_from(["bb", "pr", "list", "--limit", "25", "--all"]).is_err());
}

#[test]
fn pr_get_maps_to_get_request() {
    let request = parse_from(["bb", "pr", "get", "--id", "42"]).expect("parse should succeed");
    let Request::Pr(PrRequest::Get(request)) = request else {
        panic!("expected pr get");
    };
    assert_eq!(request.id.as_deref(), Some("42"));
    assert_eq!(request.output, "text");
}

#[test]
fn pr_get_maps_positional_id_to_get_request() {
    let request = parse_from(["bb", "pr", "get", "42"]).expect("parse should succeed");
    let Request::Pr(PrRequest::Get(request)) = request else {
        panic!("expected pr get");
    };
    assert_eq!(request.id.as_deref(), Some("42"));
}

#[test]
fn pr_view_alias_maps_to_get_request() {
    let request = parse_from(["bb", "pr", "view", "42"]).expect("parse should succeed");
    let Request::Pr(PrRequest::Get(request)) = request else {
        panic!("expected pr get");
    };
    assert_eq!(request.id.as_deref(), Some("42"));
}

#[test]
fn pr_get_maps_json_fields() {
    let request = parse_from([
        "bb",
        "pr",
        "get",
        "42",
        "--output",
        "json",
        "--json-fields",
        "id,title",
    ])
    .expect("parse should succeed");
    let Request::Pr(PrRequest::Get(request)) = request else {
        panic!("expected pr get");
    };
    assert_eq!(request.json_fields.as_deref(), Some("id,title"));
}

#[test]
fn pr_edit_alias_maps_to_update_request() {
    let request = parse_from(["bb", "pr", "edit", "--id", "42"]).expect("parse should succeed");
    assert!(matches!(
        request,
        Request::Pr(PrRequest::Update(PrUpdateRequest { id: Some(id), .. })) if id == "42"
    ));
}

#[test]
fn pr_request_changes_maps_to_request_changes_request() {
    let request =
        parse_from(["bb", "pr", "request-changes", "--id", "42"]).expect("parse should succeed");
    assert!(matches!(
        request,
        Request::Pr(PrRequest::RequestChanges(PrRequestChangesRequest {
            id: Some(id),
            ..
        })) if id == "42"
    ));
}

#[test]
fn pr_close_alias_maps_to_decline_request() {
    let request = parse_from(["bb", "pr", "close", "--id", "42"]).expect("parse should succeed");
    assert!(matches!(
        request,
        Request::Pr(PrRequest::Decline(PrDeclineRequest { id: Some(id), .. })) if id == "42"
    ));
}

#[test]
fn pr_checks_alias_maps_to_statuses_request() {
    let request = parse_from(["bb", "pr", "checks", "--id", "42"]).expect("parse should succeed");
    assert!(matches!(
        request,
        Request::Pr(PrRequest::Statuses(PrStatusesRequest { id: Some(id), .. })) if id == "42"
    ));
}

#[test]
fn pr_create_maps_body_alias_to_description() {
    let request = parse_from([
        "bb",
        "pr",
        "create",
        "--title",
        "title",
        "--source",
        "feature",
        "--destination",
        "main",
        "--body",
        "description",
    ])
    .expect("parse should succeed");
    let Request::Pr(PrRequest::Create(request)) = request else {
        panic!("expected pr create");
    };
    assert_eq!(request.description.as_deref(), Some("description"));
}

#[test]
fn pr_comment_maps_content_and_output() {
    let request = parse_from([
        "bb",
        "pr",
        "comment",
        "--id",
        "42",
        "--body",
        "needs changes",
        "--output",
        "json",
    ])
    .expect("parse should succeed");
    let Request::Pr(PrRequest::Comment(request)) = request else {
        panic!("expected pr comment");
    };
    assert_eq!(request.id.as_deref(), Some("42"));
    assert_eq!(request.content.as_deref(), Some("needs changes"));
    assert_eq!(request.output, "json");
}

#[test]
fn pr_comment_update_maps_comment_id_content_and_output() {
    let request = parse_from([
        "bb",
        "pr",
        "comment-update",
        "42",
        "--comment-id",
        "7",
        "--content",
        "updated body",
        "--output",
        "json",
    ])
    .expect("parse should succeed");
    let Request::Pr(PrRequest::CommentUpdate(request)) = request else {
        panic!("expected pr comment update");
    };
    assert_eq!(request.id.as_deref(), Some("42"));
    assert_eq!(request.comment_id.as_deref(), Some("7"));
    assert_eq!(request.content.as_deref(), Some("updated body"));
    assert_eq!(request.output, "json");
}

#[test]
fn pr_comments_rejects_positional_and_flag_id_together() {
    let error =
        parse_from(["bb", "pr", "comments", "42", "--id", "43"]).expect_err("parse should fail");
    assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
}

#[test]
fn pr_comments_maps_comment_id() {
    let request = parse_from([
        "bb",
        "pr",
        "comments",
        "42",
        "--comment-id",
        "7",
        "--output",
        "json",
    ])
    .expect("parse should succeed");
    let Request::Pr(PrRequest::Comments(request)) = request else {
        panic!("expected pr comments");
    };
    assert_eq!(request.id.as_deref(), Some("42"));
    assert_eq!(request.comment_id.as_deref(), Some("7"));
    assert_eq!(request.output, "json");
}

#[test]
fn pipeline_get_maps_uuid_and_output() {
    let request = parse_from([
        "bb",
        "pipeline",
        "get",
        "--uuid",
        "{1234}",
        "--output",
        "json",
        "--json-fields",
        "uuid,state",
    ])
    .expect("parse should succeed");
    let Request::Pipeline(PipelineRequest::Get(request)) = request else {
        panic!("expected pipeline get");
    };
    assert_eq!(request.uuid.as_deref(), Some("{1234}"));
    assert_eq!(request.output, "json");
    assert_eq!(request.json_fields.as_deref(), Some("uuid,state"));
}

#[test]
fn pipeline_get_maps_build_and_output() {
    let request = parse_from(["bb", "pipeline", "get", "--build", "17", "--output", "json"])
        .expect("parse should succeed");
    let Request::Pipeline(PipelineRequest::Get(request)) = request else {
        panic!("expected pipeline get");
    };
    assert_eq!(request.build.as_deref(), Some("17"));
    assert_eq!(request.output, "json");
}

#[test]
fn pipeline_log_maps_step_and_output() {
    let request = parse_from([
        "bb", "pipeline", "log", "--uuid", "{pipe}", "--step", "{step}",
    ])
    .expect("parse should succeed");
    let Request::Pipeline(PipelineRequest::Log(request)) = request else {
        panic!("expected pipeline log");
    };
    assert_eq!(request.uuid.as_deref(), Some("{pipe}"));
    assert_eq!(request.step.as_deref(), Some("{step}"));
    assert_eq!(request.output, "text");
}

#[test]
fn pipeline_get_maps_positional_selector() {
    let request = parse_from(["bb", "pipeline", "get", "14588", "--output", "json"])
        .expect("parse should succeed");
    let Request::Pipeline(PipelineRequest::Get(request)) = request else {
        panic!("expected pipeline get");
    };
    assert_eq!(request.positional_selector.as_deref(), Some("14588"));
    assert_eq!(request.uuid, None);
    assert_eq!(request.build, None);
}

#[test]
fn pipeline_steps_maps_positional_selector() {
    let request = parse_from(["bb", "pipeline", "steps", "14588", "--output", "json"])
        .expect("parse should succeed");
    let Request::Pipeline(PipelineRequest::Steps(request)) = request else {
        panic!("expected pipeline steps");
    };
    assert_eq!(request.positional_selector.as_deref(), Some("14588"));
}

#[test]
fn pipeline_log_maps_positional_selector() {
    let request = parse_from(["bb", "pipeline", "log", "14588", "--step", "{step}"])
        .expect("parse should succeed");
    let Request::Pipeline(PipelineRequest::Log(request)) = request else {
        panic!("expected pipeline log");
    };
    assert_eq!(request.positional_selector.as_deref(), Some("14588"));
    assert_eq!(request.step.as_deref(), Some("{step}"));
}

#[test]
fn pipeline_get_rejects_uuid_and_build_together() {
    let error = parse_from(["bb", "pipeline", "get", "--uuid", "{1234}", "--build", "17"])
        .expect_err("parse should fail");
    assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
}

#[test]
fn pipeline_stop_and_cancel_alias_map_selector_and_repo() {
    for subcommand in ["stop", "cancel"] {
        let request = parse_from([
            "bb",
            "pipeline",
            subcommand,
            "14588",
            "-R",
            "acme/widgets",
            "--output",
            "json",
        ])
        .expect("parse should succeed");
        let Request::Pipeline(PipelineRequest::Stop(request)) = request else {
            panic!("expected pipeline stop for {subcommand}");
        };
        assert_eq!(request.positional_selector.as_deref(), Some("14588"));
        assert_eq!(request.workspace.as_deref(), Some("acme"));
        assert_eq!(request.repo.as_deref(), Some("widgets"));
        assert_eq!(request.output, "json");
    }

    let error = parse_from(["bb", "pipeline", "stop", "14588", "--build", "14588"])
        .expect_err("positional + --build must conflict");
    assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
}

#[test]
fn commit_status_subcommands_and_gh_aliases_parse() {
    let request = parse_from([
        "bb",
        "commit",
        "status",
        "view",
        "-R",
        "acme/widgets",
        "--commit",
        "110daa97",
        "--key",
        "ci-build",
        "--output",
        "json",
        "--json-fields",
        "key,state",
    ])
    .expect("commit status view alias should parse");
    let Request::Commit(CommitRequest::Status(request)) = request else {
        panic!("expected commit status");
    };
    assert_eq!(request.action, CommitStatusAction::Get);
    assert_eq!(request.workspace.as_deref(), Some("acme"));
    assert_eq!(request.repo.as_deref(), Some("widgets"));
    assert_eq!(request.commit.as_deref(), Some("110daa97"));
    assert_eq!(request.key.as_deref(), Some("ci-build"));
    assert_eq!(request.json_fields.as_deref(), Some("key,state"));

    let request = parse_from([
        "bb",
        "commit",
        "status",
        "edit",
        "-R",
        "acme/widgets",
        "--commit",
        "110daa97",
        "--key",
        "ci-build",
        "--state",
        "FAILED",
        "--url",
        "https://ci.example.com/123",
    ])
    .expect("commit status edit alias should parse");
    let Request::Commit(CommitRequest::Status(request)) = request else {
        panic!("expected commit status");
    };
    assert_eq!(request.action, CommitStatusAction::Update);
    assert_eq!(request.state.as_deref(), Some("FAILED"));
    assert_eq!(request.url.as_deref(), Some("https://ci.example.com/123"));

    // --input conflicts with field flags on create and update, but update still requires --key.
    let err = parse_from([
        "bb", "commit", "status", "create", "--commit", "110daa97", "--key", "ci-build", "--input",
        "-",
    ])
    .expect_err("create --input + --key must conflict");
    assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);

    let ok = parse_from([
        "bb", "commit", "status", "update", "--commit", "110daa97", "--key", "ci-build", "--input",
        "-",
    ])
    .expect("update --key + --input should parse");
    let Request::Commit(CommitRequest::Status(ok)) = ok else {
        panic!("expected commit status");
    };
    assert_eq!(ok.action, CommitStatusAction::Update);
    assert_eq!(ok.key.as_deref(), Some("ci-build"));
    assert_eq!(ok.input.as_deref(), Some("-"));

    let err = parse_from([
        "bb", "commit", "status", "update", "--commit", "110daa97", "--key", "ci-build", "--state",
        "FAILED", "--input", "-",
    ])
    .expect_err("update --input + --state must conflict");
    assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);

    // --sort must not swallow the following flag as its value; descending sorts use --sort=-field.
    assert!(
        parse_from([
            "bb", "commit", "status", "list", "--commit", "110daa97", "--sort", "--all",
        ])
        .is_err()
    );
    let sorted = parse_from([
        "bb",
        "commit",
        "status",
        "list",
        "--commit",
        "110daa97",
        "--sort=-updated_on",
        "--all",
    ])
    .expect("--sort=-updated_on should parse alongside --all");
    let Request::Commit(CommitRequest::Status(sorted)) = sorted else {
        panic!("expected commit status");
    };
    assert_eq!(sorted.sort.as_deref(), Some("-updated_on"));
    assert!(sorted.all);
}

#[test]
fn removed_issue_and_wiki_groups_are_parse_errors() {
    // Bitbucket Cloud removed native Issues and Wikis on 2026-08-20 (#58).
    for group in ["issue", "wiki"] {
        let error = parse_from(["bb", group, "list"]).expect_err("removed group should not parse");
        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::InvalidSubcommand,
            "{group}"
        );
    }
}

/// Every `bb ...` line in the skill's fenced code blocks must parse with the real clap definitions.
#[test]
fn skill_examples_parse() {
    let mut checked = 0;
    let mut in_fence = false;
    for line in bb_core::render::skill().lines() {
        if line.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        let Some(start) = line.find("bb ") else {
            continue;
        };
        let command = &line[start..];
        // `<...>` marks a synopsis placeholder, not a runnable example.
        if !in_fence
            || command.contains('<')
            || (start > 0 && !line[..start].trim_end().ends_with('|'))
        {
            continue;
        }
        let args = shell_words(command.split(" > ").next().unwrap());
        match parse_from(&args) {
            Ok(_) => {}
            Err(error) if error.kind() == clap::error::ErrorKind::DisplayHelp => {}
            Err(error) => panic!("skill example does not parse: {command}\n{error}"),
        }
        checked += 1;
    }
    assert!(checked > 20, "only {checked} examples found");
}

/// Every visible subcommand must appear in its group's `### <group>` section of the skill.
#[test]
fn skill_lists_every_subcommand() {
    use clap::CommandFactory;
    let skill = bb_core::render::skill();
    for group in Cli::command()
        .get_subcommands()
        .filter(|c| !c.is_hide_set())
    {
        let header = format!("\n### {}\n", group.get_name());
        let section = skill
            .split_once(&header)
            .unwrap_or_else(|| panic!("skill has no section {header:?}"))
            .1;
        let section = section.split("\n#").next().unwrap();
        for sub in group.get_subcommands().filter(|c| !c.is_hide_set()) {
            let name = sub.get_name();
            assert!(
                section.contains(&format!("`{name}`")) || section.contains(&format!("`{name} ")),
                "skill section for `{}` does not mention `{name}`",
                group.get_name()
            );
        }
    }
}

/// Splits on spaces, keeping double- or single-quoted runs as one word (quotes removed).
fn shell_words(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut started = false;
    for ch in line.chars() {
        match (quote, ch) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => word.push(c),
            (None, '"' | '\'') => {
                quote = Some(ch);
                started = true;
            }
            (None, ' ') => {
                if started {
                    words.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            (None, c) => {
                word.push(c);
                started = true;
            }
        }
    }
    if started {
        words.push(word);
    }
    words
}
