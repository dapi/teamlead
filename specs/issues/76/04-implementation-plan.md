# Issue 76: План имплементации

Статус: draft
Последнее обновление: 2026-03-15

## Назначение

Этот план задает порядок реализации host-side supervisor path для stage
completion, чтобы изменения в runtime, launcher, flow SSOT, security docs и
тестах были прослеживаемыми и не разъехались между analysis и implementation
stage.

## Scope

В план входит:

- новый signal-only contract для agent-side `complete-stage`;
- host-side supervisor, который применяет privileged side effects;
- worktree-local mailbox и host-side audit trail;
- replay/failure semantics;
- обновление SSOT, feature-docs, ADR и verification.

## Вне scope

- отдельный long-running daemon supervisor;
- redesign всех остальных privileged actions вне stage finalization;
- перенос всего runtime state из `.git/.ai-teamlead/` в worktree;
- полный public-safe enforcement beyond completion boundary.

## Связанные документы

- Issue: https://github.com/dapi/ai-teamlead/issues/76
- Feature / issue spec:
  - [README.md](./README.md)
  - [01-what-we-build.md](./01-what-we-build.md)
  - [02-how-we-build.md](./02-how-we-build.md)
  - [03-how-we-verify.md](./03-how-we-verify.md)
- SSOT:
  - [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
  - [../../../docs/issue-implementation-flow.md](../../../docs/issue-implementation-flow.md)
  - [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md)
- ADR:
  - [../../../docs/adr/0020-agent-session-completion-signal.md](../../../docs/adr/0020-agent-session-completion-signal.md)
  - [../../../docs/adr/0026-stage-aware-complete-stage.md](../../../docs/adr/0026-stage-aware-complete-stage.md)
  - [../../../docs/adr/0028-github-first-reconcile-and-runtime-cache-only.md](../../../docs/adr/0028-github-first-reconcile-and-runtime-cache-only.md)
  - [../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md](../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md)
  - [../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md](../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md)
- Verification:
  - [03-how-we-verify.md](./03-how-we-verify.md)
- Code quality:
  - [../../../docs/code-quality.md](../../../docs/code-quality.md)
- Зависимые планы или фичи:
  - [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
  - [../../../docs/features/0004-issue-implementation-flow/README.md](../../../docs/features/0004-issue-implementation-flow/README.md)
  - [../../../docs/features/0006-public-repo-security/README.md](../../../docs/features/0006-public-repo-security/README.md)

## План изменений документации

- Канонические документы, которые нужно обновить:
  - `docs/issue-analysis-flow.md`
  - `docs/issue-implementation-flow.md`
  - `docs/untrusted-input-security.md`
  - новый ADR про host-side supervisor boundary
- Summary-документы и шаблоны, которые нужно синхронизировать:
  - `README.md` как repo-level summary completion contract
  - `docs/features/0003-agent-launch-orchestration/*`
  - `docs/features/0004-issue-implementation-flow/*`
  - `docs/features/0006-public-repo-security/*`
  - project-local flow entrypoints в `.ai-teamlead/flows/`
  - bootstrap/init assets и `.gitignore`, если вводится worktree-local mailbox
- Документы, которые сознательно не меняются, и почему:
  - `docs/config.md`, если mailbox path остается internal runtime convention без
    нового user-facing config;
  - unrelated ADR и feature-docs, не затрагивающие completion boundary.

## Зависимости и предпосылки

- stage-aware vocabulary `complete-stage` уже принят и должен быть сохранен;
- existing runtime manifest уже хранит trusted stage/worktree/branch binding;
- linked worktree + `workspace-write` должен стать primary regression scenario;
- `zellij`-связанные интеграционные проверки остаются только в headless path;
- новая архитектура должна дать основу для future `public-safe` enforcement, но
  не обязана закрывать весь issue `#56`.

## Порядок работ

### Этап 1. Зафиксировать новый contract layer в документации

Цель:

- определить каноническую trust boundary до изменения кода.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [../../../docs/documentation-process.md](../../../docs/documentation-process.md)

Результат этапа:

- обновлены analysis/implementation SSOT вокруг finalization semantics;
- создан новый ADR про host-side supervisor и signal-only `complete-stage`;
- security docs и feature 0003/0004/0006 синхронизированы с новым boundary.

Проверка:

- doc review на отсутствие двойного контракта между ADR-0020, ADR-0026 и новым
  ADR;
- все ссылки между issue spec, SSOT и ADR разрешаются без ручного поиска.

### Этап 2. Вынести request/audit primitives в runtime layer

Цель:

- подготовить machine-readable transport и audit substrate без прямых git/gh
  side effects.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- agent-side `complete-stage` пишет completion request в worktree-local mailbox;
- runtime/service layer умеет читать pending request и писать audit attempts;
- validation отделена от фактического выполнения privileged actions.

Проверка:

- unit tests на payload schema, parsing, validation и audit entries;
- regression на несовместимые `stage`/`outcome` и пустой `message`.

### Этап 3. Подключить host-side supervisor к launcher path

Цель:

- сделать существующий orchestration path parent-process supervisor-ом для
  sandboxed agent session.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)

Результат этапа:

- `launch-agent.sh` больше не теряет контроль после старта агента;
- после выхода агента host-side internal command применяет validated request;
- существует явный replay path для partial failure и crash recovery.

Проверка:

- integration tests на success path и replay после частичного сбоя;
- ручная проверка launcher log и audit trail.

### Этап 4. Перенести существующие side effects под supervisor и покрыть регрессии

Цель:

- сохранить текущие бизнес-семантики analysis/implementation outcomes без
  прямого privileged path из sandbox.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)

Результат этапа:

- commit/push/PR/status/issue-close logic вызывается только из supervisor-а;
- `merged` outcome остается GitHub-first path и работает через тот же trusted
  boundary;
- completion mailbox не попадает в commit и не оставляет случайный runtime
  мусор после успешного apply.

Проверка:

- headless integration scenarios для analysis и implementation outcomes;
- regression на `workspace-write` linked-worktree path;
- smoke-проверка ручного replay для pending request.

## Критерий завершения

- `complete-stage` больше не требует прямого privileged git/gh path из
  sandboxed agent session;
- host-side supervisor валидирует и применяет только разрешенные side effects;
- docs, ADR, launcher contract и runtime behavior синхронизированы;
- есть audit trail и replay path;
- headless verification подтверждает работоспособность `workspace-write` в
  linked worktree без `danger-full-access`.

## Открытые вопросы и риски

- нужно аккуратно выбрать имя и lifecycle worktree-local mailbox directory,
  чтобы он не конфликтовал с уже зафиксированной ролью `.ai-teamlead/`;
- если supervisor будет жить только в launcher path без явного replay command,
  recovery окажется слишком хрупким;
- важно не допустить silent partial success, когда commit уже создан, а request
  при этом считается полностью непримененным;
- если для public-safe mode понадобятся дополнительные publication gates,
  реализация не должна зацементировать более слабую boundary-модель.

## Журнал изменений

### 2026-03-15

- создан начальный план имплементации для issue `#76`
