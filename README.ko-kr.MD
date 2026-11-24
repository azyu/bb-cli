# bb

[![CI](https://github.com/azyu/bb-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/azyu/bb-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/azyu/bb-cli)](https://github.com/azyu/bb-cli/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[English](README.MD) | [한국어](README.ko-kr.MD)

> Rust로 만든 `gh` 스타일 Bitbucket Cloud CLI입니다. 사람과 AI 에이전트 모두 쓰기 편하게 만들었습니다.

## 특징

- 저장소, PR, 파이프라인, 커밋 리포트·상태, raw API 호출을 `gh` 스타일 명령 그룹으로 묶어 제공
- 로컬 Bitbucket 저장소의 `remote.origin.url`에서 워크스페이스와 저장소 자동 추론
- 터미널에서 읽기 편한 텍스트·테이블 출력과 자동화·에이전트용 JSON 출력 지원
- bash, zsh, fish, PowerShell 자동완성 스크립트 생성
- Linux, macOS, Windows용 미리 빌드된 바이너리 배포

## 설치

### Homebrew

```bash
brew install azyu/tap/bb
```

### 미리 빌드된 바이너리

[GitHub Releases](https://github.com/azyu/bb-cli/releases/latest)에서 사용 중인 플랫폼에 맞는 아카이브를 다운로드하세요.

| 플랫폼 | 파일 |
|--------|------|
| Linux amd64 | `bb_0.x.y_linux_amd64.tar.gz` |
| Linux arm64 | `bb_0.x.y_linux_arm64.tar.gz` |
| macOS arm64 | `bb_0.x.y_macos_arm64.tar.gz` |
| Windows x64 | `bb_0.x.y_windows_x64.zip` |
| Windows arm64 | `bb_0.x.y_windows_arm64.zip` |

### 소스에서 빌드

Rust 1.93 이상이 필요합니다.

```bash
make install
```

## 빠른 시작

### 1. 인증

```bash
bb auth login --username "<bitbucket-email>" --token "<your-token>"
```

> [!TIP]
> 명령줄 인자 대신 표준 입력(stdin)이나 환경변수로 토큰을 넘길 수도 있습니다.
> ```bash
> echo "<token>" | bb auth login --username "<email>" --with-token
> BITBUCKET_USERNAME="<email>" BITBUCKET_TOKEN="<token>" bb auth login
> ```

### 2. 활성 프로필 확인

```bash
bb auth status
```

> [!TIP]
> 계정이 여러 개라면 프로필 이름을 붙여 저장해 두세요. 기본 프로필을 바꾸거나 특정 명령에만 다른 프로필을 적용할 수 있습니다.
> ```bash
> bb auth login --profile work     --username "<업무-이메일>" --token "<token>"
> bb auth login --profile personal --username "<개인-이메일>" --token "<token>"
>
> bb auth list                   # 저장된 프로필 목록 (활성 프로필은 * 표시)
> bb auth list --output json     # 같은 내용을 JSON으로 출력
> bb auth switch --profile work  # 토큰을 다시 입력하지 않고 활성 프로필 변경
> bb pr list --profile personal  # 이 명령에만 적용 (활성 프로필은 유지)
> bb --profile personal pr list  # 같은 동작 (--profile은 서브커맨드 앞뒤 어디든 가능)
> ```

### 3. 자주 쓰는 명령 실행

로컬 Bitbucket 저장소 안에서 실행하면 `bb`가 `remote.origin.url`을 읽어 `--workspace`와 `--repo`를 자동으로 추론합니다.
저장소 밖에서 실행하거나 다른 저장소를 지정하려면 `-R <workspace>/<repo>` 옵션을 넘기세요. `-R`은 `--workspace`나 `--repo`와 함께 쓸 수 없습니다.

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
bb pipeline stop 123
bb commit report list --commit 110daa97
bb commit report delete --commit 110daa97 --id mysystem-001
bb commit status list --commit 110daa97
bb commit status create --commit 110daa97 --key ci-build --state SUCCESSFUL --url https://ci.example.com/builds/123
```

### 4. 셸 자동완성 활성화

사용 중인 셸(`bash`, `zsh`, `fish`, `powershell`)에 맞는 자동완성 스크립트를 생성할 수 있습니다.

```bash
bb completion zsh
```

## 명령 개요

| 그룹 | 서브커맨드 |
|------|-----------|
| `bb auth` | `login`, `status`, `list`, `switch`, `logout` |
| `bb repo` | `list` |
| `bb pr` | `list`, `get`, `create`, `update`, `merge`, `approve`, `unapprove`, `request-changes`, `remove-request-changes`, `decline`, `comment`, `comment-update`, `comments`, `diff`, `diffstat`, `statuses`, `activity` |
| `bb pipeline` | `list`, `get`, `steps`, `log`, `run`, `stop` (`cancel`) |
| `bb commit` | `report list\|get\|put\|delete`, `status list\|get\|create\|update` |
| `bb api` | Bitbucket Cloud raw API 엔드포인트 직접 호출 |
| `bb completion` | `bash`, `zsh`, `fish`, `powershell` |
| `bb version` | 빌드 정보 출력 |
| `bb skill` | 코딩 에이전트용 skill(Markdown) 출력. 설치된 `bb` 버전과 내용이 일치 |

> [!NOTE]
> - 기존 PR을 대상으로 하는 명령은 PR ID를 위치 인자(`<id>`)나 `--id`로 넘길 수 있습니다. `bb pr comments 123`과 `bb pr comments --id 123`은 같게 동작합니다.
> - `bb pr comments`와 `bb pr comment-update`에서 특정 댓글을 고를 때는 `--comment-id`를 쓰세요. `--id`는 항상 PR ID입니다.
> - 특정 파이프라인을 대상으로 하는 명령(`get`, `steps`, `log`, `stop`)은 위치 인자(빌드 번호 또는 `{uuid}`)나 `--build`/`--uuid`로 대상을 지정합니다.
> - 스크립트나 에이전트가 응답의 일부 필드만 필요하다면 `--output json`과 `--json-fields`를 함께 쓰세요.

## 설정과 인증

설정 파일은 아래 순서대로 탐색합니다.

1. `BB_CONFIG_PATH`
2. `$XDG_CONFIG_HOME/bb/config.json`
3. `~/.config/bb/config.json`

지원하는 인증 방식은 다음과 같습니다.

| 모드 | 인증 방식 | 사용자명 필요 여부 |
|------|----------|-------------------|
| Personal API token | Basic (`username:token`) | 예 |
| Access token | Bearer | 아니오 |

> [!CAUTION]
> 토큰은 로컬 설정 파일에 평문으로 저장됩니다. 본인 계정만 읽을 수 있도록 파일 권한을 제한해 두세요.

토큰 발급 방법과 세부 권한은 Atlassian 공식 문서를 참고하세요.

- [Create an API token](https://support.atlassian.com/bitbucket-cloud/docs/create-an-api-token/)
- [API token permissions](https://support.atlassian.com/bitbucket-cloud/docs/api-token-permissions/)

## 권한 가이드

읽기 전용으로 사용한다면 아래 기본 권한 세트를 권장합니다.

- `read:repository:bitbucket`
- `read:pullrequest:bitbucket`
- `read:pipeline:bitbucket`
- `read:user:bitbucket`
- `read:workspace:bitbucket`

생성·수정·삭제 같은 쓰기 명령이 필요할 때만 `write:*` 권한을 추가하세요.

<details>
<summary>명령별 권한 매트릭스</summary>

| 명령 | 필요 권한 |
|------|-----------|
| `bb repo list` | `read:repository` |
| `bb pr list/get/comments/diff/diffstat/statuses/activity` | `read:repository`, `read:pullrequest` |
| `bb pr create/update/merge/approve/unapprove/request-changes/remove-request-changes/decline/comment/comment-update` | `read:repository`, `read:pullrequest`, `write:pullrequest` |
| `bb pipeline list/get/steps/log` | `read:repository`, `read:pipeline` |
| `bb pipeline run/stop` | `read:repository`, `read:pipeline`, `write:pipeline` |
| `bb commit report list/get/put/delete` | `read:repository` |
| `bb commit status list/get` | `read:repository` |
| `bb commit status create/update` | `read:repository`, `write:repository` |
| `bb api` | 호출하는 엔드포인트에 따라 다름 |
| `bb completion`, `bb version`, `bb skill` | 필요 없음 |

</details>

## 개발자 문서

- [기술 명세서 (Technical spec)](docs/spec.md)
- [명령 계약 (Command contracts)](docs/command-contracts.md)
