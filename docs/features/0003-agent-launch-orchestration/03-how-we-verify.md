# Feature 0003: Как проверяем

## Критерии корректности

Решение считается корректным, если:

- `poll` и `run` используют один и тот же launch path
- агент стартует через `./.ai-teamlead/launch-agent.sh`
- `launch-agent.sh` запускается из корня репозитория
- первым аргументом в `launch-agent.sh` передается `session_uuid`
- вторым аргументом в `launch-agent.sh` передается `issue_url`
- branch/worktree подготавливаются до запуска реального агента
- branch/worktree/artifacts naming читаются из `settings.yml`
- `ai-teamlead` корректно находит или создает `zellij` session по
  `session_name`
- `ai-teamlead` корректно находит или создает tab по `tab_name`
- после запуска pane в runtime state записывается `pane_id`

## Критерии готовности

Feature считается готовой, если:

- operator может запустить issue через `run`
- `poll` может найти и запустить issue
- оба сценария приводят к одинаковому agent launcher behavior
- corner cases по session/tab покрыты тестами или зафиксированными smoke
  сценариями

## Инварианты

- `issue-analysis-flow` не является orchestration-документом
- `launch-agent.sh` является versioned project-local script
- `session_name` и `tab_name` являются stable semantic names
- `pane_id` является runtime-only значением
- runtime не генерирует отдельный launcher-script для pane

## Сценарии проверки

### Сценарий 1. Нет session

- запускается `run`
- `ai-teamlead` рендерит `${REPO}` в `zellij.session_name` и создает новую
  session с этим именем
- создается tab `issue-analysis`
- открывается новая pane

### Сценарий 2. Session уже существует

- запускается `run`
- используется существующая session
- в нужном tab открывается новая pane

### Сценарий 3. Session пропала

- session с ожидаемым именем отсутствует
- `run` или `poll` запускает recreate session
- flow продолжается без ручного вмешательства

### Сценарий 4. Session resurrect-нулась

- session существует под ожидаемым именем
- `run` или `poll` использует ее как existing session
- новая pane создается успешно

### Сценарий 5. Несколько tab с одинаковым именем

- launcher обнаруживает неоднозначный tab context
- запуск завершается ошибкой
- issue не должна silently уходить в непредсказуемый pane

### Сценарий 6. Launcher-script подготавливает analysis worktree

- `run` или `poll` открывает новую pane
- pane запускает `./.ai-teamlead/launch-agent.sh`
- launcher-script сначала привязывает `pane_id`, потом готовит analysis
  worktree
- создает каталог versioned analysis-артефактов
- только после этого может стартовать реальный агент

### Сценарий 7. Измененные templates в `settings.yml`

- владелец репозитория меняет branch/worktree/artifacts templates
- `launch-agent.sh` использует новые значения без изменения core-кода

### Сценарий 8. `codex` недоступен

- launcher подготовил analysis worktree
- `codex` отсутствует в окружении
- launcher не теряет подготовленный контекст
- пользователь получает shell внутри analysis worktree

## Диагностика и наблюдаемость

Минимально необходимо видеть:

- какой `session_name` ожидался
- какой `tab_name` ожидался
- существовала ли session до запуска
- был ли создан новый tab
- какой `pane_id` был привязан к `session_uuid`

## Журнал изменений

### 2026-03-13

- создана feature-спека orchestration flow запуска агента
