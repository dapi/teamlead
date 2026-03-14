# ADR-0014: Naming contract для `zellij` launch context

Статус: accepted
Дата: 2026-03-13

## Контекст

`ai-teamlead` использует `zellij` как interactive launcher context для
агентских сессий.

Нужно было зафиксировать:

- какие identifiers считаются стабильными и versioned
- какие identifiers являются runtime-only
- как bootstrap должен задавать launcher context для конкретного репозитория

## Решение

В versioned project-local конфиге фиксируются только стабильные semantic names:

- `zellij.session_name`
- `zellij.tab_name`

Bootstrap-правило для `session_name`:

- по умолчанию `session_name` хранится как template `${REPO}`

Bootstrap-правило для `tab_name`:

- по умолчанию `tab_name` равно `issue-analysis`

Где:

- `${REPO}` рендерится во время запуска из canonical GitHub repo slug,
  полученного из `origin`
- literal-значения `session_name` без placeholder остаются допустимыми

Runtime `zellij` identifiers не хранятся в versioned config:

- `session_id`
- `tab_id`
- `pane_id`

Они определяются во время запуска и сохраняются только в runtime state.

## Последствия

Плюсы:

- bootstrap дает предсказуемый launcher context для каждого репозитория
- разные репозитории не конфликтуют по имени `zellij` session
- versioned config не загрязняется runtime identifiers
- `session_name` использует тот же repo identifier, что и `launch_agent.*`

Минусы:

- при ручном удалении или resurrect `zellij` session нужен отдельный runtime
  handling
- runtime должен валидировать, что в `session_name` не остались
  неразрешенные `${...}`

## Связанные документы

- [README.md](/home/danil/code/teamlead/README.md)
- [docs/features/0003-agent-launch-orchestration/README.md](/home/danil/code/teamlead/docs/features/0003-agent-launch-orchestration/README.md)
- [docs/features/0002-repo-init/README.md](/home/danil/code/teamlead/docs/features/0002-repo-init/README.md)
- [docs/adr/0012-repo-init-command-and-project-contract-layer.md](/home/danil/code/teamlead/docs/adr/0012-repo-init-command-and-project-contract-layer.md)
- [docs/adr/0015-versioned-launch-agent-contract.md](/home/danil/code/teamlead/docs/adr/0015-versioned-launch-agent-contract.md)
- [docs/adr/0016-configurable-analysis-workspace-templates.md](/home/danil/code/teamlead/docs/adr/0016-configurable-analysis-workspace-templates.md)

## Журнал изменений

### 2026-03-13

- зафиксирован naming contract для `zellij.session_name` и `zellij.tab_name`

### 2026-03-14

- bootstrap placeholder `__SESSION_NAME__` удален из init-шаблона
- default для `zellij.session_name` переведен на `${REPO}`
- рендеринг `zellij.session_name` унифицирован с общим template contract
