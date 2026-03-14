# Feature 0001: Как проверяем

## Критерии корректности

Тестовая стратегия для этой feature должна соответствовать общему quality bar:

- [docs/code-quality.md](/home/danil/code/teamlead/docs/code-quality.md)

Решение считается корректным, если:

- `ai-teamlead` читает `./.ai-teamlead/settings.yml` из репозитория
- `ai-teamlead` корректно определяет repo context
- `ai-teamlead` выбирает только issues со статусом `Backlog`
- при наличии нескольких backlog issues `ai-teamlead` выбирает верхнюю issue в порядке
  GitHub Project
- перед запуском flow issue переводится в `Analysis In Progress`
- при ошибке смены статуса flow не стартует
- `ai-teamlead` не использует локальную базу как источник истины по состоянию issue
- `ai-teamlead` создает durable-связку между issue и `session_uuid`
- launcher записывает `zellij.session_id`, `zellij.tab_id`, `zellij.pane_id`
- `run` запускает нового агента в новой `zellij` pane
- в агентский запуск передается URL GitHub issue

## Критерии готовности

Feature считается готовой к первому использованию, если:

- `ai-teamlead` можно запустить в foreground в одном репозитории
- команды `poll` и `run` работают по документированным правилам
- `ai-teamlead` способен запустить `issue-analysis-flow` в `zellij`
- один репозиторий можно обслуживать без ручного редактирования кода
- второй репозиторий можно подключить заменой только repo-local конфига
- обязательные unit, integration и smoke tests для MVP пройдены

## Инварианты

- один экземпляр `ai-teamlead` обслуживает один репозиторий
- source of truth по статусу issue находится в GitHub Project
- `./.ai-teamlead/settings.yml` живет в репозитории
- при `max_parallel: 1` одновременно не должно запускаться больше одной issue
- каждая issue в анализе имеет ровно одну связанную агентскую сессию

## Сценарии проверки

### Сценарий 1. Базовый polling

- в Project есть одна `open` issue со статусом `Backlog`
- `ai-teamlead` выполняет polling
- issue переводится в `Analysis In Progress`
- flow запускается в новой pane внутри настроенной `zellij` session/tab

### Сценарий 2. Нет подходящих issues

- в Project нет issues со статусом `Backlog`
- `ai-teamlead` выполняет polling
- никаких запусков flow не происходит
- `ai-teamlead` остается в рабочем цикле без ошибок

### Сценарий 3. Ошибка смены статуса

- `ai-teamlead` находит подходящую issue
- изменение статуса в GitHub завершается ошибкой
- flow не запускается
- ошибка фиксируется в диагностике

### Сценарий 4. Повторный запуск через `run`

- issue находится в `Waiting for Clarification` или `Analysis Blocked`
- пользователь явно запускает `run`
- при выполнении правил входа issue возвращается в
  `Analysis In Progress`
- в новой `zellij` pane стартует новый агентский запуск

### Сценарий 5. Два независимых репозитория

- существуют два разных репозитория с собственными
  `./.ai-teamlead/settings.yml`
- в каждом запущен свой `ai-teamlead`
- оба процесса работают независимо и не мешают друг другу

### Сценарий 6. Ручной `poll`

- пользователь запускает `poll` в корне репозитория
- команда читает `./.ai-teamlead/settings.yml`
- команда выполняет ровно один polling cycle
- при наличии подходящей issue она claim-ится и запускается

### Сценарий 6a. Несколько backlog issues

- в `Backlog` есть несколько подходящих issues
- `ai-teamlead` или `poll` выбирает верхнюю issue в порядке GitHub Project

### Сценарий 7. Некорректный `run`

- пользователь запускает `run` для issue в недопустимом статусе
- команда не запускает flow
- причина отказа отражается в диагностике

## Диагностика и наблюдаемость

Минимально необходимо видеть:

- что `ai-teamlead` стартовал
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
- диагностические артефакты в `.git/.ai-teamlead/`
- session-binding и launcher-артефакты, позволяющие найти связанную агентскую
  сессию

Для launcher дополнительно достаточно видеть:

- `launch-layout.kdl`
- `pane-entrypoint.sh`
- `./.ai-teamlead/launch-agent.sh`

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
- [docs/adr/0011-use-zellij-main-release-in-ci.md](/home/danil/code/teamlead/docs/adr/0011-use-zellij-main-release-in-ci.md)
