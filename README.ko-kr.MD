# bb

[![CI](https://github.com/azyu/bb-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/azyu/bb-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/azyu/bb-cli)](https://github.com/azyu/bb-cli/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[English](README.md) | [한국어](README.ko-kr.MD)

> `gh`에서 영감을 받은, 빠르고 에이전트 친화적인 Bitbucket Cloud CLI. Rust로 작성.

## 특징

- **`gh` 스타일 명령 구조** — 익숙한 verb-noun 패턴
- **Git remote 자동 추론** — `remote.origin.url`에서 workspace/repo 자동 감지
- **에이전트 퍼스트 설계** — 구조화된 JSON 출력과 에러 엔벨로프
- **다중 출력 형식** — `table`, `json`, `text`
- **셸 자동완성** — bash / zsh / fish / powershell, 2단계 서브커맨드 지원
- **크로스 플랫폼 바이너리** — Linux (amd64/arm64), macOS (arm64), Windows (x64/arm64)
- **Raw API 이스케이프 해치** — `bb api`로 모든 Bitbucket Cloud 엔드포인트 호출

## 설치

### Homebrew

```bash
brew install azyu/tap/bb
```

### 미리 빌드된 바이너리

[GitHub Releases](https://github.com/azyu/bb-cli/releases/latest)에서 다운로드.

| 플랫폼 | 에셋 |
|--------|------|
| Linux amd64 | `bb_linux_amd64.tar.gz` |
| Linux arm64 | `bb_linux_arm64.tar.gz` |
| macOS arm64 | `bb_macos_arm64.tar.gz` |
| Windows x64 | `bb_windows_x64.zip` |
| Windows arm64 | `bb_windows_arm64.zip` |

### 소스에서 빌드

Rust 1.93+ 필요.

```bash
make install   # 릴리스 바이너리 빌드 → ~/.local/bin/bb
```

## 빠른 시작

### 1. 인증

```bash
bb auth login --username "<bitbucket-email>" --token "<your-token>"
```

> [!TIP]
> stdin으로 토큰을 전달하거나 환경변수를 설정할 수도 있습니다:
> ```bash
> echo "<token>" | bb auth login --username "<email>" --with-token
> BITBUCKET_USERNAME="<email>" BITBUCKET_TOKEN="<token>" bb auth login
> ```

### 2. 확인

```bash
bb auth status
```

### 3. 사용

Bitbucket 저장소 디렉토리 안에서는 workspace/repo 플래그가 자동 추론됩니다:

```bash
bb pr list
bb pr create --title "My PR" --source feature --destination main
bb repo list
bb pipeline list
```

### 4. 셸 자동완성 활성화

```bash
bb completion zsh   # bash / zsh / fish / powershell
```

## 명령어

| 그룹 | 서브커맨드 | 인증 scope |
|------|-----------|-----------|
| `bb auth` | `login`, `status`, `logout` | — |
| `bb repo` | `list` | `read:repository` |
| `bb pr` | `list`, `get`, `create`, `update`, `merge`, `approve`, `unapprove`, `request-changes`, `remove-request-changes`, `decline`, `comment`, `comments`, `diff`, `statuses`, `activity` | `read/write:pullrequest` |
| `bb pipeline` | `list`, `run` | `read/write:pipeline` |
| `bb wiki` | `list`, `get`, `put` | `read/write:wiki` |
| `bb issue` | `list`, `create`, `update` | `read/write:issue` |
| `bb api` | *(raw endpoint)* | 상황에 따라 다름 |
| `bb completion` | `bash`, `zsh`, `fish`, `powershell` | — |

> [!NOTE]
> PR 서브커맨드 이름은 Bitbucket Cloud REST operation 제목을 따릅니다:
> `view`/`edit` 대신 `get`/`update`, 로컬 별칭 대신 `request-changes`/`remove-request-changes`.

`bb <command> --help`로 상세 플래그 설명을 확인하세요.

## 설정

설정 파일 경로 우선순위:

1. `BB_CONFIG_PATH` (명시적 override)
2. `$XDG_CONFIG_HOME/bb/config.json`
3. `~/.config/bb/config.json` (기본값)

> [!CAUTION]
> 토큰은 설정 파일에 **평문으로 저장**됩니다 (MVP 단계). 파일 권한을 적절히 관리하세요.

## 인증

두 가지 인증 모드를 지원합니다:

| 모드 | 스킴 | 사용자명 필요 |
|------|------|-------------|
| Personal API 토큰 | Basic (`username:token`) | 예 |
| Access 토큰 | Bearer | 아니오 |

토큰 생성은 [Atlassian 공식 문서](https://support.atlassian.com/bitbucket-cloud/docs/create-an-api-token/)를 참고하세요.
Scope 참조는 [API 토큰 권한](https://support.atlassian.com/bitbucket-cloud/docs/api-token-permissions/)을 참고하세요.

## Token Scope

권장 기본 세트 (읽기 전용):

- `read:repository:bitbucket`
- `read:pullrequest:bitbucket`
- `read:pipeline:bitbucket`
- `read:issue:bitbucket`
- `read:wiki:bitbucket`
- `read:user:bitbucket`
- `read:workspace:bitbucket`

변경 명령이 필요하면 `write:*` scope를 추가하세요.

<details>
<summary>전체 CLI scope 매트릭스</summary>

| 명령 | 필요 scope |
|------|-----------|
| `bb repo list` | `read:repository` |
| `bb pr list/get/comments/diff/statuses/activity` | `read:repository`, `read:pullrequest` |
| `bb pr create/update/merge/approve/…/comment` | `read:repository`, `read:pullrequest`, `write:pullrequest` |
| `bb pipeline list` | `read:repository`, `read:pipeline` |
| `bb pipeline run` | `read:repository`, `read:pipeline`, `write:pipeline` |
| `bb wiki list/get` | `read:repository`, `read:wiki` |
| `bb wiki put` | `read:repository`, `read:wiki`, `write:wiki` |
| `bb issue list` | `read:repository`, `read:issue` |
| `bb issue create/update` | `read:repository`, `read:issue`, `write:issue` |
| `bb api` | `read:user`, `read:workspace` (상황에 따라 다름) |
| `bb completion`, `bb version` | — |

**기본적으로 피할 권한:** `admin:*`, `delete:*`, `write:permission:bitbucket`

</details>

## 개발

```bash
make build    # 디버그 빌드
make install  # 릴리스 빌드 → ~/.local/bin/bb
make test     # 전체 테스트 실행
make lint     # cargo check --all-targets
make fmt      # cargo fmt --all
make help     # 모든 타겟 표시
```

참고: [SPEC.md](docs/SPEC.md) · [command-contracts.md](docs/command-contracts.md)
