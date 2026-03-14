# ai-teamlead workflow

Этот проект это набор локальных CLI-first скриптов для персонального workflow
вокруг AI-driven анализа GitHub issue.

## Статус README

Этот `README.md` является repo-level входной точкой и верхнеуровневым обзором
проекта.

Он является каноническим источником для:

- назначения продукта
- границ MVP
- общей карты документации
- краткой архитектурной рамки уровня всего репозитория

Он не является каноническим источником для:

- flow-контрактов и статусных переходов
- детальной схемы конфигурации
- subsystem-level runtime-контрактов
- порядка реализации и verification-деталей

Для этих слоев `README.md` дает только summary и ссылки на профильные
канонические документы.

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

- `README.md` как repo-level overview и входная точка
- [docs/code-quality.md](./docs/code-quality.md)
- [docs/documentation-structure.md](./docs/documentation-structure.md)
- [docs/documentation-process.md](./docs/documentation-process.md)
- [docs/implementation-plan.md](./docs/implementation-plan.md)
- [docs/templates/feature-spec-template.md](./docs/templates/feature-spec-template.md)
- [docs/issue-analysis-flow.md](./docs/issue-analysis-flow.md)
- [docs/issue-implementation-flow.md](./docs/issue-implementation-flow.md)
- [docs/features/0001-ai-teamlead-cli/README.md](./docs/features/0001-ai-teamlead-cli/README.md)
- [docs/features/0002-repo-init/README.md](./docs/features/0002-repo-init/README.md)
- [docs/features/0003-agent-launch-orchestration/README.md](./docs/features/0003-agent-launch-orchestration/README.md)
- [docs/features/0004-issue-implementation-flow/README.md](./docs/features/0004-issue-implementation-flow/README.md)
- [docs/features/0005-agent-flow-integration-testing/README.md](./docs/features/0005-agent-flow-integration-testing/README.md)
- [AURA.md](./AURA.md) как project-local доступ к
  личному высокоуровневому инженерному видению разработчика

## MVP

Первая завершенная MVP-граница проекта покрывала только analysis stage и
подготовку плана реализации.

Следующий stage проекта добавляет отдельный implementation flow, но он остается
отдельным каноническим contract layer и не смешивается с analysis SSOT.

## Модель запуска

`ai-teamlead` реализуется как foreground CLI-утилита с командами:

- `init` — подключение репозитория, создание project-local contract layer
- `poll` — один цикл просмотра project snapshot: выбирает подходящую issue из
  `Backlog` и передает ее в общий issue-level `run`-path
- `run` — канонический issue-level entrypoint: определяет текущий stage issue,
  выбирает analysis или implementation flow, переводит статус, работает с
  `session_uuid` и запускает или восстанавливает launcher path
- `loop` — бесконечный foreground loop поверх `poll` с паузой из
  `runtime.poll_interval_seconds`

Разделение ответственности:

- `poll` отвечает только за selection cycle
- `run` отвечает за issue-level lifecycle
- `loop` отвечает только за повторение `poll`

## Источник истины

Источник истины по состоянию задачи это поле статуса в default GitHub Project.

Канонические flow-контракты, модели статусов и правила переходов зафиксированы
в:

- [docs/issue-analysis-flow.md](./docs/issue-analysis-flow.md)
- [docs/issue-implementation-flow.md](./docs/issue-implementation-flow.md)

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

Flow анализа и CLI-контракт вынесены в отдельный SSOT:

- [docs/issue-analysis-flow.md](./docs/issue-analysis-flow.md)

Коротко:

- если для реализации не хватает данных, агент задает вопросы в своей сессии
- если данных хватает, flow формирует план реализации
- для `feature` дополнительно обязательны `User Story` и `Use Cases`
- у flow есть human gate на ответах на вопросы и на принятии плана

## Flow реализации

Flow реализации и handoff после принятия плана вынесены в отдельный SSOT:

- [docs/issue-implementation-flow.md](./docs/issue-implementation-flow.md)

Коротко:

- оператор по-прежнему использует `run <issue>`;
- `run` сам определяет, что issue уже находится в implementation lifecycle;
- реализация опирается на approved analysis artifacts;
- implementation stage ведет issue через `Implementation In Progress`,
  `Waiting for CI`, `Waiting for Code Review` и `Implementation Blocked`.

## Статусы проекта

Ниже приведен только summary для быстрого ориентирования.

Канонический список статусов, их значения и допустимые переходы описаны в
[docs/issue-analysis-flow.md](./docs/issue-analysis-flow.md).

Для flow анализа используются следующие статусы в GitHub Project:

- `Backlog`
- `Analysis In Progress`
- `Waiting for Clarification`
- `Waiting for Plan Review`
- `Ready for Implementation`
- `Analysis Blocked`

Для flow реализации используются следующие дополнительные статусы:

- `Implementation In Progress`
- `Waiting for CI`
- `Waiting for Code Review`
- `Implementation Blocked`

## Конфигурация

Основной конфиг проекта должен называться `settings.yml`.

Конфиг хранится в YAML и должен лежать в каталоге `./.ai-teamlead/` целевого
репозитория.

Ниже приведен только repo-level overview.

Канонический контракт по repo-local asset layer, `settings.yml` и launcher path
раскрывается в связанных feature-документах и ADR:

- [docs/features/0001-ai-teamlead-cli/README.md](./docs/features/0001-ai-teamlead-cli/README.md)
- [docs/features/0002-repo-init/README.md](./docs/features/0002-repo-init/README.md)
- [docs/features/0003-agent-launch-orchestration/README.md](./docs/features/0003-agent-launch-orchestration/README.md)

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

Минимальный active override для MVP:

```yaml
github:
  project_id: "PVT_xxx"
```

Остальные MVP-поля приложение может брать из canonical default-layer. Поэтому
`templates/init/settings.yml` может оставаться comment-only шаблоном, где
defaulted-поля отражены в закомментированном виде как documented defaults, а не
как обязательный активный YAML.

Закомментированный bootstrap overview для defaulted-полей выглядит так:

```yaml
# issue_analysis_flow:
#   statuses:
#     backlog: "Backlog"
#     analysis_in_progress: "Analysis In Progress"
#     waiting_for_clarification: "Waiting for Clarification"
#     waiting_for_plan_review: "Waiting for Plan Review"
#     ready_for_implementation: "Ready for Implementation"
#     analysis_blocked: "Analysis Blocked"
#
# issue_implementation_flow:
#   statuses:
#     ready_for_implementation: "Ready for Implementation"
#     implementation_in_progress: "Implementation In Progress"
#     waiting_for_ci: "Waiting for CI"
#     waiting_for_code_review: "Waiting for Code Review"
#     implementation_blocked: "Implementation Blocked"
#
# runtime:
#   max_parallel: 1
#   poll_interval_seconds: 3600
#
# zellij:
#   session_name: "${REPO}"
#   tab_name: "issue-analysis"
#   layout: "compact"
#
# launch_agent:
#   analysis_branch_template: "analysis/issue-${ISSUE_NUMBER}"
#   worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
#   analysis_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
#   implementation_branch_template: "implementation/issue-${ISSUE_NUMBER}"
#   implementation_worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
#   implementation_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
```

`zellij.session_name` задает versioned fallback для target session, а
`zellij.tab_name` задает versioned target tab.

Documented default для `zellij.session_name` хранится в `settings.yml` как
закомментированный template `${REPO}`. Во время реального запуска `ai-teamlead`
подставляет сюда canonical GitHub repo slug из `origin`, если активный YAML не
переопределяет это поле.

Для `zellij.session_name` в MVP поддерживается только placeholder `${REPO}`.
Если после рендера в значении остались `${...}`, запуск завершается ошибкой
конфигурации. Literal-значения без placeholder остаются валидными.

Во время реального запуска `ai-teamlead`:

- выбирает target session в порядке:
  `--zellij-session` -> `ZELLIJ_SESSION_NAME` -> `zellij.session_name`
- использует выбранную session и `zellij.tab_name`, чтобы найти или создать
  нужные session/tab
- для existing session валидирует, что в ней нет panes из другого GitHub repo
- сохраняет runtime `session_id`, `tab_id`, `pane_id` уже в `.git/.ai-teamlead/`

`github.project_id` остается required-without-default полем.

Defaulted-by-application поля MVP:

- `issue_analysis_flow.statuses.*`
- `issue_implementation_flow.statuses.*`
- `runtime.max_parallel`
- `runtime.poll_interval_seconds`
- `zellij.session_name`
- `zellij.tab_name`
- `zellij.layout`
- `launch_agent.analysis_branch_template`
- `launch_agent.worktree_root_template`
- `launch_agent.analysis_artifacts_dir_template`
- `launch_agent.implementation_branch_template`
- `launch_agent.implementation_worktree_root_template`
- `launch_agent.implementation_artifacts_dir_template`

Дополнительно имеет смысл зарезервировать место для:

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

Ниже показана ожидаемая shape project-local contract layer.

Канонический контракт по составу и роли этих файлов раскрыт в
[docs/features/0002-repo-init/README.md](./docs/features/0002-repo-init/README.md)
и
[docs/features/0003-agent-launch-orchestration/README.md](./docs/features/0003-agent-launch-orchestration/README.md).

Bootstrap default для [./.ai-teamlead/launch-agent.sh](./.ai-teamlead/launch-agent.sh):

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

- [./.claude/README.md](./.claude/README.md) для
  Claude-specific материалов
- [./.codex/README.md](./.codex/README.md) как project
  convention для Codex-specific материалов

Инициализация этого слоя оформляется отдельной ручной командой:

- `ai-teamlead init`

Требование к `init`:

- команда запускается внутри git-репозитория
- в репозитории уже должен быть настроен `origin`, указывающий на GitHub
  repository
- `init` использует этот `origin` как repo context текущего проекта

После `init` оператор должен вручную завершить bootstrap:

1. раскомментировать и заменить `github.project_id` на реальный GitHub Project
   id
2. при необходимости раскомментировать и скорректировать `zellij.session_name`,
   `zellij.layout` или `launch_agent.*` templates под layout проекта
4. только после этого запускать `poll`, `run` или `loop`

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
- `poll` и `run` используют один и тот же issue-level launcher contract
- `loop` переиспользует тот же `poll` cycle и не вводит отдельный issue-level
  path
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
- проверка, что `loop` переживает пустой cycle и ошибку одного cycle
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
