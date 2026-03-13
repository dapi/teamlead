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

- по умолчанию `session_name` формируется как `{repo_name}-ai-teamlead`

Bootstrap-правило для `tab_name`:

- по умолчанию `tab_name` равно `issue-analysis`

Где:

- `repo_name` это имя текущего git-репозитория

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

Минусы:

- при ручном удалении или resurrect `zellij` session нужен отдельный runtime
  handling
- bootstrap должен уметь вычислять `repo_name`

## Связанные документы

- [README.md](/home/danil/code/teamlead/README.md)
- [docs/features/0003-agent-launch-orchestration/README.md](/home/danil/code/teamlead/docs/features/0003-agent-launch-orchestration/README.md)
- [docs/features/0002-repo-init/README.md](/home/danil/code/teamlead/docs/features/0002-repo-init/README.md)

## Журнал изменений

### 2026-03-13

- зафиксирован naming contract для `zellij.session_name` и `zellij.tab_name`
