# Issue 21: План имплементации

Статус: draft
Последнее обновление: 2026-03-15

## Назначение

Этот план связывает analysis-решение по issue `#21` с конкретным порядком
изменений в `run`, GitHub adapter, документации и тестах, чтобы preflight
normalization стала явным и проверяемым contract layer, а не набором скрытых
side effects.

## Scope

В план входит:

- явный GitHub-side preflight normalization path для manual `run`;
- assignment policy через current `gh` user только при пустом `assignee`;
- сохранение stage-aware dispatch после preflight;
- обновление SSOT, feature doc, ADR и при необходимости repo-level summary;
- unit и integration coverage для attach/status/assign/failure paths.

## Вне scope

- auto-normalization для `poll`;
- automatic reassign существующего assignee;
- новые public CLI-команды;
- изменение launcher или `zellij` contract beyond preflight ordering;
- host-side `zellij` проверки вне headless path.

## Связанные документы

- Issue:
  - [README.md](./README.md)
- Feature / issue spec:
  - [01-what-we-build.md](./01-what-we-build.md)
  - [02-how-we-build.md](./02-how-we-build.md)
  - [03-how-we-verify.md](./03-how-we-verify.md)
- SSOT:
  - [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
  - [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
- ADR:
  - [../../../docs/adr/0021-cli-contract-poll-run-loop.md](../../../docs/adr/0021-cli-contract-poll-run-loop.md)
  - [../../../docs/adr/0024-stage-aware-run-dispatch.md](../../../docs/adr/0024-stage-aware-run-dispatch.md)
- Verification:
  - [03-how-we-verify.md](./03-how-we-verify.md)
- Code quality:
  - [../../../docs/code-quality.md](../../../docs/code-quality.md)
- Зависимые планы или фичи:
  - [../14/README.md](../14/README.md)

## План изменений документации

- Канонические документы, которые нужно обновить:
  - `docs/issue-analysis-flow.md`
  - `docs/features/0003-agent-launch-orchestration/02-how-we-build.md`
  - `docs/features/0003-agent-launch-orchestration/03-how-we-verify.md`
  - новый ADR про preflight normalization в `run`
- Summary-документы и шаблоны, которые нужно синхронизировать:
  - `README.md`, если после обновления SSOT/feature-doc видимое описание
    operator contract `run` становится неточным
  - `docs/features/0003-agent-launch-orchestration/README.md`, если нужно
    сослаться на новый ADR или уточнить scope feature
- Документы, которые сознательно не меняются, и почему:
  - `docs/issue-implementation-flow.md`, потому что изменение относится к
    analysis entry path, а не к implementation lifecycle
  - `settings.yml` contract, потому что новая конфигурация не требуется

## Зависимости и предпосылки

- в коде уже существует partial path для add-to-project и set-status, который
  нужно сохранить и оформить как explicit preflight;
- `gh api user --jq ".login"` уже используется для `poll.assignee_filter`,
  значит current-user resolution можно переиспользовать;
- owner issue уже зафиксировал MVP policy в комментарии, поэтому отдельный
  discovery stage не нужен;
- verification должна обходиться без host `zellij`, потому что изменение живет
  до launcher stage.

## Порядок работ

### Этап 1. Зафиксировать канонический contract layer

Цель:

- оформить решение в SSOT, feature doc и ADR до изменения кода.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [../../../docs/documentation-process.md](../../../docs/documentation-process.md)

Результат этапа:

- `docs/issue-analysis-flow.md` явно описывает run preflight normalization;
- feature 0003 фиксирует GitHub-side side effects до launcher orchestration;
- создан ADR про run preflight normalization, `gh` identity и no-reassign MVP
  policy;
- summary-layer отмечен для синхронизации там, где это нужно.

Проверка:

- в документах нет противоречий между `run`, `poll`, claim semantics и
  assignee policy;
- ADR позволяет восстановить, почему preflight живет в `run`, а не в `poll`.

### Этап 2. Расширить GitHub adapter для ownership path

Цель:

- сделать app-layer независимым от raw `gh` команд при работе с assignee.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- `RepoIssue` содержит список `assignees`;
- `GhProjectClient` умеет выполнять `assign_issue(...)`;
- current-user resolution используется как явная часть run preflight;
- parsing и error handling покрывают новый GitHub path.

Проверка:

- unit-тесты на parsing repo issue assignees и команду assignment;
- negative tests на ошибки `gh`.

### Этап 3. Собрать явный preflight в app-layer

Цель:

- превратить partial auto-heal behavior в единый детерминированный sequence.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- `run` перед stage dispatch выполняет attach/status/assign normalization в
  явном порядке;
- existing assignee сохраняется без reassign;
- ошибки любого preflight шага прерывают запуск до launcher path;
- diagnostics различают attach, status, assign и no-op cases.

Проверка:

- unit/integration tests на happy path и failure path;
- regression, что `poll` не получает новые side effects.

### Этап 4. Закрыть verification и summary-sync

Цель:

- довести change set до project quality bar и не оставить documentation drift.

Основание:

- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- unit и integration coverage достаточны для новой логики;
- `README.md` и feature overview синхронизированы только если канонические
  документы изменили repo-level summary;
- operator-visible output помогает диагностировать preflight behavior.

Проверка:

- целевой test suite зеленый;
- ручная проверка документации не показывает противоречий между SSOT, feature
  doc, ADR и issue-level plan.

## Критерий завершения

- `run` имеет явный и задокументированный preflight normalization path;
- add-to-project, missing-status и missing-assignee behavior реализованы и
  покрыты тестами;
- existing assignee не перетирается;
- `poll` не меняет behavior;
- SSOT, feature doc, ADR и нужные summary-layers синхронизированы.

## Открытые вопросы и риски

- нужно выбрать конкретную форму assignment call в GitHub adapter так, чтобы
  она была достаточно стабильной и хорошо тестировалась;
- при частичной нормализации из-за ошибки assignment issue может остаться
  прикрепленной к project и со status `Backlog`, но без assignee;
- если repo policy ограничивает assignee set, operator-facing error должен быть
  достаточно явным, чтобы пользователь не воспринял это как скрытый баг flow.

## Журнал изменений

### 2026-03-15

- создан начальный план имплементации для issue `#21`
