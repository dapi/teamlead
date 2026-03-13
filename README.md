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
- создание ветки, worktree, коммита или PR

## Модель запуска

На первом этапе `ai-teamlead` реализуется как standalone daemon, который
запускается в foreground и сам выполняет polling.

Daemon:

1. периодически находит подходящие `open` issues в GitHub Project
2. забирает одну issue в работу
3. запускает flow анализа в заданной `zellij` session и tab

Также должны существовать ручные команды:

- `poll`
- `run`

`systemd --user timer` рассматривается как следующий этап интеграции и не
является базовой моделью запуска для первого MVP.

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
- для `feature` дополнительно обязательны `feature story` и `use cases`
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

Основной конфиг проекта должен называться `ai-teamlead.yml`.

Конфиг хранится в YAML и должен лежать в корне целевого репозитория.

Это означает:

- по умолчанию `ai-teamlead` читает `./ai-teamlead.yml` из текущего репозитория
- разные репозитории могут иметь разные `ai-teamlead.yml`
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
  session_name: "ai-teamlead"
  tab_name: "issue-analysis"
```

Обязательные поля MVP:

- `github.project_id`
- `issue_analysis_flow.statuses.*`
- `runtime.max_parallel`
- `runtime.poll_interval_seconds`
- `zellij.session_name`
- `zellij.tab_name`

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
runtime-артефактах внутри `.git/ai-teamlead/`.

Ограничение MVP:

- целевое значение `runtime.max_parallel = 1`
- если используется одна фиксированная `zellij` tab, значения больше `1` пока
  не поддерживаются корректно
