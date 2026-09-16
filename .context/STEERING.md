# PLAN

## Objective
- Rebuild `bb` as a Rust-first Bitbucket Cloud CLI with a `gh`-like structure.
- Keep the public binary name `bb`.
- Complete phase 1 with Rust MVP parity for the documented command set only.

## Phases
1. Rust migration reset (scope, plan/task tracker, docs baseline).
2. Rust workspace bootstrap (`bb-cli` + `bb-core`) and shared foundations.
3. MVP command port (`auth`, `api`, `repo`, `pr`, `pipeline`, `issue`, `wiki`, `completion`, `version`).
4. Rust-only validation, release workflow conversion, and Go removal.

## Success Criteria
- Rust workspace builds and tests cleanly with Cargo.
- The documented Cloud MVP commands are implemented in Rust and verified.
- Config precedence, auth modes, repo inference, pagination, and output modes match the documented contract.
- CI and release workflows build Rust artifacts named `bb`.
- Go entrypoints and Go-only workflows are removed after Rust verification passes.

## Current Phase
- Phase: post-v0.2.9 — promoting `--profile` to a global flag (#52)
- Owner: Main
- Tracking: [GitHub Issue #52](https://github.com/azyu/bb-cli/issues/52) — make `--profile` accepted before or after the subcommand, like `-R`. Previous: [#50](https://github.com/azyu/bb-cli/issues/50) (completed, merged as `95b7f4d` via [PR #51](https://github.com/azyu/bb-cli/pull/51))
- Notes: `bb auth switch --profile <name>` and `bb auth list --output table|json` are on `main` but not in a release; the last published version is v0.2.9. Named profiles, an active `current`, and per-command `--profile` already existed; the gap was changing the active profile without re-supplying a token, and seeing which profiles exist. Prior art (`gh` 2.101, `glab`, `tea`, `aws`, `kubectl`) shows forge CLIs persist an active account plus a per-command override and expose no env var for selecting a stored account, so no `BB_PROFILE` was added — that also keeps `docs/spec.md` env-precedence rule intact. Codex review on the PR raised two findings, both accepted and fixed: `auth list` needed `--output json` (it would have been the only `list` command without it), and the resulting JSON mode needed an arm in `wants_json_errors` to emit error envelopes.
- Previous: v0.2.9 release ([GitHub Issue #49](https://github.com/azyu/bb-cli/issues/49), completed): `v0.2.9` is published from merged main. Release Build run `34924684819` succeeded, all five platform archives plus `checksums.txt` are uploaded, `sha256sum -c checksums.txt` passed for every archive, native macOS ARM64 `bb version` reports `0.2.9+82d4253`, and the workflow synchronized the workspace version and Homebrew formula.
