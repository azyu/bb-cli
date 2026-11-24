# bb

[![CI](https://github.com/azyu/bb-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/azyu/bb-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/azyu/bb-cli)](https://github.com/azyu/bb-cli/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[English](README.MD) | [한국어](README.ko-kr.MD)

> A fast, `gh`-style Bitbucket Cloud CLI written in Rust for humans and AI agents.

## Features

- `gh`-style command groups for repositories, pull requests, pipelines, commit reports/statuses, and raw API calls
- Automatic workspace and repository detection from your local Bitbucket `remote.origin.url`
- Text and table output for interactive use, plus stable JSON output for scripts and agents
- Shell completions for bash, zsh, fish, and PowerShell
- Prebuilt cross-platform binaries for Linux, macOS, and Windows

## Installation

### Homebrew

```bash
brew install azyu/tap/bb
```

### Prebuilt binaries

Download the archive for your platform from [GitHub Releases](https://github.com/azyu/bb-cli/releases/latest).

| Platform | Asset |
|----------|-------|
| Linux amd64 | `bb_0.x.y_linux_amd64.tar.gz` |
| Linux arm64 | `bb_0.x.y_linux_arm64.tar.gz` |
| macOS arm64 | `bb_0.x.y_macos_arm64.tar.gz` |
| Windows x64 | `bb_0.x.y_windows_x64.zip` |
| Windows arm64 | `bb_0.x.y_windows_arm64.zip` |

### From source

Requires Rust 1.93 or later.

```bash
make install
```

## Quick Start

### 1. Authenticate

```bash
bb auth login --username "<bitbucket-email>" --token "<your-token>"
```

> [!TIP]
> You can also pass the token over stdin or via environment variables instead of a command-line flag:
> ```bash
> echo "<token>" | bb auth login --username "<email>" --with-token
> BITBUCKET_USERNAME="<email>" BITBUCKET_TOKEN="<token>" bb auth login
> ```

### 2. Verify your active profile

```bash
bb auth status
```

> [!TIP]
> To use multiple accounts, save each under a profile name. You can switch the active profile or override it for a single command:
> ```bash
> bb auth login --profile work     --username "<work-email>"     --token "<token>"
> bb auth login --profile personal --username "<personal-email>" --token "<token>"
>
> bb auth list                   # List saved profiles (* marks the active one)
> bb auth list --output json     # Machine-readable profile list
> bb auth switch --profile work  # Switch the active profile without re-entering a token
> bb pr list --profile personal  # Override the profile for one command only
> bb --profile personal pr list  # Same as above (--profile works before or after the subcommand)
> ```

### 3. Run common commands

Inside a local Bitbucket repository, `bb` detects `--workspace` and `--repo` from `remote.origin.url`.
Outside a repository, or when targeting a different one, pass `-R <workspace>/<repo>` instead. Do not combine `-R` with `--workspace` or `--repo`.

```bash
bb pr list
bb pr comments 123
bb pr comments 123 --comment-id 456 --output json
bb pr comment 123 --content "Reply text" --parent 456
bb pr comment-update 123 --comment-id 456 --content "Updated reply text"
bb pr create --title "My PR" --source feature --destination main
bb api --method POST --input ./body.json repositories/acme/widgets/pullrequests/123/comments
bb pipeline list
bb pipeline get --uuid "{pipeline-uuid}"
bb pipeline get --build 123
bb pipeline log --build 123 --step "{step-uuid}"
bb pipeline stop 123
bb commit report list --commit 110daa97
bb commit report delete --commit 110daa97 --id mysystem-001
bb commit status list --commit 110daa97
bb commit status create --commit 110daa97 --key ci-build --state SUCCESSFUL --url https://ci.example.com/builds/123
```

### 4. Enable shell completion

Generate a completion script for your shell (`bash`, `zsh`, `fish`, or `powershell`):

```bash
bb completion zsh
```

## Command Overview

| Group | Subcommands |
|-------|-------------|
| `bb auth` | `login`, `status`, `list`, `switch`, `logout` |
| `bb repo` | `list` |
| `bb pr` | `list`, `get`, `create`, `update`, `merge`, `approve`, `unapprove`, `request-changes`, `remove-request-changes`, `decline`, `comment`, `comment-update`, `comments`, `diff`, `diffstat`, `statuses`, `activity` |
| `bb pipeline` | `list`, `get`, `steps`, `log`, `run`, `stop` (`cancel`) |
| `bb commit` | `report list\|get\|put\|delete`, `status list\|get\|create\|update` |
| `bb api` | Direct Bitbucket Cloud API requests |
| `bb completion` | `bash`, `zsh`, `fish`, `powershell` |
| `bb version` | Build metadata |
| `bb skill` | Agent skill (Markdown) for coding agents, matching this `bb` version |

> [!NOTE]
> - Commands targeting an existing PR accept the ID positionally or with `--id` (`bb pr comments 123` and `bb pr comments --id 123` are equivalent).
> - In `bb pr comments` and `bb pr comment-update`, pass `--comment-id` to select a comment. `--id` always refers to the pull request.
> - Pipeline commands targeting a run (`get`, `steps`, `log`, `stop`) accept a positional build number or `{uuid}`, or `--build`/`--uuid`.
> - Combine `--output json` with `--json-fields` when scripts or agents only need a subset of fields.

## Configuration and Auth

`bb` resolves its configuration file in the following order:

1. `BB_CONFIG_PATH`
2. `$XDG_CONFIG_HOME/bb/config.json`
3. `~/.config/bb/config.json`

Supported authentication modes:

| Mode | Scheme | Username required |
|------|--------|-------------------|
| Personal API token | Basic (`username:token`) | Yes |
| Access token | Bearer | No |

> [!CAUTION]
> Tokens are stored in plain text in your local config file. Make sure the file permissions are restricted to your user.

For details on creating tokens and configuring permissions, see the Atlassian documentation:

- [Create an API token](https://support.atlassian.com/bitbucket-cloud/docs/create-an-api-token/)
- [API token permissions](https://support.atlassian.com/bitbucket-cloud/docs/api-token-permissions/)

## Scope Guide

For read-only usage, start with this baseline set of scopes:

- `read:repository:bitbucket`
- `read:pullrequest:bitbucket`
- `read:pipeline:bitbucket`
- `read:user:bitbucket`
- `read:workspace:bitbucket`

Only add `write:*` scopes when you need commands that create, update, or delete resources.

<details>
<summary>Command scope matrix</summary>

| Command | Required scopes |
|---------|-----------------|
| `bb repo list` | `read:repository` |
| `bb pr list/get/comments/diff/diffstat/statuses/activity` | `read:repository`, `read:pullrequest` |
| `bb pr create/update/merge/approve/unapprove/request-changes/remove-request-changes/decline/comment/comment-update` | `read:repository`, `read:pullrequest`, `write:pullrequest` |
| `bb pipeline list/get/steps/log` | `read:repository`, `read:pipeline` |
| `bb pipeline run/stop` | `read:repository`, `read:pipeline`, `write:pipeline` |
| `bb commit report list/get/put/delete` | `read:repository` |
| `bb commit status list/get/create/update` | `read:repository` |
| `bb api` | Varies by endpoint |
| `bb completion`, `bb version`, `bb skill` | None |

</details>

## Developer Docs

- [Technical specification](docs/spec.md)
- [Command contracts](docs/command-contracts.md)
