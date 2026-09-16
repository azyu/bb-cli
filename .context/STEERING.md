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
- Phase: v0.3.0 released
- Owner: Main
- Tracking: [GitHub Issue #57](https://github.com/azyu/bb-cli/issues/57) (completed). No open issues.
- Notes: `v0.3.0` is published from `4af92c2`. Release Build run `35095556902` succeeded, all five platform archives plus `checksums.txt` are uploaded, `shasum -a 256 -c checksums.txt` passed for every archive, and the natively-run macOS ARM64 binary reports `0.3.0+4af92c2` and exercises all three shipped features. The workflow synchronized the workspace version (`7d9f870`) and the Homebrew formula, whose three sha256 values were cross-checked against the release checksums. The Linux and Windows archives were checksum-verified but not executed.
- Shipped since v0.2.9: `bb auth switch` and `bb auth list --output table|json` ([#50](https://github.com/azyu/bb-cli/issues/50), [PR #51](https://github.com/azyu/bb-cli/pull/51)); global `--profile` accepted before or after the subcommand ([#52](https://github.com/azyu/bb-cli/issues/52), [PR #54](https://github.com/azyu/bb-cli/pull/54)); ko-kr README drift and prose fixes ([#55](https://github.com/azyu/bb-cli/issues/55), [PR #56](https://github.com/azyu/bb-cli/pull/56)). [#53](https://github.com/azyu/bb-cli/issues/53) was closed as superseded by #51.
- Deliberately not shipped: a `BB_PROFILE` environment variable. `docs/spec.md` rules out an env-over-config layer for command execution, and `gh`, `glab` and `tea` all expose a persisted active account plus a per-command override with no env var for selecting a stored account.
- Previous: v0.2.9 release ([GitHub Issue #49](https://github.com/azyu/bb-cli/issues/49), completed) published from merged main with all five archives verified.
