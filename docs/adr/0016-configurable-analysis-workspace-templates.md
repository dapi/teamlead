# ADR-0016: configurable analysis workspace templates in `settings.yml`

Статус: accepted
Дата: 2026-03-13

## Контекст

`launch-agent.sh` должен создавать branch/worktree и versioned analysis-артефакты
до старта реального агента. При этом naming и layout этих сущностей зависят от
конкретного проекта и не должны быть захардкожены в core-коде `ai-teamlead`.

## Решение

В `./.ai-teamlead/settings.yml` добавляется секция:

```yaml
launch_agent:
  analysis_branch_template: "analysis/issue-${ISSUE_NUMBER}"
  worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  analysis_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
```

Назначение полей:

- `analysis_branch_template` задает naming analysis branch
- `worktree_root_template` задает root path для analysis worktree
- `analysis_artifacts_dir_template` задает repo-relative каталог для versioned
  analysis-артефактов

Bootstrap defaults:

- branch: `analysis/issue-${ISSUE_NUMBER}`
- worktree root: `${HOME}/worktrees/${REPO}/${BRANCH}`
- artifacts dir: `specs/issues/${ISSUE_NUMBER}`

## Последствия

Плюсы:

- владелец репозитория может менять naming без правки core-кода
- появляется единый versioned источник настройки analysis workspace contract
- `launch-agent.sh` получает явный конфигурационный слой для branch/worktree

Минусы:

- launcher-script должен поддержать интерполяцию template variables
- появляется еще один кусок конфигурационного контракта, который нужно
  валидировать и документировать

## Альтернативы

### 1. Захардкодить naming в `launch-agent.sh`

Отклонено.

Это плохо переносится между репозиториями.

### 2. Хранить naming только в shell-script без `settings.yml`

Отклонено.

Это делает настройки менее discoverable и хуже поддающимися валидации.

## Связанные документы

- [README.md](/home/danil/code/teamlead/README.md)
- [docs/features/0003-agent-launch-orchestration/README.md](/home/danil/code/teamlead/docs/features/0003-agent-launch-orchestration/README.md)
- [docs/adr/0015-versioned-launch-agent-contract.md](/home/danil/code/teamlead/docs/adr/0015-versioned-launch-agent-contract.md)

## Журнал изменений

### 2026-03-13

- добавлены configurable templates для analysis branch, worktree root и
  artifacts dir
