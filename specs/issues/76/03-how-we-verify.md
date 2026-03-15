# Issue 76: Как проверяем

Статус: draft
Последнее обновление: 2026-03-15

## Acceptance Criteria

- agent-side `complete-stage` больше не выполняет прямые `git`/`gh` side
  effects из sandboxed session;
- host-side supervisor принимает structured completion request и применяет
  только разрешенные для stage/outcome privileged actions;
- `workspace-write` + linked worktree scenario успешно завершается без
  `danger-full-access`;
- analysis и implementation сохраняют единый stage-aware completion contract;
- существует replay path для request, если host-side apply прервался частичным
  сбоем;
- audit trail показывает, какой request был принят и какие side effects реально
  выполнены;
- связь решения с hostile-input model и `public-safe` boundary явно
  задокументирована.

## Ready Criteria

- обновлены SSOT для analysis и implementation finalization;
- создан и принят новый ADR по host-side supervisor boundary;
- обновлены feature-docs про orchestration и public repo security;
- реализация покрыта unit/integration tests нужного уровня;
- headless verification path задокументирован и не затрагивает host `zellij`
  пользователя;
- launcher, runtime и diagnostics согласованы с новым contract layer.

## Invariants

- GitHub Project status остается единственным semantic source of truth;
- agent request не является authority для branch, repo, target status или
  набора side effects;
- privileged `git`/`gh` операции выполняются только host-side supervisor-ом;
- worktree-local mailbox используется только как transient transport layer;
- request можно replay-ить без повторного запуска агента;
- supervisor не коммитит ничего вне разрешенного artifacts path для конкретной
  issue;
- failure create/update PR не должен silently теряться: он либо warning, либо
  blocker по явно описанному правилу.

## Test Plan

### Unit tests

- сериализация и валидация completion request payload;
- отказ, если `message` пустой, `stage`/`outcome` несовместимы или request
  не совпадает с trusted manifest;
- выбор target status и allowed side effects по stage/outcome;
- валидация разрешенного artifacts path и branch contract;
- audit entry generation для `applied`, `partial`, `failed`, `replayed`.

### Integration tests

- agent-flow сценарий в headless/Docker path, где агент в `workspace-write`
  пишет request, а host-side supervisor делает commit/push/status update;
- regression сценарий `plan-ready` для analysis;
- regression сценарии `ready-for-ci`, `ready-for-review`, `merged`,
  `needs-rework`, `blocked` для implementation;
- сценарий частичного сбоя: request записан, `git push` или `gh` падает,
  request остается pending и может быть replay-нут;
- сценарий missing request: агент завершился без `complete-stage`, статус issue
  не меняется, diagnostics понятны оператору;
- сценарий stale/duplicate request: supervisor не применяет side effects дважды.

### Manual or smoke validation

- headless прогон в linked worktree с общей git metadata вне writable scope
  sandbox-а;
- ручная проверка, что после успешного supervisor apply worktree не содержит
  случайно закоммиченных runtime mailbox файлов;
- ручная проверка audit artifacts и replay команды после искусственного сбоя.

## Verification Checklist

- request file создается в worktree-local mailbox и не попадает в commit;
- supervisor читает trusted manifest из `.git/.ai-teamlead/`, а не из request;
- commit/push/status transition выполняются только host-side;
- warnings и failures отражаются в audit trail и пользовательской диагностике;
- replay path доигрывает pending request без повторного запуска агента;
- analysis и implementation prompts по-прежнему используют один entrypoint
  `internal complete-stage`;
- связанная документация обновлена до или вместе с кодом.

## Operational Validation

- для `public-safe` контекста privileged publication path теперь проходит через
  один trusted boundary, который можно дополнительно обвешивать permission
  gates;
- оператор по launcher log и audit artifacts понимает, на каком шаге сломался
  finalization path;
- при временной ошибке внешней интеграции достаточно replay supervisor path, а
  не переписывать или перезапускать agent session.

## Failure Scenarios

- request файл записан, но supervisor не смог прочитать trusted manifest;
- commit прошел, но `git push` не удался;
- push прошел, но `gh pr create` или `gh pr ready` вернули warning/error;
- project status update не удался после уже созданного commit;
- request payload не совпадает с trusted stage/session binding;
- request уже был применен, но оператор повторно запустил replay.

Для каждого из этих случаев система должна оставлять понятный audit artifact и
не терять исходный request до безопасного завершения или явного отказа.

## Observability

- launcher log должен фиксировать запуск агента, обнаружение request и запуск
  supervisor apply path;
- audit artifacts должны хранить результат каждой попытки применения request;
- stdout/stderr supervisor-а должны различать `warning`, `partial failure` и
  `terminal blocker`;
- при replay должен быть виден факт повторной обработки, а не только конечный
  статус.
