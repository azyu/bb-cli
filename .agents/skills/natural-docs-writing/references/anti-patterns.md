# Anti-Patterns & Refinement Checklist

Concrete patterns to detect and fix when writing or translating technical documentation in English and Korean.

## 1. AI Formatting & Prose Slop (English & Korean)

| Pattern | Avoid | Prefer |
|---|---|---|
| Rhetorical hook | "Working with multiple accounts? Save each account..." / "여러 계정을 함께 쓰시나요?" | "Multiple accounts: save each under a profile name..." / "계정이 여러 개라면 계정마다 프로필 이름을 지정해..." |
| Bold-label-colon bullet spam | `- **Pull request IDs:** Commands targeting...`<br>`- **Comment IDs:** In bb pr comments...` | Plain bullets or short direct sentences without bold prefixes |
| Em-dash overuse | `built in Rust—inspired by gh` / `Outside a repository—or when targeting another—pass -R` | Parentheses, commas, or two short sentences |
| Hype / empty adjectives | "intuitive command groups", "직관적인 명령 그룹", "seamless workflow" | Concrete nouns and verbs ("`gh`-style command groups", "`gh` 스타일 명령 그룹") |
| Redundant negative contrast | "Not just for humans, but also for AI agents" | "For humans and AI agents" / "사람과 에이전트 모두를 위한" |
| Filler lead-ins | "In order to...", "It is important to note that...", "이러한 맥락에서", "결론적으로" | Delete and start with the action or fact |

## 2. Korean Translationese & AI Tells

| Category | Avoid | Prefer |
|---|---|---|
| Literal loanwords | `프리빌드 바이너리`, `에셋`, `레포 컨텍스트` | `미리 빌드된 바이너리`, `파일` (or `아카이브`), `저장소 정보` / `워크스페이스와 저장소` |
| `~에서 영감을 받은` | "`gh`에서 영감을 받아 Rust로 만든..." | "`gh` 스타일의 Rust 기반..." or "`gh`처럼 쓸 수 있는..." |
| Overuse of `~를 통해` / `~에 있어서` | "환경변수를 통해 토큰을 전달" | "환경변수로 토큰을 전달" |
| Heavy nominalization (`-적`, `-화`, `-성`) | "명시적으로 지정합니다", "기계 판독용" | "직접 지정합니다", "JSON 형식" / "자동화용" |
| Passive / inanimate subject | "토큰이 환경변수로 제공될 수 있습니다" | "환경변수로 토큰을 넘길 수 있습니다" |
| Monotonous endings | Repeating `~제공`, `~제공`, `~지원`, `~지원` | Mix concise noun phrases or active clauses naturally |
| Unnatural phrasing for technical actions | "일회성으로 적용", "활성 프로필은 유지" | "이번 명령에만 적용 (기본 프로필은 그대로)" |

## 3. Terminology Balance (Korean Technical Docs)

- **Keep in English** when standard in Korean dev workflows: `PR`, `JSON`, `stdin`, `stdout`, `stderr`, `bash`, `zsh`, `fish`, `PowerShell`, `Homebrew`, `Bitbucket Cloud`, `Basic`, `Bearer`, `raw API`, and all literal CLI names/flags.
- **Use natural Korean** for general words:
  - repository -> `저장소`
  - pipeline -> `파이프라인`
  - commit report -> `커밋 리포트`
  - prebuilt binary -> `미리 빌드된 바이너리`
  - asset (in release table) -> `파일`
  - active profile -> `활성 프로필` or `기본 프로필`
  - build metadata -> `빌드 정보`

## 4. Meaning-Preservation Diff Checklist

Before finishing any edit, verify every item:

- [ ] All CLI commands, subcommands, flags, and positional argument examples are byte-for-byte valid.
- [ ] Environment variable names, config paths, and precedence order are unchanged.
- [ ] All constraints, exclusions (e.g., "do not combine A with B"), and security cautions preserve exact polarity and strength.
- [ ] All URLs, relative doc links, and badges are intact.
- [ ] No new claims, flags, or behaviors were invented.
- [ ] English and Korean versions cover the exact same facts and caveats.
