# ai-teamlead workflow

Этот проект это набор локальных CLI-first скриптов для персонального workflow
вокруг AI-driven анализа GitHub issue.

## Ключевое требование

Проект должен быть спроектирован так, чтобы его можно было легко подключить к
любому GitHub-репозиторию без переписывания основной логики.

Текущее репо одновременно является:

- репозиторием самого инструмента
- dogfooding-средой, в которой инструмент разрабатывается и используется на
  самом себе

Из этого следуют требования:

- логика flow не должна быть жестко привязана к одному конкретному репозиторию
- repo-specific настройки должны жить в конфиге, а не в коде
- инструмент должен уметь работать как минимум в двух режимах:
  на собственном репозитории и на внешнем подключенном репозитории
- конфиг должен жить внутри самого целевого репозитория
- каждый репозиторий должен иметь возможность запускать свой собственный
  экземпляр `ai-teamlead` независимо от других репозиториев

## Структура документации

Документация проекта строится по трем осям:

1. Что строим
2. Как строим
3. Как проверяем

Это правило действует и на уровень проекта, и на уровень отдельных фич.

Базовые документы:

- [docs/code-quality.md](/home/danil/code/teamlead/docs/code-quality.md)
- [docs/documentation-structure.md](/home/danil/code/teamlead/docs/documentation-structure.md)
- [docs/templates/feature-spec-template.md](/home/danil/code/teamlead/docs/templates/feature-spec-template.md)
- [docs/issue-analysis-flow.md](/home/danil/code/teamlead/docs/issue-analysis-flow.md)

## MVP

MVP делает только анализ задач и подготовку плана реализации.

MVP не делает:

- автоматическую реализацию
- создание implementation-коммитов или PR

## Модель запуска

`ai-teamlead` реализуется как CLI-утилита с тремя основными командами:

- `init` — подключение репозитория, создание project-local contract layer
- `poll` — one-shot polling: находит подходящую issue в GitHub Project,
  забирает её в работу и запускает agent-launch flow в заданной `zellij`
  session и tab
- `run` — запуск flow по явно указанной issue

## Источник истины

Источник истины по состоянию задачи это поле статуса в default GitHub Project.

Постоянное локальное runtime state не используется как источник истины.
Локально могут существовать только временные рабочие файлы.

## Отбор задач

Issue подходит для запуска flow анализа, если:

- issue state = `open`
- issue находится в нужном GitHub Project
- project status = `Backlog`

Тип задачи определяется:

1. по GitHub labels
2. по тексту issue, если labels не хватает

Поддерживаемые типы:

- `bug`
- `feature`
- `chore`

## Flow анализа

Flow анализа вынесен в отдельный SSOT:

- [docs/issue-analysis-flow.md](/home/danil/code/teamlead/docs/issue-analysis-flow.md)

Коротко:

- если для реализации не хватает данных, агент задает вопросы в своей сессии
- если данных хватает, flow формирует план реализации
- для `feature` дополнительно обязательны `User Story` и `Use Cases`
- у flow есть human gate на ответах на вопросы и на принятии плана

## Статусы проекта

Для flow анализа используются следующие статусы в GitHub Project:

- `Backlog`
- `Analysis In Progress`
- `Waiting for Clarification`
- `Waiting for Plan Review`
- `Ready for Implementation`
- `Analysis Blocked`

## Конфигурация

Основной конфиг проекта должен называться `settings.yml`.

Конфиг хранится в YAML и должен лежать в каталоге `./.ai-teamlead/` целевого
репозитория.

Это означает:

- по умолчанию `ai-teamlead` читает `./.ai-teamlead/settings.yml` из текущего
  репозитория
- разные репозитории могут иметь разные `./.ai-teamlead/settings.yml`
- у каждого репозитория может быть свой отдельный запущенный экземпляр
  `ai-teamlead`
- repo context по умолчанию берется из самого репозитория, в котором запущен
  инструмент, а не из глобального конфига пользователя
- GitHub owner/repo для MVP всегда берутся из текущего git-репозитория и не
  переопределяются через конфиг

Минимальная структура для MVP:

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

launch_agent:
  analysis_branch_template: "analysis/issue-${ISSUE_NUMBER}"
  worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  analysis_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
```

`zellij.session_name` и `zellij.tab_name` задают стабильный project-local
launcher context.

Bootstrap default для `zellij.session_name` хранится в `settings.yml` как
template `${REPO}`. Во время реального запуска `ai-teamlead` подставляет сюда
canonical GitHub repo slug из `origin`.

Для `zellij.session_name` в MVP поддерживается только placeholder `${REPO}`.
Если после рендера в значении остались `${...}`, запуск завершается ошибкой
конфигурации. Literal-значения без placeholder остаются валидными.

Во время реального запуска `ai-teamlead`:

- использует эти имена, чтобы найти или создать нужные session/tab
- сохраняет runtime `session_id`, `tab_id`, `pane_id` уже в `.git/.ai-teamlead/`

Обязательные поля MVP:

- `github.project_id`
- `issue_analysis_flow.statuses.*`
- `runtime.max_parallel`
- `runtime.poll_interval_seconds`
- `zellij.session_name`
- `zellij.tab_name`
- `launch_agent.analysis_branch_template`
- `launch_agent.worktree_root_template`
- `launch_agent.analysis_artifacts_dir_template`

Что еще нужно кроме перечисленного тобой:

- `issue_analysis_flow.statuses.*`
  Лучше хранить не просто список статусов, а именованные переходные статусы по
  ролям. Тогда код будет работать с ключами `backlog`, `analysis_blocked` и так
  далее, а человек сможет сопоставить их с реальными названиями статусов в
  конкретном GitHub Project.

Пока не включаем в обязательный MVP, но стоит зарезервировать место для:

- `github.base_url`
  На случай GitHub Enterprise.
- `runtime.poll_lock_file`
  Если захотим явно настраивать lock path для poller.
- `zellij.layout`
  Если позже появятся разные режимы запуска.
- `prompts.issue_analysis_flow`
  Если вынесем prompt-файлы в конфигурируемые пути.

Для MVP durable-связка между issue и агентской сессией хранится в repo-local
runtime-артефактах внутри `.git/.ai-teamlead/`.

## Project contract layer

Versioned project-local contract живет в рабочем дереве репозитория:

```text
.ai-teamlead/
  README.md
  settings.yml
  init.sh
  launch-agent.sh
  flows/
    issue-analysis-flow.md
    issue-analysis/
      README.md
      01-what-we-build.md
      02-how-we-build.md
      03-how-we-verify.md
```

Ключевые launcher templates в `settings.yml`:

```yaml
launch_agent:
  analysis_branch_template: "analysis/issue-${ISSUE_NUMBER}"
  worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  analysis_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
```

Bootstrap default для [./.ai-teamlead/launch-agent.sh](/home/danil/code/teamlead/.ai-teamlead/launch-agent.sh):

- подготавливает analysis branch/worktree по templates из `settings.yml`
- создает каталог versioned analysis-артефактов
- запускает `codex`, если он доступен
- иначе оставляет shell внутри подготовленного worktree

Текущий контракт `launch_agent.*` в реализации:

- поддерживаются placeholder-переменные `${HOME}`, `${REPO}`,
  `${ISSUE_NUMBER}`
- для `worktree_root_template` и `analysis_artifacts_dir_template`
  дополнительно доступна `${BRANCH}`
- интерполяция выполняется простым string replace внутри `ai-teamlead`
- неизвестные placeholder-переменные в MVP не валидируются и остаются как есть
  в результирующей строке
- `worktree_root_template` должен давать абсолютный путь к worktree root
- `analysis_artifacts_dir_template` интерпретируется как путь относительно
  analysis worktree

Project-local agent assets:

- [./.claude/README.md](/home/danil/code/teamlead/.claude/README.md) для
  Claude-specific материалов
- [./.codex/README.md](/home/danil/code/teamlead/.codex/README.md) как project
  convention для Codex-specific материалов

Инициализация этого слоя оформляется отдельной ручной командой:

- `ai-teamlead init`

Требование к `init`:

- команда запускается внутри git-репозитория
- в репозитории уже должен быть настроен `origin`, указывающий на GitHub
  repository
- `init` использует этот `origin` как repo context текущего проекта

После `init` оператор должен вручную завершить bootstrap:

1. заменить placeholder в `github.project_id` на реальный GitHub Project id
2. при необходимости скорректировать `zellij.session_name`
3. при необходимости скорректировать `launch_agent.*` templates под layout
   проекта
4. только после этого запускать `poll` или `run`

Если `github.project_id` оставлен placeholder-значением или указывает на
невалидный проект, текущая реализация падает на чтении project snapshot до
выбора issue.

Runtime state и временные артефакты при этом живут отдельно в:

```text
.git/.ai-teamlead/
```

Если в корне репозитория отсутствует `./init.sh`, bootstrap может дополнительно
создать симлинк:

```text
./init.sh -> ./.ai-teamlead/init.sh
```

Bootstrapped `./.ai-teamlead/init.sh` рассчитан на запуск уже внутри worktree:

- он копирует отсутствующие `.env*` из primary worktree
- не перезаписывает уже существующие `.env*`
- определяет default branch без перебора `git worktree list`

Ограничение MVP:

- целевое значение `runtime.max_parallel = 1`
- если используется одна фиксированная `zellij` tab, значения больше `1` пока
  не поддерживаются корректно

## Как проверяем

Проект считается собранным корректно, если одновременно выполняются условия:

- `poll` выбирает первую подходящую issue в порядке snapshot GitHub Project
- `run` и `poll` используют один и тот же launcher contract
- для каждой запущенной issue создается durable binding `issue <-> session_uuid`
- `launch-agent.sh` подготавливает worktree и versioned analysis artifacts до
  старта агента
- при отсутствии `codex` launcher честно деградирует в shell внутри уже
  подготовленного analysis worktree

Минимальный ручной smoke для MVP:

- `ai-teamlead init` в чистом git-репозитории с настроенным `origin`
- ручная замена `github.project_id` в `./.ai-teamlead/settings.yml`
- успешный `poll` для issue в `Backlog`
- успешный `run` для issue в одном из разрешенных waiting-статусов
- проверка runtime-артефактов в `.git/.ai-teamlead/`
- проверка независимого запуска на втором репозитории с другим
  `zellij.session_name`

## Тестирование `zellij`

Интеграционные тесты launcher выполняются в Docker.

Правила:

- pinned версия `zellij` хранится в `ZELLIJ_VERSION`
- CI скачивает release из `dapi/zellij-main`
- внутри контейнера бинарь сохраняется как обычный `zellij`
- launcher-тесты гоняются headless через `script`

Локальный запуск:

- `mise run test-zellij`
