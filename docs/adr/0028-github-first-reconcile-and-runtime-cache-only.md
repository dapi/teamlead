# ADR-0028: GitHub-first reconcile и runtime только как cache/execution metadata

Статус: accepted
Дата: 2026-03-15
Связанный issue: #51

## Контекст

После принятия ADR-0025/0026/0027 первая implementation-версия начала
закладываться на два runtime-механизма:

- `issues/<issue_number>.json.last_known_flow_status`;
- `sessions/<session_uuid>/session.json.tracked_pr_number/url`.

Это дало быстрый путь для post-merge reconciliation, но породило архитектурный
конфликт с уже принятым
[ADR-0003](./0003-github-project-status-as-source-of-truth.md).

Проблема в том, что runtime начал претендовать не только на execution context,
но и на semantic state issue:

- `run <issue>` не всегда может восстановить реальное состояние, если runtime
  потерян, устарел или записан частично;
- tracked PR metadata становится вторым источником истины наряду с GitHub;
- `last_known_flow_status` выглядит как реплика project status, но может
  расходиться с ним;
- потеря runtime не должна ломать возможность корректно продолжить flow.

Если целевое состояние issue живет в GitHub, то локальный runtime может быть
только кэшем и техническим storage, но не обязательным semantic contract.

## Решение

Для implementation flow принимается GitHub-first модель reconcile.

Принципы:

- source of truth по lifecycle issue остается GitHub Project status;
- source of truth по существованию и фазе PR остается GitHub Pull Request;
- source of truth по branch/worktree presence остается наблюдаемое git
  состояние;
- runtime хранит только execution metadata, session binding и optional cache;
- удаление runtime не должно делать issue невосстановимой для `run <issue>`.

### Каноническая идентификация implementation PR

Implementation PR определяется не по сохраненному `pr_number`, а по
каноническому branch contract:

- для issue `N` canonical implementation branch это
  `implementation/issue-N`;
- PR считается каноническим implementation PR, если его `headRefName`
  совпадает с этим branch name;
- если для одного canonical branch найдено больше одного PR, это считается
  неоднозначностью и требует явной диагностики, а не неявной эвристики.

### Что использует reconcile

`run <issue>` сначала собирает observed state из живых источников:

- GitHub Project status;
- PR по canonical branch;
- наличие remote branch;
- наличие local branch;
- наличие local worktree для canonical branch.

На основе этого восстанавливается execution decision.

### Что runtime может хранить

Допустимые runtime-данные:

- stage-scoped session binding;
- zellij binding;
- `stage_branch`, `stage_worktree_root`, `stage_artifacts_dir` как
  execution/cache metadata;
- timestamps и диагностические данные.

Недопустимо считать обязательной semantic truth:

- `tracked_pr_number`;
- `tracked_pr_url`;
- `last_known_flow_status`.

## Последствия

Плюсы:

- `run <issue>` остается восстанавливаемым после потери runtime;
- исчезает дублирование истины между GitHub и локальным runtime;
- post-merge reconcile становится детерминированным через canonical branch;
- проще объяснить, почему issue находится в текущем состоянии.

Минусы:

- branch naming contract становится строже;
- reconcile logic в `run` усложняется;
- неоднозначные PR/branch ситуации придется явно диагностировать;
- потребуется миграция docs, runtime schema и tests.

Этот ADR частично supersede-ит:

- [ADR-0025](./0025-stage-aware-runtime-bindings.md) в части попытки хранить
  semantic state issue в runtime;
- [ADR-0026](./0026-stage-aware-complete-stage.md) в части, где
  implementation reconciliation могло зависеть от runtime как от обязательного
  источника истины;
- [ADR-0027](./0027-post-merge-implementation-lifecycle.md) в части механизма
  `tracked_pr_*` как обязательного post-merge identity contract.

## Альтернативы

### 1. Оставить tracked PR metadata как обязательную часть runtime

Отклонено.

Это удобно для первой реализации, но делает runtime вторым источником истины и
ломает восстановимость flow после потери локального state.

### 2. Закрывать issue по любому PR, связанному с issue number

Отклонено.

Это слишком эвристично и не дает надежного соответствия между issue и
implementation lifecycle.

## Связанные документы

- [../issue-implementation-flow.md](../issue-implementation-flow.md)
- [./0003-github-project-status-as-source-of-truth.md](./0003-github-project-status-as-source-of-truth.md)
- [./0024-stage-aware-run-dispatch.md](./0024-stage-aware-run-dispatch.md)
- [./0025-stage-aware-runtime-bindings.md](./0025-stage-aware-runtime-bindings.md)
- [./0026-stage-aware-complete-stage.md](./0026-stage-aware-complete-stage.md)
- [./0027-post-merge-implementation-lifecycle.md](./0027-post-merge-implementation-lifecycle.md)

## Журнал изменений

### 2026-03-15

- принят GitHub-first reconcile для implementation flow;
- runtime зафиксирован в роли cache/execution metadata;
- принят отказ от `tracked_pr_*` и `last_known_flow_status` как semantic source
  of truth;
- соответствующие части ADR-0025/0026/0027 частично superseded этим ADR
