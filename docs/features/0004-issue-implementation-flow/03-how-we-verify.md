# Feature 0004: Как проверяем

## Критерии корректности

Решение считается корректным, если:

- `run <issue>` корректно различает analysis и implementation lifecycle;
- implementation flow стартует только при наличии approved analysis artifacts;
- implementation runtime не перезаписывает analysis runtime-binding;
- implementation branch/worktree naming читается из `settings.yml`;
- finalization contract корректно переводит issue в
  `Waiting for CI`, `Waiting for Code Review`, `Done`,
  `Implementation In Progress` или `Implementation Blocked`;
- draft PR и CI checks участвуют в status transitions implementation stage;
- semantic state issue восстанавливается из GitHub, а не только из runtime.

## Критерии готовности

Feature считается готовой, если:

- оператор может одним `run <issue>` запускать issue и на analysis, и на
  implementation stage;
- implementation flow покрыт unit и integration tests;
- для implementation stage есть headless-friendly smoke path;
- README, SSOT, feature-docs и ADR синхронизированы.

## Инварианты

- `run` остается единым issue-level entrypoint;
- `issue-analysis-flow` и `issue-implementation-flow` остаются разными SSOT;
- approved analysis artifacts обязательны для implementation stage;
- один issue может иметь не более одного активного runtime-binding на stage;
- implementation branch не совпадает с analysis branch;
- без локальных проверок issue не должна переходить в `Waiting for CI`;
- runtime не должен быть единственным обязательным источником данных для
  post-merge reconcile.

## Сценарии проверки

### Сценарий 1. Вход из `Ready for Implementation`

- оператор запускает `run <issue>`;
- dispatcher выбирает implementation flow;
- issue переходит в `Implementation In Progress`;
- создается implementation context.

### Сценарий 2. Повторный запуск из `Waiting for CI`

- оператор повторно запускает `run <issue>`;
- issue возвращается в `Implementation In Progress`;
- используется существующий implementation binding.

### Сценарий 3. Finalization в `Waiting for CI`

- локальные проверки пройдены;
- finalization делает commit, push, draft PR;
- issue получает статус `Waiting for CI`.

### Сценарий 4. Finalization в `Waiting for Code Review`

- обязательные CI checks зеленые;
- finalization переводит issue в `Waiting for Code Review`.

### Сценарий 5. Blocker

- implementation flow упирается в технический блокер;
- finalization переводит issue в `Implementation Blocked`;
- runtime diagnostics позволяют повторный запуск после снятия блокера.

### Сценарий 6. Post-merge reconcile

- issue находится в `Waiting for Code Review`;
- implementation PR по canonical branch уже merged;
- повторный `run <issue>` или `complete-stage --outcome merged` переводит issue
  в `Done` без нового agent launch.

## Диагностика и наблюдаемость

Минимально необходимо видеть:

- какой stage выбран dispatcher-ом;
- какой runtime-binding используется;
- какой implementation branch/worktree выбран;
- какой outcome передан в finalization;
- какой canonical branch и какой PR связаны с issue;
- какие checks считаются обязательными для перехода к code review;
- почему merged reconciliation завершилась в `Done` или не сработала.

## Follow-up acceptance 2026-03-15

Принятый
[ADR-0028](../../adr/0028-github-first-reconcile-and-runtime-cache-only.md)
добавляет к verification contract три обязательных проверки:

- отсутствие `tracked PR metadata` не ломает deterministic reconcile;
- `last_known_flow_status` остается только cache/diagnostic полем;
- canonical branch contract достаточен для однозначного выбора PR или дает
  явную диагностику неоднозначности.
