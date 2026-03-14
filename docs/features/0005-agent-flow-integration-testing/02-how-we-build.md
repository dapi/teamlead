# Feature 0005: Как строим

## Архитектура

Платформа состоит из пяти слоев:

1. `host entrypoint`
   Единая CLI-команда, например `ai-teamlead test agent-flow`, которая
   загружает repo-local config, выбирает сценарий и orchestrate-ит запуск.
2. `sandbox builder`
   Подготавливает disposable Docker sandbox с pinned `zellij`, нужными CLI и
   volumes для текущего проекта и разрешенных host-level agent файлов.
3. `agent bridge`
   Монтирует в sandbox через volumes только текущий проект и нужные host-level
   agent files для выбранного profile: например `~/.codex`, `~/.claude` и
   другие явно разрешенные config/auth artifacts из того же окружения, где
   запущен test suite.
4. `scenario runner`
   Запускает внутри sandbox `ai-teamlead`, `launch-agent.sh`, `gh` stub и
   assertion hooks.
5. `artifact collector`
   Выгружает наружу логи, runtime-state, stdout/stderr, metadata сценария и
   итоговый verdict.

Канонический path должен выглядеть так:

1. host CLI читает `./.ai-teamlead/settings.yml`
2. host CLI читает versioned scenario manifest
3. host CLI собирает sandbox и монтирует текущий проект read-only и
   allowlisted host-level files через volumes
4. sandbox materialize-ит writable workspace внутри контейнера из смонтированного
   проекта
5. sandbox запускает `ai-teamlead` entrypoint
6. `ai-teamlead` проходит обычный launcher/orchestration path
7. выбранный agent profile (`stub`, `codex`, `claude`) отрабатывает внутри того
   же sandbox
8. все обращения `ai-teamlead` к GitHub проходят через `gh` stub и пишутся в
   invocation log
9. runner выполняет assertions
10. artifact collector сохраняет результат вне sandbox

## Границы sandbox

Sandbox должен быть не просто disposable container, а явно ограниченной
execution surface.

### Sandbox имеет доступ к

- текущему проекту внутри контейнера через read-only volume mount
- `~/.codex`, `~/.claude` и другим нужным для выбранного agent profile
  host-level config/auth файлам, смонтированным через явный allowlist volumes
- writable workspace внутри контейнера, созданному runner-ом из read-only
  project mount
- sandbox-local temporary directories и runtime-artifacts
- export-каталогу артефактов, заранее выделенному runner-ом
- встроенному `gh` stub вместо реального `gh`

### Sandbox не имеет доступа к

- host `zellij` session, tab, pane и socket
- произвольным путям пользователя вне явно разрешенных volumes
- произвольным путям в `$HOME`, не перечисленным в allowlist mounts
- host git credentials, GitHub credentials и реальному `gh` binary
- реальному GitHub API и GitHub web endpoints во время тестового прогона

Нарушение любой из этих границ должно считаться ошибкой конфигурации или
`preflight failed`, а не неявным fallback.

## Данные и состояния

### Сущности

- `test run`
  Отдельный запуск entrypoint с уникальным `run_id`.
- `scenario`
  Versioned описание одного integration path со входными данными,
  environment bridge и assertions.
- `sandbox`
  Disposable container runtime и его filesystem.
- `workspace snapshot`
  Writable sandbox workspace, созданный внутри контейнера из read-only mount
  текущего проекта.
- `agent profile`
  Набор правил, как запускать `stub`, `codex` или `claude`.
- `artifact bundle`
  Экспортируемый результат теста.
- `gh invocation log`
  Журнал всех вызовов `gh` stub внутри sandbox.

### Жизненный цикл `test run`

- `created`
- `snapshot_prepared`
- `sandbox_ready`
- `preflight_failed`
- `runtime_started`
- `agent_running`
- `asserting`
- `passed | failed | errored`
- `artifacts_exported`

Переходы должны быть линейными и диагностируемыми. Повторный запуск создает
новый `run_id` и не переиспользует mutable state прошлого прогона.

`preflight_failed` фиксирует отдельный класс завершения до старта runtime path:
например, отсутствующий agent binary, недостающий allowlisted credential или
нарушение sandbox policy, обнаруженное на этапе подготовки.

### Workspace snapshot

В MVP sandbox должен получать через volume mount:

- текущее содержимое репозитория
- versioned `.ai-teamlead/`, `.claude/`, `.codex/`, если они есть в repo
- достаточный git context для работы `ai-teamlead`, `git worktree` и launcher

Платформа должна поддержать локальную разработку, поэтому sandbox должен видеть
актуальное состояние текущего проекта, а не только последний commit.
Предпочтительный контракт:

- sandbox получает текущий working tree через read-only volume mount
- дополнительные host-level agent files (`~/.codex`, `~/.claude` и другие
  allowlisted paths) также пробрасываются через volumes
- runner materialize-ит из read-only mount отдельный writable workspace внутри
  контейнера
- все git/runtime побочные эффекты пишутся только во внутренний sandbox
  workspace, а не в host repo

## Интерфейсы

### CLI entrypoint

Черновой контракт:

```bash
ai-teamlead test agent-flow \
  --scenario run-happy-path \
  --agent codex \
  --mode live
```

Минимальные аргументы первой версии:

- `--scenario <name>`
- `--agent <stub|codex|claude>`
- `--mode <stub|live>`

Дополнительные аргументы:

- `--keep-sandbox`
- `--artifacts-dir <path>`
- `--timeout-seconds <n>`
- `--no-build`

Правила:

- `--mode stub` разрешает только `--agent stub`
- `--mode live` разрешает `codex` и `claude`
- если `--agent` не задан, default live profile = `codex`
- канонический namespace первой версии: `ai-teamlead test agent-flow`
- итоговый exit code отражает verdict сценария

### Scenario manifest

Scenario manifest должен быть versioned и лежать внутри репозитория. Черновой
формат:

```yaml
name: run-happy-path
description: Run issue-analysis flow in isolated sandbox
mode: stub
agent: stub
fixtures:
  github_stub: basic-backlog.json
  repo_state: clean
commands:
  - ai-teamlead run https://github.com/org/repo/issues/123
assertions:
  - type: exit_code
    equals: 0
  - type: issue_status
    equals: Waiting for Plan Review
  - type: file_exists
    path: specs/issues/123/README.md
  - type: gh_call
    command: project item-edit
```

Scenario не должен содержать secrets. Он описывает:

- какие fixtures нужны
- какой agent profile используется
- какие assertions обязательны
- какой `gh` stub fixture и какой expected invocation log нужны
- какой cleanup и artifact export ожидаются

CLI и manifest должны валидироваться совместно:

- если `--agent` или `--mode` противоречат manifest, runner завершается ошибкой
  валидации
- versioned manifest остается источником истины для test intent

### Agent bridge

Bridge должен быть явным и profile-based.

Для каждого agent profile задаются:

- `env_allowlist`
- `file_mounts`
- `binary_resolution`
- `preflight_checks`

Примеры допустимых данных bridge:

- env vars вида `OPENAI_API_KEY`, `OPENAI_BASE_URL`, `ANTHROPIC_API_KEY`
- user-local config dirs, auth/session files или account-state files для
  конкретного агента
- repo-local `.claude/` и `.codex/`

Bridge обязан брать значения из host environment и host config, с которыми
запущен test suite, а не из отдельного скрытого тестового профиля.

Недопустимо:

- монтировать весь `$HOME` целиком
- сохранять forwarded secrets в artifact bundle
- делать implicit fallback на host filesystem вне allowlist

### GitHub stub

GitHub слой для integration tests должен реализовываться через собственный
`gh` stub внутри sandbox.

Требования к `gh` stub:

- `PATH` внутри sandbox должен разрешать `gh` в пользу stub, а не реального CLI
- stub получает versioned fixture из scenario manifest
- stub логирует каждый вызов:
  `argv`, `cwd`, время, exit code, stdout/stderr metadata
- лог `gh` stub попадает в artifact bundle
- assertions могут проверять как возвращенные данные, так и факт вызова
  конкретных `gh` команд
- обращение к реальному GitHub вместо stub считается ошибкой тестовой среды

### Stub agent

`stub`-agent нужен не как отдельный shortcut, а как controlled implementation
того же agent contract. Он должен:

- стартовать через тот же launcher path
- получать тот же prompt context
- уметь выполнить заранее заданный сценарный outcome:
  `plan-ready`, `needs-clarification`, `blocked`
- вызывать те же внутренние команды завершения стадии

## Технические решения

- Канонический sandbox для MVP: Docker-based headless runtime.
- Канонический `zellij` внутри sandbox: pinned version по ADR-0011.
- Live и stub режимы используют один и тот же sandbox entrypoint.
- Default live path для локального тестирования: `codex`.
- `claude` поддерживается как дополнительный live-profile, в том числе для
  Claude Code с моделью класса Sonnet.
- Текущий проект монтируется в sandbox read-only; writable execution path живет
  во внутреннем sandbox workspace.
- GitHub integration в sandbox всегда проходит через `gh` stub и invocation log.
- Вердикт сценария считается по assertions, а не по одному exit code процесса.
- Артефакты должны собираться вне зависимости от `passed` или `failed`.
- Sandbox должен быть disposable по умолчанию; сохранение возможно только через
  явный флаг `--keep-sandbox`.

## Конфигурация

Глобальные repo-local defaults логично хранить в `./.ai-teamlead/settings.yml`
в новой секции `integration_tests.agent_flow`.

Черновая схема:

```yaml
integration_tests:
  agent_flow:
    sandbox_runtime: docker
    image: ai-teamlead-agent-flow-test:local
    default_timeout_seconds: 900
    artifacts_dir: ".git/.ai-teamlead/test-runs"
    scenario_root: ".ai-teamlead/tests/agent-flow"
    github:
      mode: stub
      log_path: "logs/gh-invocations.jsonl"
    agent_profiles:
      codex:
        mode: live
        default: true
        env_allowlist:
          - OPENAI_API_KEY
          - OPENAI_BASE_URL
        file_mounts:
          - "~/.codex"
      claude:
        mode: live
        env_allowlist:
          - ANTHROPIC_API_KEY
          - ANTHROPIC_BASE_URL
        file_mounts:
          - "~/.claude"
          - "~/.config/claude"
        model_family: sonnet
      stub:
        mode: stub
        env_allowlist: []
        file_mounts: []
```

Правила:

- без `integration_tests.agent_flow` entrypoint использует встроенные safe
  defaults
- встроенный default live profile = `codex`
- для `codex` и `claude` sandbox использует те же host-level env vars и
  volume-mounted account/config files, с которыми запущен test suite
- встроенный GitHub mode = `stub`
- secrets и значения токенов не хранятся в versioned YAML
- в config хранятся только имена env vars, пути mounts и runtime defaults

## Ограничения реализации

- В первой версии допускается только один sandbox backend: Docker.
- В первой версии допустим только Linux-oriented headless path.
- В первой версии GitHub взаимодействие допустимо только через `gh` stub;
  прямой real GitHub path считается out of contract.
- В первой версии live assertions должны проверять orchestration и артефакты, а
  не semantic quality generated текста.
- Если agent CLI отсутствует или в sandbox не проброшены нужные env/config
  volumes для выбранного agent profile, сценарий должен завершаться явным
  `preflight failed`, а не неявным timeout.

## Связанные документы

- [README.md](./README.md)
- [01-what-we-build.md](./01-what-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../0003-agent-launch-orchestration/02-how-we-build.md](../0003-agent-launch-orchestration/02-how-we-build.md)
