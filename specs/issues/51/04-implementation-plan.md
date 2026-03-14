# Issue 51: План имплементации

Статус: draft
Последнее обновление: 2026-03-14
Статус согласования: pending human review

## Назначение

Этот план задает порядок реализации post-merge lifecycle для implementation PR:
от документационного контракта и status model до merged finalization и cleanup
implementation artifacts.

## Scope

В scope входит:

- расширение `issue-implementation-flow` terminal path после merge;
- новый terminal status `Done`;
- runtime contract для tracked implementation PR;
- merged finalization path в `complete-stage`;
- post-merge cleanup runtime/worktree/local branch;
- unit, integration и headless-friendly verification coverage.

## Вне scope

- автоматический merge PR;
- deploy/release flow после merge;
- новый operator-facing `issue-post-merge-flow`;
- универсальная уборка любых старых worktree вне tracked implementation issue.

## Связанные документы

- Issue: [README.md](./README.md)
- Feature / issue spec:
  [01-what-we-build.md](./01-what-we-build.md),
  [02-how-we-build.md](./02-how-we-build.md),
  [03-how-we-verify.md](./03-how-we-verify.md)
- SSOT:
  [../../../docs/issue-implementation-flow.md](../../../docs/issue-implementation-flow.md),
  [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- ADR:
  [../../../docs/adr/0024-stage-aware-run-dispatch.md](../../../docs/adr/0024-stage-aware-run-dispatch.md),
  [../../../docs/adr/0025-stage-aware-runtime-bindings.md](../../../docs/adr/0025-stage-aware-runtime-bindings.md),
  [../../../docs/adr/0026-stage-aware-complete-stage.md](../../../docs/adr/0026-stage-aware-complete-stage.md)
- Verification:
  [03-how-we-verify.md](./03-how-we-verify.md)
- Code quality:
  [../../../docs/code-quality.md](../../../docs/code-quality.md)
- Зависимые планы или фичи:
  [../../../docs/features/0004-issue-implementation-flow/README.md](../../../docs/features/0004-issue-implementation-flow/README.md),
  [../5/04-implementation-plan.md](../5/04-implementation-plan.md)

## Зависимости и предпосылки

- текущий implementation flow уже умеет доводить issue до
  `Waiting for Code Review`;
- runtime можно безопасно расширить tracked PR metadata без ломки analysis
  binding;
- GitHub Project можно дополнить статусом `Done`;
- post-merge cleanup должен работать без вмешательства в host `zellij`;
- для legacy issues без tracked PR metadata понадобится отдельная
  reconciliation инструкция или fallback path.

## Порядок работ

### Этап 1. Зафиксировать документационный и ADR-контракт post-merge path

Цель:

- обновить SSOT и feature docs до начала кода;
- принять решение, что MVP расширяет `issue-implementation-flow`, а не вводит
  новый third flow.

Основание:

- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)

Результат этапа:

- `docs/issue-implementation-flow.md` описывает post-merge terminal path;
- feature 0004 синхронизирована с новым статусом `Done`;
- создан новый ADR про post-merge lifecycle и обновлены ADR-0025/0026 при
  необходимости.

Проверка:

- документация не содержит противоречий по status model, cleanup и merge
  semantics;
- по ADR можно восстановить причину появления `Done`, tracked PR metadata и
  outcome `merged`.

### Этап 2. Расширить status model и stage guards

Цель:

- сделать `Done` исполнимым terminal state на уровне конфига и доменной логики.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- `settings.yml` и code-level config поддерживают `Done`;
- stage guards различают обычный re-entry и merged reconciliation;
- CLI vocabulary принимает implementation outcome `merged`.

Проверка:

- unit-тесты на config parsing и status transitions;
- regression tests на существующие implementation outcomes.

### Этап 3. Добавить tracked PR metadata в runtime contract

Цель:

- убрать неоднозначность при определении, какой PR завершает issue.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- runtime хранит минимум `pr_number`, `branch` и `worktree_root` для
  implementation stage;
- `ready-for-ci` path сохраняет эти данные при создании или reuse PR;
- legacy-случаи без metadata получают явный fallback/manual reconcile path.

Проверка:

- unit-тесты сериализации runtime schema;
- integration-тесты на create/reuse PR с корректным сохранением metadata.

### Этап 4. Реализовать merged finalization и post-merge cleanup

Цель:

- закрыть разрыв между merged PR и terminal состоянием issue.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- `complete-stage --stage implementation --outcome merged` закрывает issue и
  переводит project item в `Done`;
- merged finalization не создает новый commit, push или PR поверх уже merged
  implementation branch;
- cleanup удаляет implementation runtime/worktree/local branch, когда это
  безопасно;
- cleanup warnings не откатывают terminal business result.

Проверка:

- integration-тесты на happy path, cleanup warning и idempotent rerun;
- manual/headless smoke на merged reconciliation path.

### Этап 5. Закрыть quality bar регрессиями и rollout notes

Цель:

- подтвердить, что post-merge path не ломает analysis и pre-merge
  implementation lifecycle.

Основание:

- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- unit и integration coverage покрывают новый merged path;
- есть операторская инструкция или migration note для legacy issues без
  tracked PR metadata;
- README и feature overview синхронизированы как summary.

Проверка:

- полный целевой test suite зеленый;
- regression-сценарии `ready-for-ci`, `ready-for-review` и `needs-rework`
  остаются валидными.

## Критерий завершения

- implementation lifecycle имеет документированный terminal state `Done`;
- tracked PR metadata и `merged` finalization реализованы и покрыты тестами;
- merge tracked PR закрывает issue и синхронизирует GitHub Project status;
- cleanup implementation artifacts работает как idempotent best-effort path;
- legacy и regression сценарии документированы и проверены.

## Открытые вопросы и риски

- нужно определить точный fallback для уже существующих issues без tracked PR
  metadata;
- при некоторых repo policies закрытие issue сразу после merge может оказаться
  слишком ранним, если позже появится обязательный deploy gate;
- cleanup local branch/worktree зависит от текущего `git worktree` состояния и
  требует аккуратной диагностики.

## Журнал изменений

### 2026-03-14

- создан начальный план имплементации для issue 51
