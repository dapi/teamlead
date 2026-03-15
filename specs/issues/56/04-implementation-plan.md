# Issue 56: План имплементации

Статус: draft
Последнее обновление: 2026-03-15

## Назначение

Этот план связывает analysis-решения по issue `#56` с конкретным порядком
внедрения security baseline для public repos, чтобы hostile-input model,
permission gates и verification появились в runtime как согласованный набор
изменений, а не как набор разрозненных защит.

## Scope

В план входит:

- visibility и operating-mode resolution для public/private/unknown repos;
- сохранение `standard` baseline для private repos без undocumented regression;
- intake policy для auto-start в public repos;
- approval contract и runtime audit trail;
- launcher-level secret filtering для `repo/worktree` filesystem view;
- permission gates для filesystem, network, execution и publication actions;
- выравнивание launcher, prompts, diagnostics и verification вокруг
  `public-safe` режима.

## Вне scope

- полная sandbox-платформа для любых внешних tools и providers;
- защита от скомпрометированной локальной ОС или user account;
- автоматическая санация всего hostile content;
- новый trusted mechanism для repo-local assets без отдельного ADR;
- универсальный security hardening вне control plane `ai-teamlead`.

## Связанные документы

- Issue: https://github.com/dapi/ai-teamlead/issues/56
- Issue spec:
  - [README.md](./README.md)
  - [01-what-we-build.md](./01-what-we-build.md)
  - [02-how-we-build.md](./02-how-we-build.md)
  - [03-how-we-verify.md](./03-how-we-verify.md)
- SSOT:
  - [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md)
  - [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
  - [../../../docs/issue-implementation-flow.md](../../../docs/issue-implementation-flow.md)
- ADR:
  - [../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md](../../../docs/adr/0029-untrusted-github-content-as-hostile-input.md)
  - [../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md](../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md)
- Verification:
  - [03-how-we-verify.md](./03-how-we-verify.md)
  - [../../../docs/features/0005-agent-flow-integration-testing/README.md](../../../docs/features/0005-agent-flow-integration-testing/README.md)
- Code quality:
  - [../../../docs/code-quality.md](../../../docs/code-quality.md)
- Зависимые планы или фичи:
  - [../../../docs/features/0006-public-repo-security/README.md](../../../docs/features/0006-public-repo-security/README.md)

## План изменений документации

- Канонические документы, которые нужно обновить:
  - `docs/untrusted-input-security.md`, если implementation уточнит enforcement
    boundaries, diagnostics contract или runtime semantics относительно текущего
    SSOT;
  - `docs/features/0006-public-repo-security/*`, если уточнятся rollout,
    affected areas или verification;
  - новый ADR нужен только при появлении отдельной стабильной config schema или
    нового trusted mechanism.
- Summary-документы и шаблоны, которые нужно синхронизировать:
  - `README.md`, если public-repo support получит более точный статус;
  - `docs/config.md` и `templates/init/settings.yml`, если runtime change set
    этой issue вводит versioned security settings;
  - project-local prompts и launcher assets, если runtime contract изменит
    operator-visible guidance.
- Документы, которые сознательно не меняются, и почему:
  - существующие ADR `0029/0030`, если реализация только исполняет уже
    принятое решение без изменения архитектурного направления.

## Зависимости и предпосылки

- contract pack по security уже существует и достаточен для старта
  implementation;
- runtime сегодня не содержит полного enforcement baseline для hostile-input
  paths;
- runtime gate без launcher filtering не даст hard deny для уже видимых
  `secret-class` files;
- current launcher defaults и repo-level config docs еще не выровнены с
  целевым approval contract этой feature;
- self-hosted/dogfooding path требует явного trust-priority rule между local
  bootstrap assets и hostile task inputs;
- проверки для `zellij`-связанных сценариев должны идти только в headless path;
- public repo support нельзя считать production-ready до появления хотя бы
  минимального runtime enforcement.

## Порядок работ

### Этап 1. Зафиксировать bootstrap-order, visibility resolution и intake contract

Цель:

- дать каждому запуску явный `public-safe` decision path до начала risky
  actions.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md)
- [../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md](../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md)

Результат этапа:

- определен bootstrap-order для trusted local control plane и hostile task
  inputs;
- runtime умеет определять `repo_visibility`;
- `public` и `unknown` visibility приводят к `public-safe`;
- `private` visibility по умолчанию оставляет `standard`, если config не
  требует `force-public-safe`;
- определен минимальный `secret-class` contract для repo/worktree и host paths;
- зафиксированы правила `poll` vs explicit `run`, включая `manual-override`;
- зафиксированы owner resolution, missing metadata behavior и self-hosted trust
  priority rule;
- зафиксирован mapping `eligible/manual-override/skipped/denied` к runtime
  behavior и flow statuses;
- diagnostics показывают выбранный `operating_mode`.

Проверка:

- unit tests на mode resolution;
- unit tests на intake decision;
- integration checks для `public`, `private` и `unknown` cases.

### Этап 2. Ввести launcher-level secret filtering и path classification

Цель:

- сделать hard deny для `secret-class` enforce-able до запуска risky actions.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md)

Результат этапа:

- launcher формирует agent-visible filesystem view для `repo/worktree`;
- `secret-class` paths (`.env*`, key/cert files и host credential dirs) не
  попадают в обычный research/edit path;
- path classification различает ordinary repo files, repo secrets и external
  host paths;
- verification доказывает, что normal repo docs/code остаются доступны.

Проверка:

- unit tests для path classification и secret globs;
- integration tests на launcher filtering и repo-local secret deny.

### Этап 3. Ввести approval contract и policy-матрицу risky actions

Цель:

- превратить risky actions в проверяемый contract, а не в свободную эвристику.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)

Результат этапа:

- approval в MVP может приходить только из agent session;
- approval привязывается к `session_uuid`, `action_kind` и target;
- определен canonical approval handshake, timeout/deny semantics и
  `target_fingerprint` contract;
- approval lifecycle зафиксирован для reuse, expiration и restart/re-run;
- policy-матрица `allow`/`approval`/`deny` зафиксирована для `filesystem`,
  `network`, `execution`, `publication`.

Проверка:

- unit tests для approval source validation и policy matrix;
- integration tests на deny/approval behavior по всем gate-категориям.

### Этап 4. Внедрить intake policy, permission gates и publication boundaries

Цель:

- сделать intake, gate-решения и publication path enforce-able в runtime.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)

Результат этапа:

- `poll` obeys intake policy, а explicit `run` поддерживает только
  `manual-override` без trust upgrade;
- risky actions классифицируются как минимум по четырем gate-категориям;
- dangerous execution, access вне repo/worktree и публикация потенциально
  чувствительных данных либо требуют approval, либо запрещаются;
- publication path различает канонический GitHub workflow и внешние uploads;
- внешние publish sinks остаются hard `deny` в MVP;
- verification покрывает linked PR/issues, linked artifacts и external content;
- diagnostics объясняют причину deny/approval без утечки секретов.

Проверка:

- unit tests на classification и policy decisions;
- integration tests на deny/approval paths;
- hostile-input scenarios для data exfiltration и execution abuse.

### Этап 5. Синхронизировать prompts, launcher defaults, config и verification

Цель:

- убрать расхождение между runtime policy и operator-facing layer.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/features/0005-agent-flow-integration-testing/README.md](../../../docs/features/0005-agent-flow-integration-testing/README.md)

Результат этапа:

- project-local prompts явно различают `operator intent` и hostile content;
- launcher defaults, config docs и runtime messaging не противоречат approval
  contract;
- missing/malformed security config и legacy flags не могут ослабить
  `public-safe` baseline;
- launcher и runtime messaging показывают `public-safe` режим и причину
  блокировок;
- headless verification покрывает hostile issue, comments, repo-local docs и
  runtime outputs.

Проверка:

- review prompt-layer и launcher assets;
- review `docs/config.md`, `settings.yml` template и launch defaults на
  совместимость с approval contract;
- headless agent-flow scenarios без использования host `zellij`.

## Критерий завершения

- `public-safe` режим детерминированно включается для `public` и `unknown`
  visibility;
- `private` repos сохраняют `standard` baseline без скрытого изменения текущего
  workflow, если нет явного override;
- auto-intake для public repos не стартует hostile issue вне выбранной policy;
- explicit `run` вне intake policy работает только как `manual-override` без
  trust upgrade;
- ordinary repo/worktree files остаются доступны агенту для research/edit path,
  а `secret-class` paths получают enforce-able deny;
- approval в MVP приходит только из agent session и логируется в audit trail;
- approval lifecycle и self-hosted trust priority не оставляют неоднозначности
  для restart/re-run и dogfooding path;
- skipped/denied paths однозначно отражаются в diagnostics и корректно мапятся
  на flow outcome/status;
- high-risk actions проходят через enforce-able permission gates;
- operator получает понятную диагностику причин deny/approval;
- docs, prompts, config surface и tests синхронизированы с runtime behavior.

## Открытые вопросы и риски

- может понадобиться отдельный ADR для стабильной security config schema;
- visibility resolution может потребовать дополнительной GitHub metadata или
  fallback-логики для degraded mode;
- слишком широкая первая версия gates рискует замедлить rollout, слишком узкая
  оставить опасные дыры;
- publication path особенно чувствителен к утечкам и требует отдельной
  дисциплины проверки.

## Журнал изменений

### 2026-03-15

- создан начальный план имплементации для issue `#56`
