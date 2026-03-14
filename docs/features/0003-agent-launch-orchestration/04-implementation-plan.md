# Feature 0003: План реализации

Статус: draft
Последнее обновление: 2026-03-14

## Назначение

Этот документ задает порядок реализации orchestration flow запуска агента.

В отличие от `02-how-we-build.md`, этот документ описывает не целевую
архитектуру, а последовательность практических шагов реализации.

## Зависимости

- Feature 0001 (CLI, poll, run) — orchestration использует launch path,
  который вызывается из `poll` и `run`
- Feature 0002 (repo-init) — `launch-agent.sh` и `settings.yml` создаются
  командой `init`

## Порядок работ

### Этап 1. Zellij session/tab management

Цель:

- реализовать поиск или создание `zellij` session по `session_name`
- реализовать поиск или создание tab по `tab_name`
- реализовать открытие новой pane

Результат этапа:

- `ai-teamlead` умеет находить или создавать zellij context
- обработаны corner cases 1-6 из `02-how-we-build.md`
- при неоднозначном tab context запуск завершается ошибкой

### Этап 2. Session binding и runtime-артефакты запуска

Цель:

- генерировать `session_uuid` при claim
- создавать `session.json` и `issues/<issue_number>.json`
- генерировать `launch-layout.kdl` и `pane-entrypoint.sh`

Результат этапа:

- после claim существует durable session-binding
- launcher layout и shim готовы для передачи в `zellij`
- файлы лежат в `.git/.ai-teamlead/sessions/<session_uuid>/`

### Этап 3. Единый launch path для poll и run

Цель:

- реализовать общий launch path, который используют и `poll`, и `run`
- launch path: claim → session binding → zellij pane → `launch-agent.sh`

Результат этапа:

- `poll` после выбора issue использует тот же код запуска, что и `run`
- `launch-agent.sh` вызывается с корректными аргументами
  (`session_uuid`, `issue_url`)

### Этап 4. Команда `bind-zellij-pane`

Цель:

- реализовать `ai-teamlead internal bind-zellij-pane <session_uuid>`
- команда читает `ZELLIJ_PANE_ID` из окружения
- дописывает `pane_id` в `session.json`

Результат этапа:

- после старта pane в runtime state записан `pane_id`
- связка issue → session → zellij pane полностью прослеживаема

### Этап 5. Интеграция с `launch-agent.sh`

Цель:

- убедиться, что `pane-entrypoint.sh` корректно вызывает versioned
  `./.ai-teamlead/launch-agent.sh`
- launcher получает `session_uuid` и `issue_url`
- launcher запускается из корня репозитория как `cwd`

Результат этапа:

- end-to-end цепочка от `run`/`poll` до реального агента работает
- `launch-agent.sh` готовит worktree и стартует агент

### Этап 6. Smoke verification

Цель:

- пройти сценарии 1-8 из `03-how-we-verify.md`

Минимальный набор:

- `run` при отсутствии session создает session/tab/pane
- `run` при существующей session открывает новую pane
- `poll` использует тот же launch path
- несколько tab с одинаковым именем → ошибка
- `codex` недоступен → пользователь получает shell в worktree
- измененные templates в `settings.yml` подхватываются

## Критерий завершения

Feature можно считать реализованной, если:

- `poll` и `run` используют единый launch path
- zellij session/tab создаются или переиспользуются по `session_name`/`tab_name`
- `launch-agent.sh` вызывается с корректными аргументами
- `pane_id` записывается в runtime state
- corner cases по session/tab покрыты тестами или smoke сценариями
- end-to-end цепочка до реального агента работает

## Журнал изменений

### 2026-03-14

- создан начальный план реализации для Feature 0003
