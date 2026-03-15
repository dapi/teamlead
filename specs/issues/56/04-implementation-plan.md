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
- intake policy для auto-start в public repos;
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
- Feature / issue spec:
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
  - `docs/untrusted-input-security.md`, если implementation уточнит security
    config surface, enforcement boundaries или diagnostics contract;
  - `docs/features/0006-public-repo-security/*`, если уточнятся rollout,
    affected areas или verification;
  - новый ADR нужен только при появлении отдельной стабильной config schema или
    нового trusted mechanism.
- Summary-документы и шаблоны, которые нужно синхронизировать:
  - `README.md`, если public-repo support получит более точный статус;
  - `docs/config.md` и `templates/init/settings.yml`, если появятся versioned
    security settings;
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
- проверки для `zellij`-связанных сценариев должны идти только в headless path;
- public repo support нельзя считать production-ready до появления хотя бы
  минимального runtime enforcement.

## Порядок работ

### Этап 1. Ввести visibility и operating-mode resolution

Цель:

- дать каждому запуску явный security mode до начала risky actions.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md)
- [../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md](../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md)

Результат этапа:

- runtime умеет определять `repo_visibility`;
- `public` и `unknown` visibility приводят к `public-safe`;
- diagnostics показывают выбранный `operating_mode`.

Проверка:

- unit tests на mode resolution;
- integration checks для `public`, `private` и `unknown` cases.

### Этап 2. Добавить intake policy для public repos

Цель:

- ограничить auto-start hostile issue еще до входа в agent workflow.

Основание:

- [01-what-we-build.md](./01-what-we-build.md)
- [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md)
- [../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md](../../../docs/adr/0030-public-repo-safe-mode-and-permission-gates.md)

Результат этапа:

- `poll` и `run` учитывают author-based intake policy для public repos;
- owner-authored issue рассматривается только как intake gate;
- comments остаются hostile-by-default и не получают trust upgrade.

Проверка:

- unit tests для author-policy resolution;
- integration tests на deny/skip behavior для issue вне allowlist.

### Этап 3. Внедрить permission gates для high-risk actions

Цель:

- сделать filesystem, network, execution и publication paths enforce-able.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)

Результат этапа:

- risky actions классифицируются как минимум по четырем gate-категориям;
- dangerous execution, access вне repo/worktree и публикация потенциально
  чувствительных данных либо требуют approval, либо запрещаются;
- diagnostics объясняют причину deny/approval без утечки секретов.

Проверка:

- unit tests на classification и policy decisions;
- integration tests на deny/approval paths;
- hostile-input scenarios для data exfiltration и execution abuse.

### Этап 4. Синхронизировать prompts, launcher и verification

Цель:

- убрать расхождение между runtime policy и operator-facing layer.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/features/0005-agent-flow-integration-testing/README.md](../../../docs/features/0005-agent-flow-integration-testing/README.md)

Результат этапа:

- project-local prompts явно различают `operator intent` и hostile content;
- launcher и runtime messaging показывают security mode и причину блокировок;
- headless verification покрывает hostile issue, comments, repo-local docs и
  runtime outputs.

Проверка:

- review prompt-layer и launcher assets;
- headless agent-flow scenarios без использования host `zellij`.

## Критерий завершения

- `public-safe` режим детерминированно включается для `public` и `unknown`
  visibility;
- auto-intake для public repos не стартует hostile issue вне выбранной policy;
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
