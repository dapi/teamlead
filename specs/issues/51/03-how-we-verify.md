# Issue 51: Как проверяем

Статус: draft
Последнее обновление: 2026-03-14
Статус согласования: pending human review

## Acceptance Criteria

- `issue-implementation-flow` больше не обрывается на `Waiting for Code Review`
  и содержит явный post-merge terminal path;
- merge tracked implementation PR переводит project item в `Done` и закрывает
  issue;
- связь между tracked PR, issue state и GitHub Project status описана
  детерминированно и без конкурирующих источников истины;
- cleanup implementation runtime/worktree/local branch описан как
  idempotent best-effort contract;
- отсутствие или неоднозначность tracked PR не приводит к silent close issue;
- release/deploy semantics явно остаются вне scope текущего изменения;
- verification strategy включает unit, integration и manual/headless проверки
  для merged path.

## Ready Criteria

- зафиксировано, что для MVP post-merge path расширяет
  `issue-implementation-flow`, а не создает новый flow;
- согласовано имя terminal status: `Done`;
- согласован canonical способ идентификации tracked PR;
- в analysis artifacts явно зафиксирована необходимость нового ADR и
  синхронизации ADR-0025/0026;
- определено, что cleanup локальных артефактов не должен откатывать terminal
  business result.

## Invariants

- GitHub Project status остается source of truth по lifecycle issue;
- issue может быть закрыта автоматически только по merge tracked implementation
  PR, а не по любому связанному PR;
- approved versioned artifacts в `specs/issues/${ISSUE_NUMBER}` не удаляются
  post-merge cleanup path;
- post-merge finalization остается idempotent при повторном запуске;
- cleanup warning не должен тихо оставлять систему в противоречивом состоянии
  без явной диагностики.

## Test Plan

Unit tests:

- config поддерживает terminal implementation status `Done`;
- stage domain корректно валидирует переход `Waiting for Code Review` -> `Done`;
- parser outcome vocabulary принимает `merged` только для implementation stage;
- runtime schema сериализует и десериализует tracked PR metadata;
- cleanup policy корректно различает удаляемые implementation artifacts и
  сохраняемые versioned docs.

Integration tests:

- issue в `Waiting for Code Review` с merged tracked PR переводится в `Done`,
  issue закрывается, session/binding помечается completed;
- повторный post-merge finalize run остается idempotent;
- `merged` outcome не пытается делать новый commit, push или PR create;
- issue не закрывается, если tracked PR отсутствует или не merged;
- cleanup failure на local worktree дает warning, но terminal status и issue
  close сохраняются;
- regression paths `ready-for-ci`, `ready-for-review` и `needs-rework` не
  ломаются после добавления `merged`.

Manual or headless validation:

- проверить сценарий merge tracked PR на test double или headless stub без
  затрагивания host `zellij`;
- проверить one-off reconciliation для issue, где PR уже merged до повторного
  запуска `run`;
- проверить, что cleanup не затрагивает `specs/issues/${ISSUE_NUMBER}` и другие
  versioned документы.

## Verification Checklist

- SSOT и feature docs синхронизированы с новым terminal path;
- новый ADR добавлен, а ADR-0025/0026 обновлены при необходимости;
- config и project statuses поддерживают `Done`;
- runtime contract хранит tracked PR identity;
- merged finalization покрыта unit и integration tests;
- idempotency и cleanup diagnostics проверены отдельными сценариями;
- pre-merge implementation flow не получил регрессий.

## Happy Path

1. Issue находится в `Waiting for Code Review`, а tracked implementation PR
   merged в default branch.
2. Post-merge reconciliation path читает tracked PR identity из runtime.
3. Flow подтверждает merge именно нужного PR.
4. `complete-stage --stage implementation --outcome merged` переводит project
   item в `Done` и закрывает issue.
5. Cleanup удаляет implementation runtime/worktree/local branch, если это
   безопасно, и оставляет versioned artifacts нетронутыми.

## Edge Cases

- tracked PR уже merged, но local worktree был удален вручную до cleanup;
- GitHub auto-delete уже удалил remote branch, а local branch еще существует;
- issue имеет несколько связанных PR в истории, но закрываться должен только
  tracked implementation PR;
- runtime metadata неполная для старой issue, зависшей в `Waiting for Code Review`.

## Failure Scenarios

- tracked PR не найден в runtime metadata;
- tracked PR существует, но еще не merged;
- закрытие issue через GitHub прошло, а cleanup local worktree упал;
- проект не содержит статуса `Done`, и finalization не может корректно обновить
  project item.

## Observability

- finalization пишет отдельные сообщения для merge detection, issue close,
  project status update и каждого cleanup шага;
- warning-сообщения различают blocker до terminalization и cleanup problem после
  terminalization;
- integration logs позволяют восстановить `issue`, `stage`, `pr_number`,
  итоговый status и cleanup outcome без ручного чтения исходного кода.
