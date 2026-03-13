# ADR-0015: versioned `./.ai-teamlead/launch-agent.sh`

Статус: accepted
Дата: 2026-03-13

## Контекст

Нужно четко разделить три слоя:

- `issue-analysis-flow` как markdown prompt для агента
- orchestration в `ai-teamlead`, которое выбирает issue и открывает pane
- project-local launcher, который знает branch/worktree lifecycle конкретного
  репозитория

Если branch/worktree logic держать внутри core-приложения, оно слишком глубоко
знает project-specific workflow. Если генерировать runtime launcher-script в
`.git/.ai-teamlead/`, владелец репозитория теряет контролируемую versioned
точку кастомизации.

## Решение

Принимается следующий контракт:

- versioned launcher-script хранится в `./.ai-teamlead/launch-agent.sh`
- `ai-teamlead` не генерирует runtime `launch-agent.sh`
- runtime в `.git/.ai-teamlead/` хранит только технические launcher-артефакты,
  например `launch-layout.kdl`
- `poll` и `run` используют один и тот же launcher contract
- `launch-agent.sh` запускается в новой pane из корня репозитория
- первым аргументом передается `session_uuid`
- вторым аргументом передается URL issue

Ответственность `launch-agent.sh`:

- вызвать `ai-teamlead internal bind-zellij-pane <session_uuid>`
- определить или создать analysis branch/worktree
- перейти в analysis worktree
- запустить `./init.sh`, если это нужно проекту
- запустить реального агента с `issue-analysis-flow`

## Последствия

Плюсы:

- project owner получает versioned точку кастомизации launcher behavior
- branch/worktree lifecycle можно менять без перекомпиляции core-приложения
- `issue-analysis-flow` остается prompt-документом, а не orchestration-спекой
- появляется естественная точка расширения под другие мультиплексоры и
  launcher-стратегии

Минусы:

- часть behavior переносится из Rust-кода в shell-script
- нужен bootstrap этого script и отдельные тесты на launcher contract
- exact semantics branch/worktree становятся repository-specific и хуже
  унифицируются по умолчанию

## Альтернативы

### 1. Генерировать runtime `launch-agent.sh` в `.git/.ai-teamlead/`

Отклонено.

Это лишает владельца репозитория versioned contract layer для кастомизации.

### 2. Держать branch/worktree lifecycle внутри `ai-teamlead`

Отклонено.

Это слишком сильно привязывает core-приложение к конкретной git-стратегии
проекта.

## Связанные документы

- [README.md](/home/danil/code/teamlead/README.md)
- [docs/features/0002-repo-init/README.md](/home/danil/code/teamlead/docs/features/0002-repo-init/README.md)
- [docs/features/0003-agent-launch-orchestration/README.md](/home/danil/code/teamlead/docs/features/0003-agent-launch-orchestration/README.md)
- [docs/issue-analysis-flow.md](/home/danil/code/teamlead/docs/issue-analysis-flow.md)

## Журнал изменений

### 2026-03-13

- зафиксирован versioned launcher contract в `./.ai-teamlead/launch-agent.sh`
