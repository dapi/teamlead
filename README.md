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

## Системные требования

- **Rust** (edition 2024) и **Cargo**
- **Git**
- **GitHub CLI** (`gh`) с активной авторизацией
- **zellij** для управления agent-сессиями
- **Docker** — только для интеграционных тестов (`test-zellij`)

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
- [ROADMAP.md](./ROADMAP.md) как repo-level карта backlog, кластеров и
  зависимостей
- [docs/governance.md](./docs/governance.md)
- [docs/code-quality.md](./docs/code-quality.md)
- [docs/config.md](./docs/config.md)
- [docs/documentation-structure.md](./docs/documentation-structure.md)
- [docs/documentation-process.md](./docs/documentation-process.md)
- [docs/implementation-plan.md](./docs/implementation-plan.md)
- [docs/templates/feature-spec-template.md](./docs/templates/feature-spec-template.md)
- [docs/templates/implementation-plan-template.md](./docs/templates/implementation-plan-template.md)
- [docs/issue-analysis-flow.md](./docs/issue-analysis-flow.md)
- [docs/issue-implementation-flow.md](./docs/issue-implementation-flow.md)
- [docs/untrusted-input-security.md](./docs/untrusted-input-security.md)
- [docs/features/0001-ai-teamlead-cli/README.md](./docs/features/0001-ai-teamlead-cli/README.md)
- [docs/features/0002-repo-init/README.md](./docs/features/0002-repo-init/README.md)
- [docs/features/0003-agent-launch-orchestration/README.md](./docs/features/0003-agent-launch-orchestration/README.md)
- [docs/features/0004-issue-implementation-flow/README.md](./docs/features/0004-issue-implementation-flow/README.md)
- [docs/features/0005-agent-flow-integration-testing/README.md](./docs/features/0005-agent-flow-integration-testing/README.md)
- [docs/features/0006-public-repo-security/README.md](./docs/features/0006-public-repo-security/README.md)
- [docs/features/0007-default-issue-aware-tab-naming/README.md](./docs/features/0007-default-issue-aware-tab-naming/README.md)
- [docs/adr/0034-default-issue-aware-tab-name-for-tab-launch.md](./docs/adr/0034-default-issue-aware-tab-name-for-tab-launch.md)
- [docs/adr/](./docs/adr/) — Architecture Decision Records
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
  `Waiting for CI`, `Waiting for Code Review`, `Done` и
  `Implementation Blocked`.

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
- `Done`
- `Implementation Blocked`

## Конфигурация

Основной конфиг проекта называется `settings.yml` и лежит в
`./.ai-teamlead/` целевого репозитория.

Коротко:

- `ai-teamlead` всегда читает `./.ai-teamlead/settings.yml` из текущего
  репозитория;
- минимальный active override для MVP это `github.project_id`;
- остальные поля могут приходить из runtime defaults и bootstrap template;
- launcher contract для `zellij.session_name`, `zellij.tab_name`,
  `zellij.launch_target`, `zellij.layout` и `launch_agent.*` вынесен в
  отдельную документацию.

Канонический документ по конфигурации:

- [docs/config.md](./docs/config.md)

Связанные документы:

- [docs/features/0001-ai-teamlead-cli/README.md](./docs/features/0001-ai-teamlead-cli/README.md)
- [docs/features/0002-repo-init/README.md](./docs/features/0002-repo-init/README.md)
- [docs/features/0003-agent-launch-orchestration/README.md](./docs/features/0003-agent-launch-orchestration/README.md)
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
  implementation_branch_template: "implementation/issue-${ISSUE_NUMBER}"
  implementation_worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  implementation_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
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
   `zellij.launch_target`, `zellij.layout` или `launch_agent.*` templates под
   layout проекта
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

## Полный локальный test suite

Канонический локальный entrypoint для полного прогона, включая live AI smoke:

- `./test-local.sh`

Этот скрипт последовательно запускает:

- `cargo test`
- `run-happy-path` в `stub`-режиме
- `live-codex-smoke`
- `live-claude-smoke`

Для успешного прогона локально нужны:

- Docker
- локально доступный `codex`
- локально доступный `claude`
- активная авторизация для обоих live-агентов на хосте
