# Feature 0004: issue implementation flow

Статус: draft
Владелец: владелец репозитория
Последнее обновление: 2026-03-15

## Контекст

Эта feature описывает следующий stage после analysis: реализацию issue на базе
утвержденного SDD-комплекта.

Важно:

- analysis и implementation остаются разными flow;
- оператор по-прежнему использует единый `run <issue>`;
- `run` сам определяет, какой stage и какой flow должны быть запущены;
- coding stage требует собственного runtime, launcher и PR/CI lifecycle.

## Что строим

- [01-what-we-build.md](./01-what-we-build.md)

## Как строим

- [02-how-we-build.md](./02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## План реализации

- [04-implementation-plan.md](./04-implementation-plan.md)

## Связанные документы

- [README.md](../../../README.md)
- [docs/issue-analysis-flow.md](../../issue-analysis-flow.md)
- [docs/issue-implementation-flow.md](../../issue-implementation-flow.md)
- [docs/features/0001-ai-teamlead-cli/README.md](../0001-ai-teamlead-cli/README.md)
- [docs/features/0003-agent-launch-orchestration/README.md](../0003-agent-launch-orchestration/README.md)
- [docs/adr/0024-stage-aware-run-dispatch.md](../../adr/0024-stage-aware-run-dispatch.md)
- [docs/adr/0025-stage-aware-runtime-bindings.md](../../adr/0025-stage-aware-runtime-bindings.md)
- [docs/adr/0026-stage-aware-complete-stage.md](../../adr/0026-stage-aware-complete-stage.md)
- [docs/adr/0027-post-merge-implementation-lifecycle.md](../../adr/0027-post-merge-implementation-lifecycle.md)
- [docs/adr/0028-github-first-reconcile-and-runtime-cache-only.md](../../adr/0028-github-first-reconcile-and-runtime-cache-only.md)

## Открытые вопросы

- нужен ли отдельный implementation prompt для разных типов задач или в первой
  версии достаточно одного project-local prompt entrypoint;
- должен ли перевод PR из draft в ready-for-review происходить внутри
  finalization command или оставаться явным human gate;
- потребуется ли в будущем отдельный post-merge flow для deploy/release path
  поверх базового terminal status `Done`.

## Журнал изменений

### 2026-03-14

- создана feature 0004 для `issue-implementation-flow`

### 2026-03-15

- принят
  [ADR-0028](../../adr/0028-github-first-reconcile-and-runtime-cache-only.md)
  и зафиксирован GitHub-first reconcile для implementation flow
