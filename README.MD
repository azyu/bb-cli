# bb

[![CI](https://github.com/azyu/bb-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/azyu/bb-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/azyu/bb-cli)](https://github.com/azyu/bb-cli/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[English](README.md) | [한국어](README.ko-kr.MD)

> A fast, agent-friendly Bitbucket Cloud CLI inspired by `gh`. Built in Rust.

## Features

- **`gh`-style command grouping** — familiar verb-noun structure
- **Git remote inference** — auto-detects workspace/repo from `remote.origin.url`
- **Agent-first design** — structured JSON output and error envelopes
- **Multi-output formats** — `table`, `json`, `text`
- **Shell completion** — bash / zsh / fish / powershell with 2-level subcommand support
- **Cross-platform binaries** — Linux (amd64/arm64), macOS (arm64), Windows (x64/arm64)
- **Raw API escape hatch** — `bb api` for any Bitbucket Cloud endpoint

## Installation

### Homebrew

```bash
brew install azyu/tap/bb
```

### Prebuilt binary

Download from [GitHub Releases](https://github.com/azyu/bb-cli/releases/latest).

| Platform | Asset |
|----------|-------|
| Linux amd64 | `bb_linux_amd64.tar.gz` |
| Linux arm64 | `bb_linux_arm64.tar.gz` |
| macOS arm64 | `bb_macos_arm64.tar.gz` |
| Windows x64 | `bb_windows_x64.zip` |
| Windows arm64 | `bb_windows_arm64.zip` |

### From source

Requires Rust 1.93+.

```bash
make install   # builds release binary → ~/.local/bin/bb
```

## Quick Start

### 1. Authenticate

```bash
bb auth login --username "<bitbucket-email>" --token "<your-token>"
```

> [!TIP]
> You can also pipe the token via stdin or set environment variables:
> ```bash
> echo "<token>" | bb auth login --username "<email>" --with-token
> BITBUCKET_USERNAME="<email>" BITBUCKET_TOKEN="<token>" bb auth login
> ```

### 2. Verify

```bash
bb auth status
```

### 3. Use

Inside a Bitbucket repo directory, workspace and repo flags are inferred automatically:

```bash
bb pr list
bb pr create --title "My PR" --source feature --destination main
bb repo list
bb pipeline list
```

### 4. Enable shell completion

```bash
bb completion zsh   # bash / zsh / fish / powershell
```

## Commands

| Group | Subcommands | Auth scope |
|-------|-------------|------------|
| `bb auth` | `login`, `status`, `logout` | — |
| `bb repo` | `list` | `read:repository` |
| `bb pr` | `list`, `get`, `create`, `update`, `merge`, `approve`, `unapprove`, `request-changes`, `remove-request-changes`, `decline`, `comment`, `comments`, `diff`, `statuses`, `activity` | `read/write:pullrequest` |
| `bb pipeline` | `list`, `run` | `read/write:pipeline` |
| `bb wiki` | `list`, `get`, `put` | `read/write:wiki` |
| `bb issue` | `list`, `create`, `update` | `read/write:issue` |
| `bb api` | *(raw endpoint)* | varies |
| `bb completion` | `bash`, `zsh`, `fish`, `powershell` | — |

> [!NOTE]
> PR subcommand names follow Bitbucket Cloud REST operation titles:
> `get`/`update` instead of `view`/`edit`, `request-changes`/`remove-request-changes` instead of local synonyms.

Use `bb <command> --help` for detailed flag descriptions.

## Configuration

Config path priority:

1. `BB_CONFIG_PATH` (explicit override)
2. `$XDG_CONFIG_HOME/bb/config.json`
3. `~/.config/bb/config.json` (default)

> [!CAUTION]
> Tokens are stored as **plain text** in the config file (MVP state). Guard file permissions accordingly.

## Authentication

Two auth modes are supported:

| Mode | Scheme | Username required |
|------|--------|-------------------|
| Personal API token | Basic (`username:token`) | Yes |
| Access token | Bearer | No |

For token creation, see the [Atlassian docs](https://support.atlassian.com/bitbucket-cloud/docs/create-an-api-token/).
For scope reference, see the [API token permissions](https://support.atlassian.com/bitbucket-cloud/docs/api-token-permissions/).

## Token Scopes

Recommended baseline (read-only):

- `read:repository:bitbucket`
- `read:pullrequest:bitbucket`
- `read:pipeline:bitbucket`
- `read:issue:bitbucket`
- `read:wiki:bitbucket`
- `read:user:bitbucket`
- `read:workspace:bitbucket`

Add `write:*` scopes as needed for mutating commands.

<details>
<summary>Full CLI scope matrix</summary>

| Command | Required scopes |
|---------|----------------|
| `bb repo list` | `read:repository` |
| `bb pr list/get/comments/diff/statuses/activity` | `read:repository`, `read:pullrequest` |
| `bb pr create/update/merge/approve/…/comment` | `read:repository`, `read:pullrequest`, `write:pullrequest` |
| `bb pipeline list` | `read:repository`, `read:pipeline` |
| `bb pipeline run` | `read:repository`, `read:pipeline`, `write:pipeline` |
| `bb wiki list/get` | `read:repository`, `read:wiki` |
| `bb wiki put` | `read:repository`, `read:wiki`, `write:wiki` |
| `bb issue list` | `read:repository`, `read:issue` |
| `bb issue create/update` | `read:repository`, `read:issue`, `write:issue` |
| `bb api` | `read:user`, `read:workspace` (varies) |
| `bb completion`, `bb version` | — |

**Avoid by default:** `admin:*`, `delete:*`, `write:permission:bitbucket`

</details>

## Development

```bash
make build    # debug build
make install  # release build → ~/.local/bin/bb
make test     # run all tests
make lint     # cargo check --all-targets
make fmt      # cargo fmt --all
make help     # show all targets
```

See also: [SPEC.md](docs/SPEC.md) · [command-contracts.md](docs/command-contracts.md)
