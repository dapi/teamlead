# Конфигурация

## Назначение

Этот документ фиксирует repo-level контракт для `./.ai-teamlead/settings.yml`.

`README.md` оставляет только краткий overview и ссылки. Детали конфигурации
живут здесь как в более узком каноническом документе.

## Где лежит конфиг

- файл должен называться `settings.yml`
- файл должен лежать в `./.ai-teamlead/` целевого репозитория
- `ai-teamlead` всегда читает конфиг из текущего репозитория, а не из
  глобального пользовательского state

Следствие:

- разные репозитории могут иметь разные `./.ai-teamlead/settings.yml`
- у каждого репозитория может быть свой независимый экземпляр `ai-teamlead`
- GitHub owner/repo для MVP берутся из текущего git-репозитория и не
  переопределяются через config

## Минимальный active override

Минимальный обязательный active YAML для MVP:

```yaml
github:
  project_id: "PVT_xxx"
```

`github.project_id` остается `required-without-default` полем.

## Comment-only bootstrap template

Остальные MVP-поля приложение может брать из canonical runtime defaults.
Поэтому `templates/init/settings.yml` и сгенерированный bootstrap допускаются
как comment-only template, где documented defaults показаны в закомментированном
виде.

Bootstrap overview:

```yaml
# issue_analysis_flow:
#   statuses:
#     backlog: "Backlog"
#     analysis_in_progress: "Analysis In Progress"
#     waiting_for_clarification: "Waiting for Clarification"
#     waiting_for_plan_review: "Waiting for Plan Review"
#     ready_for_implementation: "Ready for Implementation"
#     analysis_blocked: "Analysis Blocked"
#
# issue_implementation_flow:
#   statuses:
#     ready_for_implementation: "Ready for Implementation"
#     implementation_in_progress: "Implementation In Progress"
#     waiting_for_ci: "Waiting for CI"
#     waiting_for_code_review: "Waiting for Code Review"
#     implementation_blocked: "Implementation Blocked"
#     done: "Done"
#
# poll:
#   assignee_filter: "$me"
#
# runtime:
#   max_parallel: 1
#   poll_interval_seconds: 3600
#
# zellij:
#   session_name: "${REPO}"
#   tab_name: "issue-analysis"
#   launch_target: "tab"
#   tab_name_template: "#${ISSUE_NUMBER}"
#   layout: "compact"
#
# launch_agent:
#   analysis_branch_template: "analysis/issue-${ISSUE_NUMBER}"
#   worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
#   analysis_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
#   global_args:
#     claude:
#       - "--permission-mode"
#       - "auto"
#     codex:
#       - "--ask-for-approval"
#       - "never"
#       - "--sandbox"
#       - "workspace-write"
#   implementation_branch_template: "implementation/issue-${ISSUE_NUMBER}"
#   implementation_worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
#   implementation_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
```

## Launcher-поля `zellij`

`zellij.session_name` задает versioned fallback для target session.

Правила:

- documented default хранится как template `${REPO}`
- runtime рендерит `${REPO}` из canonical GitHub repo slug
- порядок выбора session:
  `--zellij-session` -> `ZELLIJ_SESSION_NAME` -> `zellij.session_name`
- для `zellij.session_name` разрешен только placeholder `${REPO}`
- literal-значения без placeholder допустимы
- если после рендера остаются `${...}`, это ошибка конфигурации

`zellij.tab_name` задает stable shared tab.

`zellij.launch_target` задает default launcher mode внутри выбранной session.

Правила:

- поддерживаются только `pane` и `tab`
- runtime default при отсутствии поля = `tab`
- public CLI override есть только у `run`:
  `--launch-target <pane|tab>`
- precedence:
  `run --launch-target` -> `zellij.launch_target` -> runtime default `tab`
- `poll` и `loop` остаются config-driven и не имеют public override

Семантика:

- `pane`:
  - переиспользует stable shared tab `zellij.tab_name`
  - при отсутствии shared tab создает его через versioned
    `.ai-teamlead/zellij/analysis-tab.kdl`
  - при duplicate tabs завершает запуск ошибкой
- `tab`:
  - создает отдельный analysis tab
  - effective имя tab по умолчанию рендерится из `#${ISSUE_NUMBER}`
  - если задан `zellij.tab_name_template`, используется явный override из
    конфига

`zellij.tab_name_template`:

- поле с application default `#${ISSUE_NUMBER}`
- влияет только на `tab`-режим
- поддерживает только `${ISSUE_NUMBER}`
- не меняет semantics stable `zellij.tab_name`

`zellij.layout`:

- optional поле
- используется только при создании новой session
- остается `example-only extension`, а не обязательным active override

## Defaulted-by-application поля

- `issue_analysis_flow.statuses.*`
- `issue_implementation_flow.statuses.*`
- `runtime.max_parallel`
- `runtime.poll_interval_seconds`
- `zellij.session_name`
- `zellij.tab_name`
- `zellij.launch_target`
- `zellij.tab_name_template`
- `launch_agent.analysis_branch_template`
- `launch_agent.worktree_root_template`
- `launch_agent.analysis_artifacts_dir_template`
- `launch_agent.implementation_branch_template`
- `launch_agent.implementation_worktree_root_template`
- `launch_agent.implementation_artifacts_dir_template`

## Example-only extension поля

- `poll.assignee_filter`
- `zellij.layout`

Эти поля показываются в bootstrap template как opt-in examples и не обязаны
появляться в active YAML.

## `poll.assignee_filter`

`poll.assignee_filter` задает opt-in фильтрацию backlog по assignee для
`poll` и `loop`.

Поддерживаемые режимы:

- поле не задано: поведение `poll` не меняется, фильтра по assignee нет;
- `"$me"`: backlog фильтруется по текущему GitHub-пользователю, которого
  runtime резолвит через `gh api user --jq '.login'`;
- `"username"`: backlog фильтруется по указанному login.

Правила:

- поле не влияет на ручной `run`;
- `"$me"` должен резолвиться один раз на запуск `poll` или на жизнь процесса
  `loop`;
- issue без assignee не матчится, если фильтр задан;
- если у issue несколько assignees, достаточно совпадения хотя бы одного login.

## Launch agent templates

`launch_agent.*` задают versioned naming/path contract для:

- branch
- worktree root
- artifacts dir
- agent global args

Поддерживаемые placeholder-переменные в первой версии:

- `${HOME}`
- `${REPO}`
- `${ISSUE_NUMBER}`
- `${BRANCH}`

Для `launch_agent.global_args.*` действуют отдельные правила:

- значения задаются как список строк, а не как raw shell string
- отсутствие пользовательского override означает application defaults
- canonical defaults:
  - `codex`: `["--ask-for-approval", "never", "--sandbox", "workspace-write"]`
  - `claude`: `["--permission-mode", "auto"]`

## Выбор агента в runtime

`launch-agent.sh` определяет, какой агент запускается для coding session:

- если в системе доступен `codex`, launcher запускает `codex`
- если `codex` недоступен, но доступен `claude`, launcher запускает `claude`
- если ни один из агентов недоступен, launcher оставляет shell внутри
  подготовленного worktree

`launch_agent.global_args` позволяет задать аргументы для каждого агента
раздельно. При отсутствии пользовательского override применяются application
defaults:

- `codex`: `["--ask-for-approval", "never", "--sandbox", "workspace-write"]`
- `claude`: `["--permission-mode", "auto"]`

## Runtime-последствия

Для MVP durable-связка между issue и агентской сессией хранится в
repo-local runtime-артефактах внутри `.git/.ai-teamlead/`.

Связанные документы:

- [./features/0001-ai-teamlead-cli/README.md](./features/0001-ai-teamlead-cli/README.md)
- [./features/0003-agent-launch-orchestration/README.md](./features/0003-agent-launch-orchestration/README.md)
- [./adr/0001-repo-local-ai-config.md](./adr/0001-repo-local-ai-config.md)
- [./adr/0032-zellij-launch-target-pane-tab.md](./adr/0032-zellij-launch-target-pane-tab.md)

## Zero-config defaults

Все поля из секции «Defaulted-by-application» имеют canonical runtime defaults,
встроенные в приложение. Это означает:

- оператору достаточно указать только `github.project_id` в active YAML;
- остальные поля приложение заполняет собственными defaults;
- bootstrap template (`templates/init/settings.yml`) содержит все defaults
  в закомментированном виде для наглядности;
- раскомментирование поля в `settings.yml` переопределяет application default.

Связанный ADR:
[ADR-0033](./adr/0033-zero-config-settings-template-and-runtime-default-layer.md).

## Журнал изменений

### 2026-03-15

- добавлен журнал изменений
- документ приведён к стандарту SSOT
- добавлена секция «Zero-config defaults» (ADR-0033)
