# bb

[![CI](https://github.com/azyu/bb-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/azyu/bb-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/azyu/bb-cli)](https://github.com/azyu/bb-cli/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[English](README.MD) | [한국어](README.ko-kr.MD)

> Rust로 만든, 빠르고 에이전트 친화적인 Bitbucket Cloud CLI. `gh`에서 영감을 받았습니다.

## 특징

- repo, PR, pipeline, issue, wiki와 raw API 호출을 `gh` 스타일 그룹으로 제공
- 로컬 Bitbucket `remote.origin.url`에서 repo context 자동 추론
- 사람용 text/table 출력과 자동화용 안정적인 JSON 출력을 함께 지원
- bash, zsh, fish, PowerShell용 셸 자동완성 제공
- Linux, macOS, Windows용 릴리스 바이너리 제공

## 설치

### Homebrew

```bash
brew install azyu/tap/bb
```

### 미리 빌드된 바이너리

[GitHub Releases](https://github.com/azyu/bb-cli/releases/latest)에서 최신 아카이브를 다운로드하세요.

| 플랫폼 | 에셋 |
|--------|------|
| Linux amd64 | `bb_0.x.y_linux_amd64.tar.gz` |
| Linux arm64 | `bb_0.x.y_linux_arm64.tar.gz` |
| macOS arm64 | `bb_0.x.y_macos_arm64.tar.gz` |
| Windows x64 | `bb_0.x.y_windows_x64.zip` |
| Windows arm64 | `bb_0.x.y_windows_arm64.zip` |

### 소스에서 빌드

Rust 1.93+가 필요합니다.

```bash
make install
```

## 빠른 시작

### 1. 인증

```bash
bb auth login --username "<bitbucket-email>" --token "<your-token>"
```

> [!TIP]
> stdin이나 환경변수로도 토큰을 전달할 수 있습니다.
> ```bash
> echo "<token>" | bb auth login --username "<email>" --with-token
> BITBUCKET_USERNAME="<email>" BITBUCKET_TOKEN="<token>" bb auth login
> ```

### 2. 활성 프로필 확인

```bash
bb auth status
```

> [!TIP]
> **계정이 여러 개라면** 계정마다 프로필 이름을 붙여 두세요. 그러면 전환하거나 명령 단위로
> 활성 프로필을 덮어쓸 수 있습니다.
> ```bash
> bb auth login --profile work     --username "<업무-이메일>" --token "<token>"
> bb auth login --profile personal --username "<개인-이메일>" --token "<token>"
>
> bb auth list                   # 저장된 프로필 목록, 활성 프로필은 * 표시
> bb auth list --output json     # 동일 내용, 기계 판독용
> bb auth switch --profile work  # 활성 프로필 변경, 토큰 재입력 불필요
> bb pr list --profile personal  # 이번 명령만 덮어쓰기, 활성 프로필은 그대로
> bb --profile personal pr list  # 동일; --profile은 어느 위치든 가능
> ```

### 3. 자주 쓰는 명령 실행

Bitbucket 저장소 안에서는 `bb`가 `remote.origin.url`에서 `--workspace`와 `--repo`를 자동으로 추론합니다.
저장소 밖에서는 `-R <workspace>/<repo>`로 대상을 명시적으로 지정합니다. `-R`은 `--workspace` 또는 `--repo`와 함께 사용할 수 없습니다.

```bash
bb pr list
bb pr comments 123
bb pr comments 123 --comment-id 456 --output json
bb pr comment 123 --content "답글 내용" --parent 456
bb pr comment-update 123 --comment-id 456 --content "수정된 댓글"
bb pr create --title "My PR" --source feature --destination main
bb api --method POST --input ./body.json repositories/acme/widgets/pullrequests/123/comments
bb pipeline list
bb pipeline get --uuid "{pipeline-uuid}"
bb pipeline get --build 123
bb pipeline log --build 123 --step "{step-uuid}"
```

### 4. 셸 자동완성 활성화

```bash
bb completion zsh
```

## 명령 개요

| 그룹 | 서브커맨드 |
|------|-----------|
| `bb auth` | `login`, `status`, `list`, `switch`, `logout` |
| `bb repo` | `list` |
| `bb pr` | `list`, `get`, `create`, `update`, `merge`, `approve`, `unapprove`, `request-changes`, `remove-request-changes`, `decline`, `comment`, `comment-update`, `comments`, `diff`, `diffstat`, `statuses`, `activity` |
| `bb pipeline` | `list`, `get`, `steps`, `log`, `run` |
| `bb issue` | `list`, `create`, `update` |
| `bb wiki` | `list`, `get`, `put` |
| `bb api` | Bitbucket Cloud raw endpoint 호출 |
| `bb completion` | `bash`, `zsh`, `fish`, `powershell` |
| `bb version` | 빌드 메타데이터 출력 |

> [!NOTE]
> 기존 PR을 대상으로 하는 명령은 PR ID를 positional `<id>` 또는 `--id`로 받을 수 있습니다.
>
> 예: `bb pr comments 123`, `bb pr comments --id 123`
>
> Pipeline 조회 명령은 `--uuid` 또는 `--build` 중 하나를 받을 수 있습니다.
>
> `bb pr comments`와 `bb pr comment-update`에서 `--comment-id`는 해당 PR 안의 단일 댓글을 선택합니다. 댓글 ID를 `--id`로 넘기면 안 됩니다.
>
> 에이전트가 JSON 전체가 아니라 일부 필드만 필요하면 `--output json`과 `--json-fields`를 함께 사용하세요.

## 설정과 인증

설정 파일 경로 우선순위:

1. `BB_CONFIG_PATH`
2. `$XDG_CONFIG_HOME/bb/config.json`
3. `~/.config/bb/config.json`

인증 모드:

| 모드 | 방식 | 사용자명 필요 |
|------|------|---------------|
| Personal API token | Basic (`username:token`) | 예 |
| Access token | Bearer | 아니오 |

> [!CAUTION]
> 토큰은 로컬 설정 파일에 평문으로 저장됩니다. 파일 권한을 엄격하게 관리하세요.

토큰 생성과 권한은 Atlassian 공식 문서를 참고하세요.

- [Create an API token](https://support.atlassian.com/bitbucket-cloud/docs/create-an-api-token/)
- [API token permissions](https://support.atlassian.com/bitbucket-cloud/docs/api-token-permissions/)

## 권한 가이드

권장 읽기 전용 기본 세트:

- `read:repository:bitbucket`
- `read:pullrequest:bitbucket`
- `read:pipeline:bitbucket`
- `read:issue:bitbucket`
- `read:wiki:bitbucket`
- `read:user:bitbucket`
- `read:workspace:bitbucket`

변경 명령을 사용할 때만 `write:*` 권한을 추가하세요.

<details>
<summary>명령별 권한 매트릭스</summary>

| 명령 | 필요 권한 |
|------|-----------|
| `bb repo list` | `read:repository` |
| `bb pr list/get/comments/diff/diffstat/statuses/activity` | `read:repository`, `read:pullrequest` |
| `bb pr create/update/merge/approve/unapprove/request-changes/remove-request-changes/decline/comment/comment-update` | `read:repository`, `read:pullrequest`, `write:pullrequest` |
| `bb pipeline list/get/steps/log` | `read:repository`, `read:pipeline` |
| `bb pipeline run` | `read:repository`, `read:pipeline`, `write:pipeline` |
| `bb wiki list/get` | `read:repository`, `read:wiki` |
| `bb wiki put` | `read:repository`, `read:wiki`, `write:wiki` |
| `bb issue list` | `read:repository`, `read:issue` |
| `bb issue create/update` | `read:repository`, `read:issue`, `write:issue` |
| `bb api` | 엔드포인트에 따라 다름 |
| `bb completion`, `bb version` | 없음 |

</details>

## 개발자 문서

- [Technical spec](docs/spec.md)
- [Command contracts](docs/command-contracts.md)
