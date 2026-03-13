# Feature 0001: Runtime-артефакты

Статус: draft
Последнее обновление: 2026-03-13

## Назначение

Этот документ фиксирует точную схему repo-local runtime-артефактов для MVP.

Цель:

- сделать runtime layout детерминированным
- сделать session-binding проверяемым
- исключить расползание временных файлов по неформальному контракту

## Корневая директория

Все runtime-артефакты MVP хранятся в:

```text
.git/.ai-teamlead/
```

## Структура директорий

Минимальная структура:

```text
.git/.ai-teamlead/
  lock/
    poll.lock
  sessions/
    <session_uuid>/
      session.json
      launch-layout.kdl
      pane-entrypoint.sh
  issues/
    <issue_number>.json
```

## Назначение директорий и файлов

### `lock/poll.lock`

Назначение:

- защита от параллельного запуска polling cycle внутри одного репозитория

### `sessions/<session_uuid>/session.json`

Назначение:

- основной durable session-binding для одной issue

Обязательные поля:

```json
{
  "session_uuid": "uuid",
  "issue_number": 123,
  "repo_root": "/abs/path/to/repo",
  "github_owner": "derived-from-git",
  "github_repo": "derived-from-git",
  "project_id": "PVT_xxx",
  "status": "active",
  "created_at": "2026-03-13T12:00:00Z",
  "updated_at": "2026-03-13T12:00:00Z",
  "zellij": {
    "session_name": "ai-teamlead",
    "tab_name": "issue-analysis",
    "session_id": "zellij-session-id",
    "tab_id": "zellij-tab-id",
    "pane_id": "zellij-pane-id"
  }
}
```

Значение `status` в MVP:

- `active`
- `waiting_for_clarification`
- `waiting_for_plan_review`
- `completed`
- `blocked`

### `sessions/<session_uuid>/launch-layout.kdl`

Назначение:

- runtime-файл launcher для запуска нужной `zellij` tab

### `sessions/<session_uuid>/pane-entrypoint.sh`

Назначение:

- тонкий runtime-shim для запуска versioned `./.ai-teamlead/launch-agent.sh`
  из корректного repo context

Инварианты:

- shim не несет branch/worktree логики
- shim может передавать служебные переменные окружения, например путь к
  бинарю `ai-teamlead`
- orchestration flow по-прежнему определяется versioned
  `./.ai-teamlead/launch-agent.sh`

### `issues/<issue_number>.json`

Назначение:

- быстрый индекс для поиска session-binding по номеру issue

Обязательные поля:

```json
{
  "issue_number": 123,
  "session_uuid": "uuid",
  "last_known_flow_status": "Waiting for Clarification",
  "updated_at": "2026-03-13T12:15:00Z"
}
```

## Инварианты

- одна issue в анализе связана ровно с одним `session_uuid`
- один `session_uuid` принадлежит ровно одной issue
- `issues/<issue_number>.json` и `sessions/<session_uuid>/session.json` не
  должны противоречить друг другу
- runtime-артефакты не являются source of truth по статусу issue в GitHub
- runtime-артефакты являются source of truth для локального session-binding и
  технических метаданных запуска

## Правила обновления

- `session.json` создается при первом успешном claim
- `issues/<issue_number>.json` обновляется при каждом изменении session-binding
  или локально известного статуса flow
- launcher layout и shim внутри `sessions/<session_uuid>/` могут
  пересоздаваться при повторном запуске

## Правила чтения для `run`

Команда `run` должна использовать runtime-артефакты так:

- для всех waiting-статусов и `Analysis Blocked` проверять наличие
  существующего session-binding
- использовать `session.json` для повторного входа в уже связанную агентскую
  сессию
- не пытаться восстанавливать диалог из отдельных JSON-артефактов

## Диагностическая ценность

Эта схема должна позволять ответить на вопросы:

- какая агентская сессия связана с issue
- в какой `zellij` panel она запущена
- какой launcher layout относится к этой сессии
- можно ли повторно войти в связанную агентскую сессию

## Открытые вопросы

- нужен ли отдельный файл `runtime.json` для агрегированного состояния daemon
- нужен ли позднее отдельный слой structured artifacts поверх истории агентской
  сессии

## Журнал изменений

### 2026-03-13

- создана точная схема runtime/session-артефактов для MVP
