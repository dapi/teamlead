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
- `ai-teamlead` корректно выбирает target `zellij` session по правилу
  `args -> env -> settings`
- `ai-teamlead` корректно находит или создает `zellij` session по effective
  target session
- `run` корректно выбирает launch target по правилу
  `CLI -> settings -> runtime default`
- `poll` и `loop` не экспонируют public override по launch target
- при `session missing` launcher корректно различает path `custom layout` и
  `default fallback`
- `ai-teamlead` корректно разделяет `pane` и `tab` launcher paths
- launcher использует issue-aware effective tab name в `tab`-режиме и
  сохраняет его в runtime metadata даже без active override в YAML
- analysis tab использует versioned tab-layout contract и не выглядит как bare
  technical tab, если project-local contract ожидает bar/plugins и другой UX
- после запуска pane в runtime state записывается `pane_id`
- launcher передает canonical default args в `codex` и `claude`, если repo не
  задал override
- launcher не смешивает `codex` args и `claude` args между ветками запуска

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
- `zellij.session_name` является versioned fallback, а не единственным
  источником target session
- `zellij.tab_name` является stable semantic name
- `zellij.launch_target` влияет только на способ открытия launch context внутри
  уже выбранной session
- `zellij.tab_name_template` является defaulted issue-aware template для
  tab-launch path и не подменяет stable role `zellij.tab_name`
- generated `launch-layout.kdl` отвечает за analysis tab, а не за базовую
  session при `layout = None`
- generated `launch-layout.kdl` не должен принудительно задавать
  `close_on_exit false`
- внешний вид analysis tab задается явным versioned contract, а не попыткой
  сериализовать live-state текущей session обратно в layout
- `pane_id` является runtime-only значением
- runtime не генерирует отдельный launcher-script для pane
- shared multi-repo existing session запрещена
- agent global args передаются через shell-safe array contract, а не через raw
  shell string

## Сценарии проверки

### Сценарий 1. Нет session

- запускается `run`
- `ai-teamlead` рендерит `${REPO}` в `zellij.session_name` и создает новую
  session с этим именем
- создается tab `issue-analysis`
- открывается новая pane

### Сценарий 2. Session уже существует

- запускается `run` в `pane`-режиме
- используется существующая session
- в shared tab открывается новая pane

### Сценарий 2a. Existing session в `tab`-режиме

- запускается `run`
- используется существующая session
- создается новый analysis tab с issue-aware именем по effective policy

### Сценарий 2a. Новая session с `zellij.layout`

- `run` запускается при отсутствии session
- в `settings.yml` задан `zellij.layout`
- launcher создает новую session через пользовательский layout
- analysis tab добавляется отдельным generated layout из versioned contract и
  выглядит как родной tab этой session

### Сценарий 2b. Новая session без `zellij.layout`

- `run` запускается при отсутствии session
- `zellij.layout` отсутствует
- launcher не использует bare generated layout как базовую session
- analysis tab добавляется отдельным generated layout из versioned contract

### Сценарий 3. Команда запущена внутри `zellij`

- `run` или `poll` запускается с `ZELLIJ_SESSION_NAME`
- CLI override отсутствует
- используется текущая session из окружения, а не fallback из `settings.yml`

### Сценарий 4. CLI override задан явно

- `run` или `poll` запускается с `--zellij-session`
- в окружении также может присутствовать `ZELLIJ_SESSION_NAME`
- используется session из CLI override

### Сценарий 5. Session пропала

- session с ожидаемым именем отсутствует
- `run` или `poll` запускает recreate session
- flow продолжается без ручного вмешательства

### Сценарий 6. Session resurrect-нулась

- session существует под ожидаемым именем
- `run` или `poll` использует ее как existing session
- новая pane создается успешно

### Сценарий 7. Existing session содержит другой repo

- launcher обнаруживает panes другого GitHub repo в выбранной session
- запуск завершается ошибкой
- issue не должна silently уходить в shared multi-repo session

### Сценарий 8. Несколько tab с одинаковым именем

- launcher обнаруживает неоднозначный tab context
- запуск завершается ошибкой
- issue не должна silently уходить в непредсказуемый pane или duplicate tab

### Сценарий 9. Launcher-script подготавливает analysis worktree

- `run` или `poll` открывает новую pane
- pane запускает `./.ai-teamlead/launch-agent.sh`
- launcher-script сначала привязывает `pane_id`, потом готовит analysis
  worktree
- создает каталог versioned analysis-артефактов
- только после этого может стартовать реальный агент

### Сценарий 10. Измененные templates в `settings.yml`

- владелец репозитория меняет branch/worktree/artifacts templates
- `launch-agent.sh` использует новые значения без изменения core-кода

### Сценарий 11. `codex` недоступен

- launcher подготовил analysis worktree
- `codex` отсутствует в окружении
- если `claude` доступен, launcher запускает `claude` с canonical или
  пользовательскими `claude` args
- если `claude` тоже отсутствует, launcher не теряет подготовленный контекст и
  оставляет пользователя в shell внутри analysis worktree

### Сценарий 12. Canonical `codex` defaults

- `run` или `poll` запускает analysis stage
- repo не задает override для `launch_agent.global_args.codex`
- `codex` получает `--ask-for-approval never --sandbox workspace-write`

### Сценарий 13. Canonical `claude` defaults

- `codex` отсутствует в окружении
- repo не задает override для `launch_agent.global_args.claude`
- `claude` получает `--permission-mode auto`

### Сценарий 14. Пользовательский override для `codex`

- repo задает `launch_agent.global_args.codex`
- launcher использует пользовательский список args
- canonical default `--ask-for-approval never --sandbox workspace-write` не
  дублируется поверх override

## Диагностика и наблюдаемость

Минимально необходимо видеть:

- какой effective `session_name` ожидался
- какой effective `launch_target` ожидался
- какой effective `tab_name` ожидался
- существовала ли session до запуска
- был ли создан новый tab
- какой branch создания session был выбран:
  `existing session`, `custom layout`, `default fallback`
- какой versioned tab-layout contract использовался для analysis tab
- отдельно ли завершился шаг `create session` до шага `add analysis tab`
- какой `pane_id` был привязан к `session_uuid`

## Журнал изменений

### 2026-03-13

- создана feature-спека orchestration flow запуска агента

### 2026-03-14

- добавлено требование заказчика: analysis tab должна выглядеть как родной tab
  session
