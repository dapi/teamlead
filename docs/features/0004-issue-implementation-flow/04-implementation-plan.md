# Feature 0004: План реализации

Статус: draft
Последнее обновление: 2026-03-14

## Назначение

Этот план задает порядок реализации `issue-implementation-flow` как нового
stable stage в `ai-teamlead`.

## Scope

В scope входит:

- новый SSOT implementation stage;
- stage-aware dispatch внутри `run`;
- config contract implementation statuses и workspace templates;
- stage-aware runtime-binding;
- implementation launcher and finalization path;
- тесты для нового stage.

Вне scope:

- merge automation;
- release/deploy flow;
- множественные implementation PR на одну issue.

## Связанные документы

- [README.md](./README.md)
- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../issue-analysis-flow.md](../../issue-analysis-flow.md)
- [../../issue-implementation-flow.md](../../issue-implementation-flow.md)
- [../../code-quality.md](../../code-quality.md)
- [../../adr/0024-stage-aware-run-dispatch.md](../../adr/0024-stage-aware-run-dispatch.md)
- [../../adr/0025-stage-aware-runtime-bindings.md](../../adr/0025-stage-aware-runtime-bindings.md)
- [../../adr/0026-stage-aware-complete-stage.md](../../adr/0026-stage-aware-complete-stage.md)

## Зависимости и предпосылки

- analysis stage уже умеет доводить issue до `Ready for Implementation`;
- approved analysis artifacts доступны как versioned input;
- проект готов добавить новые GitHub Project statuses;
- `gh` CLI доступен для PR/checks operations.

## Порядок работ

### Этап 1. Stage-aware documentation layer

Цель:

- зафиксировать SSOT, feature и ADR до изменения кода.

Основание:

- implementation stage меняет CLI, runtime и status model.

Результат этапа:

- есть `docs/issue-implementation-flow.md`;
- есть feature 0004;
- есть ADR по dispatch, runtime и finalization.

### Этап 2. Расширить `run` до stage-aware dispatcher

Цель:

- сохранить единый пользовательский вход `run <issue>`.

Основание:

- это ожидаемый пользовательский контракт.

Результат этапа:

- `run` различает analysis и implementation statuses;
- `run` выбирает нужный flow path по current project status.

### Этап 3. Добавить implementation config и runtime model

Цель:

- сделать implementation stage исполнимой и повторно запускаемой.

Основание:

- без stage-aware runtime implementation binding конфликтует с analysis.

Результат этапа:

- config поддерживает implementation statuses и templates;
- runtime хранит stage binding отдельно;
- повторный `run` умеет находить implementation context.

### Этап 4. Реализовать launcher и finalization

Цель:

- довести implementation stage до PR/CI lifecycle.

Основание:

- issue требует явный contract для commit, push и PR.

Результат этапа:

- есть implementation launcher path;
- `complete-stage` поддерживает implementation outcomes;
- issue корректно движется по implementation statuses.

### Этап 5. Закрыть verification

Цель:

- доказать, что новый stage не ломает analysis MVP.

Основание:

- `docs/code-quality.md` требует тесты на каждую значимую feature.

Результат этапа:

- добавлены unit и integration tests;
- есть headless smoke path;
- analysis и implementation regression остаются зелеными.

## Критерий завершения

Feature можно считать реализованной, если:

- `run <issue>` stage-aware;
- implementation statuses поддержаны в config и коде;
- runtime различает analysis и implementation binding;
- implementation finalization доводит issue минимум до `Waiting for CI`;
- tests покрывают основной и re-entry path implementation stage.

## Риски и открытые вопросы

- migration runtime schema может потребовать отдельной совместимости со старыми
  analysis sessions;
- CI checks могут быть слишком медленными для синхронного completion path;
- перевод PR в ready-for-review может оказаться отдельным human gate.

## Журнал изменений

### 2026-03-14

- создан implementation plan для feature 0004
