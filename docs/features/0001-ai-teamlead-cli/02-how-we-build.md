# Feature 0001: Как строим

## Архитектура

Первая версия строится как CLI-утилита, работающая в контексте одного репозитория.

Состав решения:

- загрузчик repo-local конфига `./.ai-teamlead/settings.yml`
- selection cycle `poll`
- issue-level orchestration path `run`
- foreground loop `loop`
- GitHub adapter на базе `gh` CLI для чтения и изменения состояния issue в
  Project
- dispatcher запуска flow
- launcher интеграции с `zellij`

## Данные и состояния

Ключевые входные данные:

- repo context текущего git-репозитория
- конфиг `./.ai-teamlead/settings.yml`
- GitHub Project id
- mapping статусных имен для `issue-analysis-flow`

Ключевые состояния:

- `poll` запущен
- `poll` не нашел подходящей issue, завершился
- `poll` нашел подходящую issue
- `run` проверил допустимость входа
- issue переведена в `Analysis In Progress`
- issue связана с `session_uuid`
- flow запущен
- `poll` завершился
- `loop` продолжает следующий цикл после пустого результата или ошибки одного
  цикла

Состояния issue определяются не локально, а через GitHub Project statuses.

## Интерфейсы

Внешние интерфейсы:

- `gh` CLI для работы с issue и GitHub Project
- `zellij` для запуска agent session
- локальный git-репозиторий как источник repo context

Внутренние интерфейсы:

- config loader
- issue selector
- issue claimer
- flow runner
- logging/diagnostics

## Технические решения

Уже принятые решения:

- repo-local конфиг `./.ai-teamlead/settings.yml`
- versioned project-local flow в `./.ai-teamlead/flows/issue-analysis-flow.md`
- foreground CLI-утилита с командами `init`, `poll`, `run`, `loop`
- `loop` является foreground-оберткой над `poll`, а не отдельным daemon model
- язык реализации MVP: Rust
- GitHub Project status как источник истины
- `max_parallel: 1` для MVP
- repo-local runtime-артефакты в `.git/.ai-teamlead/`
- минимальный CLI-контракт состоит из команд `poll`, `run`, `loop`
- базовый GitHub integration layer строится на `gh` CLI
- GitHub owner/repo жестко берутся из текущего git-репозитория
- каждая issue в анализе имеет связанную агентскую сессию с `session_uuid`
- `poll` выбирает issue из `Backlog` в порядке snapshot GitHub Project
- docker-based CI для `zellij` использует pinned release из `dapi/zellij-main`

Дополнительные правила реализации:

- `ai-teamlead` должен работать только в контексте одного репозитория
- несколько репозиториев могут использоваться независимо
- смена статуса в GitHub должна происходить до старта анализа
- локальный state не должен подменять GitHub как источник истины

## Конфигурация

Минимальный контракт `./.ai-teamlead/settings.yml`:

```yaml
github:
  project_id: "PVT_xxx"

issue_analysis_flow:
  statuses:
    backlog: "Backlog"
    analysis_in_progress: "Analysis In Progress"
    waiting_for_clarification: "Waiting for Clarification"
    waiting_for_plan_review: "Waiting for Plan Review"
    ready_for_implementation: "Ready for Implementation"
    analysis_blocked: "Analysis Blocked"

runtime:
  max_parallel: 1
  poll_interval_seconds: 3600

zellij:
  session_name: "${REPO}"
  tab_name: "issue-analysis"
```

Здесь:

- `session_name` следует правилу из
  [ADR-0021](../../../docs/adr/0021-zellij-session-target-resolution.md):
  default хранится как `${REPO}` и рендерится из GitHub repo slug, но
  используется как fallback после CLI override и `ZELLIJ_SESSION_NAME`
- `tab_name` это стабильный project-local идентификатор для orchestration
- runtime `session_id`, `tab_id`, `pane_id` не задаются в конфиге
- `ai-teamlead` во время запуска должен выбрать effective target session,
  запретить shared multi-repo existing session и затем либо найти существующие
  session/tab, либо создать их

## Ограничения реализации

- пока не проектируем отдельный supervisor model
- пока не проектируем multi-worker execution
- пока не проектируем глобальный конфиг пользователя
- пока не проектируем собственный native GitHub API client
- пока не вводим отдельный health/status интерфейс

## Runtime layout

Repo-local runtime-артефакты хранятся в:

```text
.git/.ai-teamlead/
```

Внутри этой директории на первом этапе допускаются:

- lock-файлы
- временные prompt-файлы
- временные артефакты анализа
- диагностические следы активного запуска
- durable session-артефакты, включая связь `issue <-> session_uuid`
- `zellij.session_id`, `zellij.tab_id`, `zellij.pane_id`
- журнал нормализованных действий оператора

Для `zellij` launcher дополнительно создаются:

- `launch-layout.kdl`
- `pane-entrypoint.sh`

Точная схема файлов и полей вынесена в:

- [05-runtime-artifacts.md](./05-runtime-artifacts.md)

## CLI-контракт

Для MVP фиксируются ручные команды `poll`, `run`, `loop`.

### `poll`

Назначение:

- выполнить один цикл выбора следующей подходящей issue

Правила:

- команда работает в контексте текущего репозитория
- команда читает `./.ai-teamlead/settings.yml` из репозитория
- команда выбирает только issue со статусом `Backlog`
- команда не принимает issue как аргумент
- команда не должна брать больше `runtime.max_parallel` issues за цикл
- при наличии нескольких подходящих issues команда выбирает верхнюю issue в
  порядке GitHub Project
- команда не реализует отдельный issue-level lifecycle
- если issue выбрана, команда передает ее в общий issue-level `run`-path
- если issue не найдена, команда завершает цикл без ошибки

### `run`

Назначение:

- запустить flow по явно указанной issue

Правила:

- команда принимает issue number или issue URL
- команда работает в контексте текущего репозитория
- команда использует те же правила допустимых статусов, что и SSOT
- команда не должна обходить правила transition model
- команда является каноническим issue-level entrypoint
- команда отвечает за claim, re-entry, `session_uuid` и launcher orchestration
- команда запускает или восстанавливает launcher path в stable launch context
- в запуск агента передается project-local `issue-analysis-flow`
- в запуск агента передается URL GitHub issue
- `poll` после выбора issue использует тот же `run`-path

### `loop`

Назначение:

- выполнять непрерывный foreground loop поверх `poll`

Правила:

- команда не принимает issue как аргумент
- команда работает в контексте текущего репозитория
- команда выполняет bootstrap один раз до входа в цикл
- между циклами команда делает паузу по `runtime.poll_interval_seconds`
- пустой цикл `poll` не завершает `loop`
- ошибка одного цикла не завершает `loop`
- bootstrap/config/runtime ошибки до входа в loop остаются фатальными
- `loop` использует ровно те же selection semantics, что и `poll`
- `loop` использует ровно тот же issue-level `run`-path, что и `poll` и `run`

## Launcher

`zellij` launcher работает так:

- создает `launch-layout.kdl` и тонкий `pane-entrypoint.sh` в session-директории
- рендерит `launch-layout.kdl` из versioned project-local template
  `./.ai-teamlead/zellij/analysis-tab.kdl`
- если zellij session еще не существует, запускает новую session через
  `zellij --session <name>` или
  `zellij --session <name> --layout <layout-name>`, если задан `zellij.layout`
- после создания новой session launcher отдельно ждет появления session и
  только затем добавляет analysis tab
- если session уже существует, добавляет tab через
  `zellij action new-tab --layout <launch-layout.kdl>` в target session
- launcher создает новую pane для конкретного запуска issue-analysis
- `pane-entrypoint.sh` только задает repo context и передает путь к бинарю
  `ai-teamlead`
- через этот shim launcher запускает versioned
  `./.ai-teamlead/launch-agent.sh` в новой pane
- `launch-agent.sh` должен запускаться из корня репозитория как `cwd`
- сам `launch-agent.sh` вызывает внутреннюю команду `bind-zellij-pane`,
  готовит analysis worktree и после этого стартует настроенного агента
  (`codex` или `claude`) с project-local `issue-analysis-flow` и URL issue
- минимальный launcher input для агента:
  `./.ai-teamlead/flows/issue-analysis-flow.md`, URL GitHub issue, `session_uuid`
