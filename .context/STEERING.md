# PLAN

## Objective
- Rebuild `bb` as a Rust-first Bitbucket Cloud CLI with a `gh`-like structure.
- Keep the public binary name `bb`.
- Complete phase 1 with Rust MVP parity for the documented command set only.

## Phases
1. Rust migration reset (scope, plan/task tracker, docs baseline).
2. Rust workspace bootstrap (`bb-cli` + `bb-core`) and shared foundations.
3. MVP command port (`auth`, `api`, `repo`, `pr`, `pipeline`, `completion`, `version`). `issue` and `wiki` were ported, then removed in #58 after the Bitbucket Cloud sunset.
4. Rust-only validation, release workflow conversion, and Go removal.

## Success Criteria
- Rust workspace builds and tests cleanly with Cargo.
- The documented Cloud MVP commands are implemented in Rust and verified.
- Config precedence, auth modes, repo inference, pagination, and output modes match the documented contract.
- CI and release workflows build Rust artifacts named `bb`.
- Go entrypoints and Go-only workflows are removed after Rust verification passes.

## Current Phase
- Phase: v0.5.0 released
- Owner: Main
- Tracking: [GitHub Issue #70](https://github.com/azyu/bb-cli/issues/70) (v0.5.0 release, completed). Open backlog issues: none (`#63` remains open on GitHub as a cleanup item).
- Notes: `v0.5.0` is published from `4753617` via Release Build run `45563470790` (tag push). All five archives pass `shasum -a 256 -c checksums.txt`; the natively-run macOS ARM64 binary reports `0.5.0+4753617` and `bb skill` lists `comment-resolve`. Workspace version synced in `82c0d5b`; Homebrew `bb.rb` is 0.5.0 with sha256 values matching the release checksums. Linux and Windows archives were checksum-verified but not executed. The `built:` line in `bb version` shows `github.event.repository.updated_at`, not the build time (reads 2026-11-09 for both v0.4.0 and v0.5.0).
- Shipped in v0.5.0 (since v0.4.0): `bb skill` embedded agent skill ([#65](https://github.com/azyu/bb-cli/issues/65), [PR #66](https://github.com/azyu/bb-cli/pull/66)); `bb pipeline stop` (alias `cancel`) and `bb commit status list|get(view)|create|update(edit)` ([#60](https://github.com/azyu/bb-cli/issues/60), [PR #67](https://github.com/azyu/bb-cli/pull/67)). `bb pr comment-resolve` and `comment-reopen` ([#68](https://github.com/azyu/bb-cli/issues/68), [PR #69](https://github.com/azyu/bb-cli/pull/69)). Live Bitbucket write verification for `bb pipeline stop`, `bb commit status create|update`, and `bb pr comment-resolve|comment-reopen` has not been run yet.
- Previous: v0.4.0 release. `v0.4.0` is published from `bddd7b8` via Release Build run `42571261154` (`workflow_dispatch`). The tag-push run `42569477276` was cancelled because GitHub retired the `macos-14` runner image on 2026-11-02; [PR #64](https://github.com/azyu/bb-cli/pull/64) moved the macOS build to `macos-latest` and stamps the tag's commit in `bb version` on dispatch re-runs. All five archives pass `shasum -a 256 -c checksums.txt`; the natively-run macOS ARM64 binary reports `0.4.0+bddd7b8`. Workspace version synced in `cb2d16a`. The Homebrew job first failed with `Bad credentials`: `HOMEBREW_TAP_TOKEN` was replaced in all six repos that use it (bb-cli, kis-cli, krx-cli, tossinvest-cli, skillctl, transnovel) with a fine-grained token scoped to `azyu/homebrew-tap`, and the re-run updated `bb.rb` to 0.4.0 with sha256 values matching the release checksums. Linux and Windows archives were checksum-verified but not executed; `bb commit report put|delete` have not been run against live Bitbucket.
- Shipped since v0.3.0: `bb commit report list|get|put|delete` ([#59](https://github.com/azyu/bb-cli/issues/59), [PR #61](https://github.com/azyu/bb-cli/pull/61)). Removed: `bb issue` and `bb wiki` ([#58](https://github.com/azyu/bb-cli/issues/58), [PR #62](https://github.com/azyu/bb-cli/pull/62)); Bitbucket Cloud removed Issues and Wikis on 2026-08-20.
- Previous: v0.3.0 release ([GitHub Issue #57](https://github.com/azyu/bb-cli/issues/57), completed).
- Notes: `v0.3.0` is published from `4af92c2`. Release Build run `35095556902` succeeded, all five platform archives plus `checksums.txt` are uploaded, `shasum -a 256 -c checksums.txt` passed for every archive, and the natively-run macOS ARM64 binary reports `0.3.0+4af92c2` and exercises all three shipped features. The workflow synchronized the workspace version (`7d9f870`) and the Homebrew formula, whose three sha256 values were cross-checked against the release checksums. The Linux and Windows archives were checksum-verified but not executed.
- Shipped since v0.2.9: `bb auth switch` and `bb auth list --output table|json` ([#50](https://github.com/azyu/bb-cli/issues/50), [PR #51](https://github.com/azyu/bb-cli/pull/51)); global `--profile` accepted before or after the subcommand ([#52](https://github.com/azyu/bb-cli/issues/52), [PR #54](https://github.com/azyu/bb-cli/pull/54)); ko-kr README drift and prose fixes ([#55](https://github.com/azyu/bb-cli/issues/55), [PR #56](https://github.com/azyu/bb-cli/pull/56)). [#53](https://github.com/azyu/bb-cli/issues/53) was closed as superseded by #51.
- Deliberately not shipped: a `BB_PROFILE` environment variable. `docs/spec.md` rules out an env-over-config layer for command execution, and `gh`, `glab` and `tea` all expose a persisted active account plus a per-command override with no env var for selecting a stored account.
- Previous: v0.2.9 release ([GitHub Issue #49](https://github.com/azyu/bb-cli/issues/49), completed) published from merged main with all five archives verified.
