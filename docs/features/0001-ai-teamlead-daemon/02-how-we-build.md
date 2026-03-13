# Feature 0001: Как строим

## Архитектура

Первая версия строится как один foreground daemon на один репозиторий.

Состав решения:

- загрузчик repo-local конфига `ai-teamlead.yml`
- polling loop
- GitHub adapter на базе `gh` CLI для чтения и изменения состояния issue в
  Project
- dispatcher запуска flow
- launcher интеграции с `zellij`

## Данные и состояния

Ключевые входные данные:

- repo context текущего git-репозитория
- конфиг `ai-teamlead.yml`
- GitHub Project id
- mapping статусных имен для `issue-analysis-flow`

Ключевые состояния:

- daemon запущен
- daemon ждет следующего polling cycle
- daemon нашел подходящую issue
- issue переведена в `Analysis In Progress`
- issue связана с `session_uuid`
- flow запущен
- daemon вернулся в цикл ожидания

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

- repo-local конфиг `ai-teamlead.yml`
- standalone daemon в foreground
- single-process loop
- язык реализации MVP: Rust
- GitHub Project status как источник истины
- `max_parallel: 1` для MVP
- repo-local runtime-артефакты в `.git/ai-teamlead/`
- минимальный CLI-контракт состоит из команд `poll` и `run`
- базовый GitHub integration layer строится на `gh` CLI
- GitHub owner/repo жестко берутся из текущего git-репозитория
- каждая issue в анализе имеет связанную агентскую сессию с `session_uuid`
- `poll` выбирает issue из `Backlog` по возрастанию issue number

Дополнительные правила реализации:

- daemon должен работать только в контексте одного репозитория
- несколько репозиториев могут иметь независимые daemon-инстансы
- смена статуса в GitHub должна происходить до старта анализа
- локальный state не должен подменять GitHub как источник истины

## Конфигурация

Минимальный контракт `ai-teamlead.yml`:

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
  session_name: "ai-teamlead"
  tab_name: "issue-analysis"
```

## Ограничения реализации

- пока не проектируем отдельный supervisor model
- пока не проектируем multi-worker execution
- пока не проектируем глобальный конфиг пользователя
- пока не проектируем собственный native GitHub API client
- пока не вводим отдельный health/status интерфейс

## Runtime layout

Repo-local runtime-артефакты daemon хранятся в:

```text
.git/ai-teamlead/
```

Внутри этой директории на первом этапе допускаются:

- lock-файлы
- временные prompt-файлы
- временные артефакты анализа
- диагностические следы активного запуска
- durable session-артефакты, включая связь `issue <-> session_uuid`
- `zellij.session_id`, `zellij.tab_id`, `zellij.pane_id`
- журнал нормализованных действий оператора

Точная схема файлов и полей вынесена в:

- [05-runtime-artifacts.md](/home/danil/code/teamlead/docs/features/0001-ai-teamlead-daemon/05-runtime-artifacts.md)

## CLI-контракт

Для MVP фиксируются две ручные команды:

### `poll`

Назначение:

- выполнить один цикл выбора следующей подходящей issue

Правила:

- команда работает в контексте текущего репозитория
- команда читает `ai-teamlead.yml` из корня репозитория
- команда выбирает только issue со статусом `Backlog`
- команда не принимает issue как аргумент
- команда не должна брать больше `runtime.max_parallel` issues за цикл
- при наличии нескольких подходящих issues команда выбирает минимальный issue
  number

### `run`

Назначение:

- запустить flow по явно указанной issue

Правила:

- команда принимает issue number или issue URL
- команда работает в контексте текущего репозитория
- команда использует те же правила допустимых статусов, что и SSOT
- команда не должна обходить правила transition model
- для waiting-статусов команда опирается на durable session-артефакты, а не на
  эвристику по текущему процессу
