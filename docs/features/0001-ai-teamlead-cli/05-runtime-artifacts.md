# Feature 0001: Runtime-артефакты

Статус: draft
Последнее обновление: 2026-03-14

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
    "session_name": "teamlead",
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

Важно не смешивать разные словари состояний:

- `session.json.status` это локальный lifecycle session-binding
- `issues/<issue_number>.json.last_known_flow_status` это последнее локально
  известное значение flow-статуса из GitHub Project
- source of truth по status issue остается в GitHub Project, а не в runtime json

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
- использовать `session.json` для reuse существующего `session_uuid`
- при повторном `run` открывать новую pane в stable `zellij` launch context, а
  не пытаться вернуть пользователя в старую pane
- не пытаться восстанавливать диалог из отдельных JSON-артефактов

## Диагностическая ценность

Эта схема должна позволять ответить на вопросы:

- какая агентская сессия связана с issue
- в какой `zellij` panel она запущена
- какой launcher layout относится к этой сессии
- можно ли повторно войти в связанную агентскую сессию

## Цепочка launcher и resurrect-артефактов

Для одной живой agent session в MVP существуют два разных слоя файлов:

1. runtime-артефакты, которые создает `ai-teamlead`
2. session snapshot-артефакты, которые создает сам `zellij`

Их не нужно смешивать.

### Таблица файлов

| Файл | Кто создает | Где лежит | Для чего нужен |
| --- | --- | --- | --- |
| `launch-layout.kdl` | `ai-teamlead` | `.git/.ai-teamlead/sessions/<session_uuid>/` | runtime-rendered layout для analysis tab конкретного запуска; строится из versioned template `.ai-teamlead/zellij/analysis-tab.kdl` |
| `pane-entrypoint.sh` | `ai-teamlead` | `.git/.ai-teamlead/sessions/<session_uuid>/` | тонкий runtime-shim; переходит в repo root, выставляет `AI_TEAMLEAD_BIN` и вызывает versioned `./.ai-teamlead/launch-agent.sh <session_uuid> <issue_url>` |
| `session.json` | `ai-teamlead` | `.git/.ai-teamlead/sessions/<session_uuid>/` | durable session-binding между issue, `session_uuid` и `zellij` identifiers |
| `session-layout.kdl` | `zellij` | `~/.cache/zellij/contract_version_1/session_info/<session_name>/` | snapshot текущего layout сессии для восстановления; отражает уже итоговый `cwd` и tab structure |
| `session-metadata.kdl` | `zellij` | `~/.cache/zellij/contract_version_1/session_info/<session_name>/` | snapshot метаданных pane/tab/session; содержит `terminal_command`, pane state, tab state и прочие runtime details |

### Разница между `launch-layout.kdl` и `session-layout.kdl`

`launch-layout.kdl`:

- это то, что `ai-teamlead` подает в `zellij` на вход
- описывает analysis tab для конкретного запуска
- рендерится из versioned project-local template
  `./.ai-teamlead/zellij/analysis-tab.kdl`
- ссылается на `pane-entrypoint.sh`

`session-layout.kdl`:

- это то, что `zellij` сам сохраняет после запуска
- описывает уже текущее состояние сессии
- может содержать другой `cwd`, чем был у стартового launcher
- используется для механизма восстановления и introspection самой сессии

### Разница между `pane-entrypoint.sh` и `terminal_command` в metadata

`pane-entrypoint.sh`:

- содержит реальные значения `session_uuid` и `issue_url`
- является нашим runtime-generated shim для конкретного запуска
- определяет, как именно начинается цепочка `launch-agent.sh`

`terminal_command` в `session-metadata.kdl`:

- это строка, которую `zellij` запомнил как команду текущей pane
- обычно это запуск `bash <abs-path-to-pane-entrypoint.sh>`
- эта metadata не обязана раскрывать дальше аргументы, которые уже зашиты в
  сам `pane-entrypoint.sh`

### Практическая цепочка запуска

Для одной issue цепочка выглядит так:

1. `ai-teamlead` читает `./.ai-teamlead/zellij/analysis-tab.kdl`.
2. `ai-teamlead` создает `launch-layout.kdl`.
3. `ai-teamlead` создает `pane-entrypoint.sh`.
4. если session отсутствует, `ai-teamlead` сначала создает базовую session.
5. `zellij` добавляет analysis tab по `launch-layout.kdl`.
6. pane запускает `pane-entrypoint.sh`.
7. `pane-entrypoint.sh` вызывает versioned
   `./.ai-teamlead/launch-agent.sh <session_uuid> <issue_url>`.
8. `launch-agent.sh` готовит worktree и запускает реального агента.
9. `zellij` сохраняет собственные snapshot-файлы:
   - `session-layout.kdl`
   - `session-metadata.kdl`

## Исключенные артефакты

Согласно [ADR-0013](../../adr/0013-agent-session-history-as-dialog-source.md),
из обязательной runtime-модели MVP исключены:

- `questions.json`
- `analysis-plan.json`
- `operator-events.jsonl`

Источником истины по диалогу является история агентской сессии, а не отдельные
JSON-файлы. Возвращение структурированных артефактов возможно в будущем как
дополнительный слой поверх истории сессии.

## Связанные решения

- [ADR-0004](../../adr/0004-runtime-artifacts-in-git-dir.md) — runtime-артефакты
  в `.git/`
- [ADR-0008](../../adr/0008-bind-issue-to-agent-session-uuid.md) — привязка
  issue к `session_uuid`
- [ADR-0013](../../adr/0013-agent-session-history-as-dialog-source.md) — история
  сессии как источник диалога
- [ADR-0014](../../adr/0014-zellij-launch-context-naming.md) — naming convention
  для `zellij` session/tab (определяет поля `zellij.*` в `session.json`)
- [ADR-0015](../../adr/0015-versioned-launch-agent-contract.md) — versioned
  launch-agent контракт

## Открытые вопросы

- нужен ли отдельный файл `runtime.json` для агрегированного состояния `ai-teamlead`
- нужен ли позднее отдельный слой structured artifacts поверх истории агентской
  сессии

## Журнал изменений

### 2026-03-13

- создана точная схема runtime/session-артефактов для MVP
