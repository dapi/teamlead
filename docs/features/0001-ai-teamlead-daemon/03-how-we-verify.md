# Feature 0001: Как проверяем

## Критерии корректности

Тестовая стратегия для этой feature должна соответствовать общему quality bar:

- [docs/code-quality.md](/home/danil/code/teamlead/docs/code-quality.md)

Решение считается корректным, если:

- daemon читает `ai-teamlead.yml` из корня репозитория
- daemon корректно определяет repo context
- daemon выбирает только issues со статусом `Backlog`
- при наличии нескольких backlog issues daemon выбирает минимальный issue number
- перед запуском flow issue переводится в `Analysis In Progress`
- при ошибке смены статуса flow не стартует
- daemon не использует локальную базу как источник истины по состоянию issue
- daemon создает durable-связку между issue и `session_uuid`

## Критерии готовности

Feature считается готовой к первому использованию, если:

- daemon можно запустить в foreground в одном репозитории
- команды `poll` и `run` работают по документированным правилам
- daemon способен запустить `issue-analysis-flow` в `zellij`
- один репозиторий можно обслуживать без ручного редактирования кода
- второй репозиторий можно подключить заменой только repo-local конфига
- обязательные unit, integration и smoke tests для MVP пройдены

## Инварианты

- один экземпляр daemon обслуживает один репозиторий
- source of truth по статусу issue находится в GitHub Project
- `ai-teamlead.yml` живет в корне репозитория
- при `max_parallel: 1` одновременно не должно запускаться больше одной issue
- каждая issue в анализе имеет ровно одну связанную агентскую сессию

## Сценарии проверки

### Сценарий 1. Базовый polling

- в Project есть одна `open` issue со статусом `Backlog`
- daemon выполняет polling
- issue переводится в `Analysis In Progress`
- flow запускается в настроенной `zellij` tab

### Сценарий 2. Нет подходящих issues

- в Project нет issues со статусом `Backlog`
- daemon выполняет polling
- никаких запусков flow не происходит
- daemon остается в рабочем цикле без ошибок

### Сценарий 3. Ошибка смены статуса

- daemon находит подходящую issue
- изменение статуса в GitHub завершается ошибкой
- flow не запускается
- ошибка фиксируется в диагностике

### Сценарий 4. Повторный запуск через `run`

- issue находится в `Waiting for Clarification` или `Analysis Blocked`
- пользователь явно запускает `run`
- при выполнении правил входа issue возвращается в
  `Analysis In Progress`
- flow стартует повторно

### Сценарий 4a. `run` после новых ответов оператора

- issue находится в `Waiting for Clarification`
- в durable session-артефактах есть новый нормализованный ответ оператора
- `run` допускает повторный запуск анализа

### Сценарий 4b. `run` без новых данных

- issue находится в `Waiting for Clarification`
- в durable session-артефактах нет новых операторских данных после последнего
  набора вопросов
- `run` не запускает flow
- причина отказа отражается в диагностике

### Сценарий 5. Два независимых репозитория

- существуют два разных репозитория с собственными `ai-teamlead.yml`
- в каждом запущен свой daemon
- оба процесса работают независимо и не мешают друг другу

### Сценарий 6. Ручной `poll`

- пользователь запускает `poll` в корне репозитория
- команда читает `ai-teamlead.yml`
- команда выполняет ровно один polling cycle
- при наличии подходящей issue она claim-ится и запускается

### Сценарий 6a. Несколько backlog issues

- в `Backlog` есть несколько подходящих issues
- daemon или `poll` выбирает issue с минимальным issue number

### Сценарий 7. Некорректный `run`

- пользователь запускает `run` для issue в недопустимом статусе
- команда не запускает flow
- причина отказа отражается в диагностике

## Диагностика и наблюдаемость

Минимально необходимо видеть:

- что daemon стартовал
- какой репозиторий и какой `project_id` он обслуживает
- когда начинается polling cycle
- какая issue выбрана
- удалось ли изменить статус
- был ли запущен flow
- почему запуск не состоялся, если он был пропущен

В MVP отдельный health/status интерфейс не требуется.

Достаточными средствами наблюдаемости считаются:

- stdout/stderr foreground-процесса
- сообщения ручных команд `poll` и `run`
- диагностические артефакты в `.git/ai-teamlead/`
- session-артефакты, позволяющие восстановить вопросы, план и действия оператора

## Связанные документы

- [README.md](/home/danil/code/teamlead/README.md)
- [docs/issue-analysis-flow.md](/home/danil/code/teamlead/docs/issue-analysis-flow.md)
- [docs/adr/0001-repo-local-ai-config.md](/home/danil/code/teamlead/docs/adr/0001-repo-local-ai-config.md)
- [docs/adr/0002-standalone-foreground-daemon.md](/home/danil/code/teamlead/docs/adr/0002-standalone-foreground-daemon.md)
- [docs/adr/0003-github-project-status-as-source-of-truth.md](/home/danil/code/teamlead/docs/adr/0003-github-project-status-as-source-of-truth.md)
- [docs/adr/0004-runtime-artifacts-in-git-dir.md](/home/danil/code/teamlead/docs/adr/0004-runtime-artifacts-in-git-dir.md)
- [docs/adr/0005-cli-contract-for-poll-and-run.md](/home/danil/code/teamlead/docs/adr/0005-cli-contract-for-poll-and-run.md)
- [docs/adr/0006-use-gh-cli-as-github-integration-layer.md](/home/danil/code/teamlead/docs/adr/0006-use-gh-cli-as-github-integration-layer.md)
- [docs/adr/0007-no-separate-health-interface-in-mvp.md](/home/danil/code/teamlead/docs/adr/0007-no-separate-health-interface-in-mvp.md)
- [docs/adr/0008-bind-issue-to-agent-session-uuid.md](/home/danil/code/teamlead/docs/adr/0008-bind-issue-to-agent-session-uuid.md)
- [docs/adr/0009-deterministic-backlog-ordering.md](/home/danil/code/teamlead/docs/adr/0009-deterministic-backlog-ordering.md)
