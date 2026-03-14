# issue-implementation-flow

Статус: draft, evolving
Владелец: владелец репозитория
Роль: SSOT для flow реализации issue
Последнее обновление: 2026-03-14

## Назначение

Этот документ определяет единый источник истины для flow, который берет issue
после утвержденного analysis-плана и проводит ее через coding stage до одного
из следующих результатов:

- изменения реализованы, запушены и ждут обязательные CI checks;
- изменения готовы к human code review;
- реализация возвращена на доработку;
- реализация заблокирована технической или продуктовой проблемой.

`issue-implementation-flow` является отдельным flow от `issue-analysis-flow`, но
оба flow запускаются через один и тот же issue-level CLI entrypoint `run`.

## Scope

Вход:

- GitHub issue находится в состоянии `open`;
- issue находится в настроенном default GitHub Project;
- project status входит в implementation lifecycle.

Выход:

- issue переведена в `Waiting for CI`;
- или issue переведена в `Waiting for Code Review`;
- или issue возвращена в `Implementation In Progress`;
- или issue переведена в `Implementation Blocked`.

Контекст исполнения:

- flow должен быть применим к произвольному подключенному GitHub-репозиторию;
- approved analysis artifacts являются обязательным versioned входом;
- coding stage не должен подменять GitHub Project как source of truth по
  статусу issue;
- repo-specific branch/worktree/launcher semantics должны приходить из
  versioned project-local config и assets.

## Вне scope

- merge automation;
- deploy, release и post-merge operation flow;
- автоматическое принятие code review;
- автоматическое закрытие issue после merge;
- параллельные implementation PR для одной issue.

## Политика развития

`issue-implementation-flow` это живая спецификация и отдельный SSOT для
implementation stage.

Правила развития:

- каждое существенное изменение flow сначала фиксируется в этом файле;
- значимые решения по dispatch, runtime, PR lifecycle и finalization
  оформляются отдельными ADR;
- при изменении статусной модели нужно синхронно обновлять статусы, переходы и
  verification-контракт;
- реализация должна следовать этому SSOT, а не формировать его задним числом;
- если документ разрастается, соседние аспекты выносятся в feature-docs и ADR
  рядом с основным SSOT.

## Источник истины

Источник истины по состоянию issue это поле статуса в настроенном default
GitHub Project.

Repo-local runtime state допускается только для:

- stage-scoped session-binding;
- launcher-артефактов;
- технических данных повторного запуска;
- диагностики конкретной agent session.

Runtime state не должен подменять project status и не должен использоваться для
обхода допустимых переходов.

## Связь с `run`

Пользовательский контракт остается единым:

- оператор всегда вызывает `run <issue>`;
- `run` читает текущий project status issue;
- `run` как stage-aware dispatcher выбирает analysis flow или implementation
  flow;
- после выбора stage система использует соответствующий SSOT, runtime-binding и
  launcher path.

Следствие:

- implementation stage не требует отдельной top-level CLI-команды;
- analysis и implementation flow остаются разными каноническими документами;
- логика выбора stage концентрируется в `run`, а не в prompt-документах.

## Approved analysis artifacts

Implementation flow может стартовать только если approved analysis artifacts
доступны как versioned вход в:

- `specs/issues/${ISSUE_NUMBER}/`

Минимальный контракт:

- пакет анализа имеет `Статус согласования: approved`;
- пакет анализа фиксирует `Approved By` и `Approved At`;
- если approved artifacts отсутствуют или невалидны, implementation flow
  завершается blocker-исходом, а не продолжает работу по догадкам.

## Статусы GitHub Project

Для `issue-implementation-flow` определяются следующие статусы:

1. `Ready for Implementation`
   Значение: analysis-план утвержден человеком, issue готова к coding stage.
2. `Implementation In Progress`
   Значение: агент или разработчик выполняет кодовые изменения и локальную
   валидацию.
3. `Waiting for CI`
   Значение: изменения запушены, draft PR создан или обновлен, issue ожидает
   обязательные CI checks.
4. `Waiting for Code Review`
   Значение: обязательные quality gates пройдены, issue готова к human review.
5. `Implementation Blocked`
   Значение: реализация не может продолжаться без внешнего вмешательства.

## Правила переходов

Разрешенные переходы:

- `Ready for Implementation` -> `Implementation In Progress`
- `Implementation In Progress` -> `Waiting for CI`
- `Implementation In Progress` -> `Implementation Blocked`
- `Waiting for CI` -> `Waiting for Code Review`
- `Waiting for CI` -> `Implementation In Progress`
- `Waiting for Code Review` -> `Implementation In Progress`
- `Implementation Blocked` -> `Implementation In Progress`

Запрещенные переходы:

- прямой переход из `Ready for Implementation` в `Waiting for Code Review`
  без coding stage;
- прямой переход из `Implementation In Progress` в `Waiting for Code Review`
  без PR/CI contract;
- автоматический merge и автоматическое закрытие issue в рамках этого flow.

## Условия входа

Issue может быть запущена через implementation flow только если одновременно
выполняются все условия:

- GitHub issue state = `open`;
- issue прикреплена к настроенному default GitHub Project;
- project status входит в множество:
  `Ready for Implementation`,
  `Implementation In Progress`,
  `Waiting for CI`,
  `Waiting for Code Review`,
  `Implementation Blocked`;
- approved analysis artifacts доступны и валидны;
- для re-entry статусов существует stage-specific runtime-binding или
  документированный способ его восстановить.

## Шаги flow

### 1. Stage dispatch

`run` определяет, что issue находится в implementation lifecycle, и выбирает
`issue-implementation-flow` вместо `issue-analysis-flow`.

### 2. Claim или re-entry

Если issue находится в `Ready for Implementation`:

- `run` переводит ее в `Implementation In Progress`;
- создается implementation session-binding;
- подготавливается implementation branch/worktree и launcher context.

Если issue находится в `Implementation In Progress`:

- `run` выполняет re-entry в текущий implementation context;
- переиспользуется stage-specific binding;
- открывается новая pane или другой stage-specific launcher path в стабильном
  launch context.

Если issue находится в `Waiting for CI`, `Waiting for Code Review` или
`Implementation Blocked`:

- повторный `run` допускается только как явное operator-intent действие;
- issue переводится обратно в `Implementation In Progress`;
- stage-specific binding и implementation branch lifecycle переиспользуются.

### 3. Реализация

Агент или разработчик:

- читает approved analysis artifacts;
- восстанавливает implementation context;
- вносит кодовые изменения;
- запускает обязательные локальные проверки;
- при необходимости обновляет связанные docs и follow-up ADR.

### 4. Finalization

Implementation stage завершается через stage-aware internal CLI-команду
`complete-stage`.

Допустимые outcomes implementation stage:

- `ready-for-ci`
- `ready-for-review`
- `needs-rework`
- `blocked`

Семантика:

- `ready-for-ci`:
  - коммитит и пушит implementation branch;
  - создает или переиспользует draft PR;
  - переводит issue в `Waiting for CI`.
- `ready-for-review`:
  - подтверждает, что обязательные CI checks зеленые;
  - при необходимости переводит PR в ready-for-review;
  - переводит issue в `Waiting for Code Review`.
- `needs-rework`:
  - сохраняет изменения и диагностику;
  - возвращает issue в `Implementation In Progress`.
- `blocked`:
  - по возможности сохраняет прогресс;
  - переводит issue в `Implementation Blocked`.

## Human gate

Human gate обязателен минимум в двух местах:

- при утверждении analysis-плана до входа в `Ready for Implementation`;
- при code review после `Waiting for Code Review`.

При этом:

- зеленый CI не заменяет human review;
- review feedback может вернуть issue в `Implementation In Progress`;
- merge и закрытие issue находятся вне scope текущего flow.

## Протокол оператора

Для MVP достаточно следующих операторских намерений:

1. Запустить реализацию.
   Результат: `run <issue>` переводит issue в `Implementation In Progress` и
   запускает implementation flow.
2. Вернуть в работу после CI или review.
   Результат: `run <issue>` переводит issue обратно в
   `Implementation In Progress` и переиспользует implementation context.
3. Зафиксировать blocker.
   Результат: implementation finalization переводит issue в
   `Implementation Blocked`.
4. Принять реализацию к review.
   Результат: issue оказывается в `Waiting for Code Review`.

## Конфигурация

Project-local config должен поддержать stage-specific implementation contract:

```yaml
issue_implementation_flow:
  statuses:
    ready_for_implementation: "Ready for Implementation"
    implementation_in_progress: "Implementation In Progress"
    waiting_for_ci: "Waiting for CI"
    waiting_for_code_review: "Waiting for Code Review"
    implementation_blocked: "Implementation Blocked"

launch_agent:
  implementation_branch_template: "implementation/issue-${ISSUE_NUMBER}"
  implementation_worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  implementation_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
```

Точный runtime и launcher contract задаются связанными ADR и feature-docs.

## Связанные документы

- [README.md](../README.md)
- [issue-analysis-flow.md](./issue-analysis-flow.md)
- [features/0004-issue-implementation-flow/README.md](./features/0004-issue-implementation-flow/README.md)
- [adr/0024-stage-aware-run-dispatch.md](./adr/0024-stage-aware-run-dispatch.md)
- [adr/0025-stage-aware-runtime-bindings.md](./adr/0025-stage-aware-runtime-bindings.md)
- [adr/0026-stage-aware-complete-stage.md](./adr/0026-stage-aware-complete-stage.md)

## Журнал изменений

### 2026-03-14

- создан SSOT для `issue-implementation-flow`
- зафиксирован единый `run <issue>` как stage-aware entrypoint
- добавлена status model для implementation lifecycle
- добавлен stage-aware finalization contract для implementation outcomes
