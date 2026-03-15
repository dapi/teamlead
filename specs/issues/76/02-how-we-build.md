# Issue 76: Как строим

Статус: draft
Последнее обновление: 2026-03-15

## Approach

Базовый подход: разделить текущий `complete-stage` на два контура.

### 1. Agent-side completion request

Агентская команда `ai-teamlead internal complete-stage` сохраняется как
канонический prompt-level entrypoint, но меняет поведение:

- валидирует локально только базовые поля (`session_uuid`, `stage`, `outcome`,
  непустой `message`);
- записывает machine-readable completion request в worktree-local mailbox,
  доступный из sandbox;
- не выполняет `git add`, `git commit`, `git push`, `gh pr create`,
  `gh pr ready`, `gh issue close` и update GitHub Project status;
- завершает stage signal-only semantics и возвращает понятную диагностику.

### 2. Host-side supervisor execution

Host-side supervisor запускается вне sandbox агента тем же orchestration path,
который стартовал agent session.

Минимальный runtime path:

1. `launch-agent.sh` перестает `exec`-ить агента и вместо этого запускает его
   как дочерний процесс.
2. После завершения агента launcher вызывает host-only internal command
   supervisor-а.
3. Supervisor читает trusted session manifest из `.git/.ai-teamlead/` и
   completion request из worktree-local mailbox.
4. Supervisor валидирует request против trusted manifest и stage-specific rules.
5. Только после успешной валидации он выполняет git/gh/project side effects,
   пишет audit trail и помечает request как примененный.

Этот вариант сохраняет единый prompt contract, но переносит execution
boundary туда, где уже есть доступ к primary repo root, git metadata и host
credentials.

## Affected Areas

- `src/complete_stage.rs`
  нужно разделить на agent-side request writing и host-side executor logic;
- новый supervisor/service layer в Rust
  должен инкапсулировать validation, audit trail, replay и side effects;
- `src/runtime.rs`
  должен получить хранение audit metadata и trusted pointers на stage workspace,
  не превращая request mailbox в source of truth;
- `.ai-teamlead/launch-agent.sh`
  должен стать parent/supervisor wrapper, а не просто `exec`-shim для агента;
- `.ai-teamlead/flows/issue-analysis-flow.md`
  и `.ai-teamlead/flows/issue-implementation-flow.md`
  должны сохранить вызов `complete-stage`, но описывать host-supervised
  semantics;
- `docs/issue-analysis-flow.md` и `docs/issue-implementation-flow.md`
  должны обновить finalization contract и failure semantics;
- `docs/features/0003-agent-launch-orchestration/*`
  нужно синхронизировать по launcher/supervisor boundary;
- `docs/features/0006-public-repo-security/*` и
  `docs/untrusted-input-security.md`
  должны явно сослаться на supervisor как trusted publication boundary;
- `.gitignore`, bootstrap template и init path
  должны получить игнорируемый worktree-local mailbox directory;
- integration/unit tests
  должны покрыть новый supervisor path и regression по existing outcomes.

## Interfaces And Data

### Agent-side request payload

Completion request должен быть минимальным и не нести лишнего authority.

Минимальный payload:

```json
{
  "schema_version": 1,
  "session_uuid": "9e388d33-2c29-4ccd-addd-9b07c1607e2c",
  "stage": "analysis",
  "outcome": "plan-ready",
  "message": "SDD-комплект собран",
  "requested_at": "2026-03-15T21:00:00+03:00"
}
```

Принципиально не должны доверяться данным из request:

- branch;
- repo root;
- project id;
- target status;
- artifacts path;
- разрешенный набор side effects.

Все эти значения supervisor обязан брать из trusted runtime manifest и
stage-aware config/runtime defaults.

### Worktree-local mailbox

Поскольку sandboxed linked worktree не должен зависеть от записи в
`.git/worktrees/...`, request должен жить в отдельном worktree-local transient
каталоге, например:

```text
.ai-teamlead-local/stage-completion/<session_uuid>.json
```

Требования:

- каталог должен быть игнорируемым для git и не попадать в versioned artifacts;
- mailbox используется только как transport между sandbox и host;
- после успешного применения request supervisor удаляет или архивирует файл;
- при ошибке request остается на месте для replay.

### Trusted host-side state

Trusted source of truth остается в `.git/.ai-teamlead/`:

- `session.json` хранит issue/stage/repo/project binding;
- issue index хранит last known flow status только как runtime cache;
- host-side audit trail хранится рядом с session metadata, а не в worktree.

### Host-side audit trail

Для каждой попытки supervisor должен писать отдельный audit artifact, например:

```text
.git/.ai-teamlead/sessions/<session_uuid>/completion-attempts/<timestamp>.json
```

Минимум в audit entry:

- digest или path принятого request;
- validated stage/outcome;
- какие side effects были запланированы;
- какие side effects выполнены фактически;
- commit SHA / PR URL / target status при наличии;
- warnings и failure reason;
- timestamp и result (`applied`, `partial`, `failed`, `replayed`).

### Recovery interface

Нужен явный host-side replay path, который не зависит от повторного запуска
агента. Минимально допустимый вариант:

- отдельная internal command для повторной обработки последнего pending request
  по `session_uuid`.

Это позволяет безопасно доигрывать случаи, когда:

- request уже записан, но supervisor упал до `git push`;
- `gh` временно недоступен;
- status transition не применился после уже выполненного commit.

## Configuration And Runtime Assumptions

- новый public config для пользователя не обязателен, если mailbox path можно
  deterministically вывести из worktree root;
- launcher должен экспортировать agent-у путь до request file через env var,
  чтобы prompt-level команда не вычисляла его эвристически;
- stage branch, worktree root и artifacts dir уже должны быть зафиксированы в
  trusted runtime manifest до запуска supervisor;
- analysis и implementation продолжают использовать один completion vocabulary,
  включая `merged` для implementation finalization;
- `merged` outcome остается GitHub-first path и валидируется host-side
  supervisor-ом через текущие правила ADR-0028.

## Risks

- если agent завершился без request file, issue останется в активном статусе и
  потребует operator follow-up;
- если mailbox directory не будет надежно игнорироваться, локальный worktree
  получит лишний грязный runtime мусор;
- перенос логики в launcher wrapper повышает требования к корректной обработке
  exit code, signal handling и replay after crash;
- частичная миграция без обновления SSOT/ADR создаст двойной контракт
  `complete-stage`;
- public-safe policy может потребовать дополнительных gates для host-side
  publication actions поверх базового supervisor path.

## Architecture Notes

### Почему supervisor должен жить на host-side orchestration path

Уже существующий `launch-agent.sh` знает:

- `session_uuid`;
- issue/stage context;
- worktree root;
- путь к проектному `ai-teamlead` binary.

Именно этот слой естественно становится parent-process для sandboxed агента.
Добавление отдельного постоянного daemon-а для MVP не нужно: достаточно
supervised parent/child lifecycle в существующем launcher path.

### Почему request должен быть minimal-authority

Agent session обрабатывает hostile input и не должна приносить в trusted слой
собственный branch/status contract.

Поэтому request сообщает только:

- что агент считает результатом stage;
- какое краткое user-facing сообщение нужно использовать.

Все остальное supervisor выводит из trusted manifest, config и GitHub observed
state.

## ADR Impact

Изменение требует нового ADR уровня execution/runtime boundary.

Этот ADR должен зафиксировать:

- host-side supervisor как канонический privileged execution boundary;
- agent-side `complete-stage` как signal-only contract;
- worktree-local mailbox как transport-only layer;
- audit/replay semantics;
- supersede privileged-execution часть
  [ADR-0020](../../../docs/adr/0020-agent-session-completion-signal.md);
- уточнение роли
  [ADR-0026](../../../docs/adr/0026-stage-aware-complete-stage.md):
  stage-aware vocabulary сохраняется, но сам privileged apply path переезжает в
  supervisor.

## Alternatives Considered

### 1. Сохранить прямой `git`/`gh` path из sandbox и расширить полномочия агента

Отклонено.

Это не решает конфликт с linked worktree metadata и ухудшает security posture
для public repos.

### 2. Пусть launcher угадывает outcome по наличию файлов

Отклонено.

Outcome знает только агент; угадывание по artifacts ломает distinction между
`plan-ready`, `needs-clarification`, `blocked` и implementation outcomes.

### 3. Выделить отдельный long-running daemon supervisor

Отклонено для MVP.

Это добавляет лишний persistent state и operator surface без необходимости:
существующий launcher уже может быть parent/supervisor process.

## Migration Or Rollout Notes

- сначала нужно обновить SSOT, feature-docs и принять новый ADR;
- затем реализовать supervisor path, сохранив prompt-level вызов
  `internal complete-stage`;
- после этого обновить integration scenarios на sandboxed linked-worktree path;
- переход считается завершенным только когда `workspace-write` снова проходит
  stage finalization без `danger-full-access`.
