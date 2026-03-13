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
.git/ai-teamlead/
```

## Структура директорий

Минимальная структура:

```text
.git/ai-teamlead/
  lock/
    poll.lock
  sessions/
    <session_uuid>/
      session.json
      questions.json
      analysis-plan.json
      operator-events.jsonl
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

### `sessions/<session_uuid>/questions.json`

Назначение:

- последний опубликованный набор блокирующих вопросов

Обязательные поля:

```json
{
  "session_uuid": "uuid",
  "issue_number": 123,
  "revision": 1,
  "generated_at": "2026-03-13T12:05:00Z",
  "questions": [
    {
      "id": "q1",
      "text": "..."
    }
  ]
}
```

Если вопросы еще не задавались, файл может отсутствовать.

### `sessions/<session_uuid>/analysis-plan.json`

Назначение:

- последний опубликованный пакет анализа и план

Обязательные поля:

```json
{
  "session_uuid": "uuid",
  "issue_number": 123,
  "revision": 1,
  "generated_at": "2026-03-13T12:10:00Z",
  "summary": "...",
  "scope": ["..."],
  "non_goals": ["..."],
  "assumptions": ["..."],
  "risks": ["..."],
  "open_questions": ["..."],
  "implementation_plan": ["..."],
  "feature_story": "...",
  "use_cases": ["..."]
}
```

Правила:

- для `bug` и `chore` поля `feature_story` и `use_cases` могут отсутствовать
- для `feature` они обязательны

Если план еще не публиковался, файл может отсутствовать.

### `sessions/<session_uuid>/operator-events.jsonl`

Назначение:

- append-only журнал нормализованных действий оператора

Формат:

- одна JSON-запись на строку

Минимальный формат записи:

```json
{
  "timestamp": "2026-03-13T12:15:00Z",
  "session_uuid": "uuid",
  "issue_number": 123,
  "event_type": "answers_submitted",
  "payload": {
    "source": "agent-session"
  }
}
```

Допустимые `event_type` в MVP:

- `answers_submitted`
- `plan_approved`
- `plan_revision_requested`
- `manual_retry_requested`

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
  operator event trail

## Правила обновления

- `session.json` создается при первом успешном claim
- `questions.json` перезаписывается при публикации нового набора вопросов
- `analysis-plan.json` перезаписывается при публикации нового пакета анализа
- `operator-events.jsonl` только дописывается
- `issues/<issue_number>.json` обновляется при каждом изменении session-binding
  или локально известного статуса flow

## Правила чтения для `run`

Команда `run` должна использовать runtime-артефакты так:

- для `Waiting for Clarification` искать в `operator-events.jsonl` событие
  `answers_submitted`, появившееся после последней ревизии `questions.json`
- для `Waiting for Plan Review` искать в `operator-events.jsonl` событие
  `plan_revision_requested`
- для `Analysis Blocked` принимать явный ручной retry без дополнительных
  operator events

## Диагностическая ценность

Эта схема должна позволять ответить на вопросы:

- какая агентская сессия связана с issue
- в какой `zellij` panel она запущена
- какие вопросы были последними
- какой план был последним
- какие действия оператора уже были зафиксированы

## Открытые вопросы

- нужен ли отдельный файл `runtime.json` для агрегированного состояния daemon
- нужно ли хранить несколько ревизий вопросов и планов вместо only-last schema

## Журнал изменений

### 2026-03-13

- создана точная схема runtime/session-артефактов для MVP
