# Issue 51: Как проверяем

Статус: approved
Последнее обновление: 2026-03-15
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-14T23:05:41+03:00

## Acceptance Criteria

- `issue-implementation-flow` больше не обрывается на `Waiting for Code Review`
  и содержит явный post-merge terminal path;
- merge канонического implementation PR переводит project item в `Done` и
  закрывает issue;
- связь между каноническим implementation PR, issue state и GitHub Project
  status описана
  детерминированно и без конкурирующих источников истины;
- cleanup implementation runtime/worktree/local branch описан как
  idempotent best-effort contract;
- отсутствие или неоднозначность канонического PR не приводит к silent close
  issue;
- release/deploy semantics явно остаются вне scope текущего изменения;
- verification strategy включает unit, integration и manual/headless проверки
  для merged path.

## Ready Criteria

- зафиксировано, что для MVP post-merge path расширяет
  `issue-implementation-flow`, а не создает новый flow;
- согласовано имя terminal status: `Done`;
- согласован canonical способ идентификации implementation PR;
- в analysis artifacts явно зафиксирована необходимость нового ADR и
  синхронизации ADR-0025/0026;
- определено, что cleanup локальных артефактов не должен откатывать terminal
  business result.

## Invariants

- GitHub Project status остается source of truth по lifecycle issue;
- issue может быть закрыта автоматически только по merge канонического
  implementation PR, а не по любому связанному PR;
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
- observed-state derive rules не зависят от обязательной runtime PR metadata;
- cleanup policy корректно различает удаляемые implementation artifacts и
  сохраняемые versioned docs.

Integration tests:

- issue в `Waiting for Code Review` с merged каноническим implementation PR
  переводится в `Done`, issue закрывается, session/binding помечается
  completed;
- повторный post-merge finalize run остается idempotent;
- `merged` outcome не пытается делать новый commit, push или PR create;
- issue не закрывается, если канонический PR отсутствует, неоднозначен или не
  merged;
- cleanup failure на local worktree дает warning, но terminal status и issue
  close сохраняются;
- regression paths `ready-for-ci`, `ready-for-review` и `needs-rework` не
  ломаются после добавления `merged`.

Manual or headless validation:

- проверить сценарий merge канонического implementation PR на test double или
  headless stub без
  затрагивания host `zellij`;
- проверить one-off reconciliation для issue, где PR уже merged до повторного
  запуска `run`;
- проверить, что cleanup не затрагивает `specs/issues/${ISSUE_NUMBER}` и другие
  versioned документы.

## Verification Checklist

- SSOT и feature docs синхронизированы с новым terminal path;
- новый ADR добавлен, а ADR-0025/0026 обновлены при необходимости;
- config и project statuses поддерживают `Done`;
- runtime contract не является обязательным источником PR identity;
- merged finalization покрыта unit и integration tests;
- idempotency и cleanup diagnostics проверены отдельными сценариями;
- pre-merge implementation flow не получил регрессий.

## Happy Path

1. Issue находится в `Waiting for Code Review`, а канонический implementation
   PR merged в default branch.
2. Post-merge reconciliation path восстанавливает PR identity из GitHub по
   canonical branch contract.
3. Flow подтверждает merge именно нужного PR без обязательной runtime PR
   metadata.
4. `complete-stage --stage implementation --outcome merged` переводит project
   item в `Done` и закрывает issue.
5. Cleanup удаляет implementation runtime/worktree/local branch, если это
   безопасно, и оставляет versioned artifacts нетронутыми.

## Edge Cases

- канонический implementation PR уже merged, но local worktree был удален
  вручную до cleanup;
- GitHub auto-delete уже удалил remote branch, а local branch еще существует;
- issue имеет несколько связанных PR в истории, но закрываться должен только
  канонический implementation PR;
- runtime metadata неполная для старой issue, зависшей в `Waiting for Code Review`.

## Failure Scenarios

- канонический PR не найден или найден неоднозначно;
- канонический PR существует, но еще не merged;
- закрытие issue через GitHub прошло, а cleanup local worktree упал;
- проект не содержит статуса `Done`, и finalization не может корректно обновить
  project item.

## Observability

- finalization пишет отдельные сообщения для merge detection, issue close,
  project status update и каждого cleanup шага;
- warning-сообщения различают blocker до terminalization и cleanup problem после
  terminalization;
- integration logs позволяют восстановить `issue`, `stage`, canonical PR
  identity, итоговый status и cleanup outcome без ручного чтения исходного
  кода.

## Follow-up acceptance 2026-03-15

После принятия
[ADR-0028](../../../docs/adr/0028-github-first-reconcile-and-runtime-cache-only.md)
этот verification contract уточняется:

- требование сериализовать `tracked PR metadata` в runtime больше не является
  обязательным acceptance criterion;
- вместо этого обязательно проверить, что reconcile работает без runtime PR
  metadata и восстанавливает состояние из GitHub Project, canonical PR и git
  refs/worktree;
- неоднозначность нескольких PR для канонической branch должна давать явную
  диагностику, а не неявный выбор.

Остальные критерии про `Done`, issue close, cleanup и regression coverage
остаются без изменения.
