# Issue 8: План имплементации

Статус: draft
Последнее обновление: 2026-03-15
Статус согласования: pending human review

## Назначение

Этот план связывает analysis-решения по issue `#8` с конкретным порядком
реализации release/distribution contract для `ai-teamlead`, чтобы versioning,
CI publish, changelog и install channels развивались как один согласованный
pipeline, а не как набор разрозненных правок.

## Scope

В план входит:

- единый public release entrypoint;
- version/tag/changelog contract;
- простой operator-facing bump path для `major` / `minor` / `patch`;
- локальная генерация отдельных Release Notes;
- release workflow в GitHub Actions;
- GitHub Release assets и checksums;
- install path через `brew` и `curl`;
- минимальная release-oriented документация и verification.

## Вне scope

- публикация в дополнительные package manager;
- advanced signing/notarization первой версии;
- полный redesign user-facing README;
- отдельный deploy или post-release operation flow.

## Связанные документы

- Issue: https://github.com/dapi/ai-teamlead/issues/8
- Feature / issue spec:
  - [README.md](./README.md)
  - [01-what-we-build.md](./01-what-we-build.md)
  - [02-how-we-build.md](./02-how-we-build.md)
  - [03-how-we-verify.md](./03-how-we-verify.md)
- SSOT:
  - [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
  - [../../../docs/features/0001-ai-teamlead-cli/README.md](../../../docs/features/0001-ai-teamlead-cli/README.md)
  - [../../../docs/features/0002-repo-init/README.md](../../../docs/features/0002-repo-init/README.md)
  - [../../../docs/features/0004-issue-implementation-flow/README.md](../../../docs/features/0004-issue-implementation-flow/README.md)
- ADR:
  - [../../../docs/adr/0011-use-zellij-main-release-in-ci.md](../../../docs/adr/0011-use-zellij-main-release-in-ci.md)
- Verification:
  - [03-how-we-verify.md](./03-how-we-verify.md)
- Code quality:
  - [../../../docs/code-quality.md](../../../docs/code-quality.md)
- Зависимые планы или фичи:
  - [../51/README.md](../51/README.md)

## План изменений документации

- Канонические документы, которые нужно обновить:
  - новый ADR по release/distribution contract;
  - guide по Release Notes и, возможно, template для них;
  - новый feature-doc слой `docs/features/0008-release-distribution/` как
    постоянный дом для release/distribution knowledge;
  - новый SSOT `docs/release-flow.md` как канонический operational contract для
    release lifecycle, retry и publish semantics;
  - `README.md` как summary уровня репозитория, если release path становится
    частью верхнеуровневого позиционирования.
- Summary-документы и шаблоны, которые нужно синхронизировать:
  - минимальные install snippets и release notes guidance;
  - при необходимости `templates/init/README.md` или соседние bootstrap docs,
    если release/install path нужно упомянуть как supported distribution model.
- Документы, которые сознательно не меняются, и почему:
  - flow-документы analysis/implementation не должны поглощать release contract,
    потому что release остается отдельным operational слоем;
  - полный user-facing README переносится в scope issue `#9`.

## Зависимости и предпосылки

- текущий development CI уже стабилен и не должен смешиваться с publish path;
- выбранный release tool должен уметь генерировать или поддерживать тот же
  asset contract для GitHub Release, Homebrew и shell installer;
- выбранный release/bump tool должен поддерживать полный SemVer 2.0.0 contract
  и не вводить собственную упрощенную трактовку `major` / `minor` / `patch`;
- локальный release path должен уметь генерировать Release Notes без
  обязательной зависимости на внешний облачный LLM;
- нужен доступ к Homebrew tap update path и способ аутентификации для него;
- канонический tap первой версии фиксируется как `dapi/homebrew-ai-teamlead`,
  а CI должен получить secret `HOMEBREW_TAP_GITHUB_TOKEN`;
- первая publishable версия должна быть явно выбрана до запуска реального
  release workflow.
- security baseline из issue `#56` и
  [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md)
  должен быть учтен для public install/download path.

## Порядок работ

### Этап 1. Зафиксировать release contract в документации и ADR

Цель:

- определить канонический version/tag/changelog/install contract до изменения
  CI и publish tooling.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [../../../docs/documentation-process.md](../../../docs/documentation-process.md)

Результат этапа:

- создан новый ADR по release/distribution contract;
- зафиксированы:
  - выбранный release tooling approach `cargo-dist`;
  - единый public release entrypoint;
  - semver tag contract;
  - штатный bump contract для `major` / `minor` / `patch`;
  - guardrails для exact-version bootstrap/recovery path;
  - source of truth для версии;
  - changelog gate;
  - guide и storage contract для Release Notes;
  - минимальный формат Release Notes;
  - стратегия `brew` и `curl`;
  - tap contract `dapi/homebrew-ai-teamlead`;
  - direct-commit path обновления tap;
  - integrity contract для `curl` installer;
  - partial-failure/retry contract;
  - выбранный минимальный release matrix первой версии.

Проверка:

- doc review на непротиворечивость между issue spec, ADR и repo-level summary;
- отсутствуют плавающие места, где версия или release source of truth неясны.

### Этап 2. Ввести versioning и changelog gates

Цель:

- сделать release-подготовку машино-проверяемой до публикации артефактов.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- добавлен `CHANGELOG.md`;
- добавлен или выбран единый bump entrypoint для `major` / `minor` / `patch`;
- exact-version path ограничен bootstrap/recovery правилами и автоматически
  отклоняется вне этих сценариев;
- добавлен guide по Release Notes;
- guide по Release Notes фиксирует обязательные секции и отличие от
  `CHANGELOG.md`;
- release entrypoint генерирует `docs/releases/vX.Y.Z.md` локально до tag/push;
- tooling валидирует совпадение:
  - `Cargo.toml version`;
  - `vX.Y.Z` tag;
  - changelog section;
  - `docs/releases/vX.Y.Z.md` как publish body для GitHub Release;
- ошибка на mismatch проявляется до publish step.

Проверка:

- unit/integration тесты на mismatch version и отсутствие changelog section;
- тесты на корректный bump `patch`, `minor`, `major` и соблюдение SemVer 2.0.0;
- тесты на генерацию, хранение и публикацию Release Notes;
- dry-run release path не проходит при нарушении контракта.

### Этап 3. Реализовать release workflow и publish assets

Цель:

- перевести сборку публичных артефактов в повторяемый tag-driven CI path.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- появился отдельный release workflow;
- публикуются binary assets и checksums для agreed matrix;
- publish path не зависит от ручной локальной сборки;
- release logs и diagnostics позволяют восстановить, что именно было
  опубликовано.

Проверка:

- dry-run release;
- контролируемый smoke выпуск тестовой версии;
- сверка asset names, checksums и release notes;
- проверка bootstrap первой publishable версии без backfill старых релизов.

### Этап 4. Подключить install channels `brew` и `curl`

Цель:

- сделать опубликованные assets доступными через стабильные install commands.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- Homebrew formula обновляется автоматически из release pipeline;
- обновление formula выполняется в tap `dapi/homebrew-ai-teamlead` через
  `HOMEBREW_TAP_GITHUB_TOKEN` и direct commit в default branch;
- `curl` installer умеет ставить latest stable и explicit version;
- оба канала используют опубликованные release assets и checksums.

Проверка:

- smoke path для `brew install`;
- smoke path на синхронность formula URL/checksum с опубликованным release;
- smoke path для `curl` installer на чистом окружении;
- smoke path на обязательную checksum-валидацию `curl` installer;
- проверка установленной бинарной версии после установки.

### Этап 5. Синхронизировать минимальную release-документацию

Цель:

- сделать release/install contract discoverable без ожидания отдельной задачи на
  полный user-facing README.

Основание:

- [01-what-we-build.md](./01-what-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)

Результат этапа:

- в repo-level docs есть минимально достаточные install/release instructions;
- changelog и release notes usage описаны одинаково;
- issue `#9` получает устойчивую базу для полного user-facing README.

Проверка:

- doc review на соответствие реальному publish path;
- install snippets воспроизводимы на опубликованной версии.

## Критерий завершения

- release flow документирован, реализован и проходит dry-run/smoke проверки;
- весь релиз запускается одним public entrypoint;
- bump `major` / `minor` / `patch` выполняется одним понятным entrypoint и не
  требует ручной синхронизации version metadata;
- exact-version path остается только bootstrap/recovery исключением и не
  размывает основной SemVer bump contract;
- `Cargo.toml`, semver tag, changelog и GitHub Release связаны единым
  контрактом;
- Semantic Versioning 2.0.0 соблюдается полностью;
- Release Notes локально генерируются, сохраняются в repo и попадают в GitHub
  Release как отдельный от changelog артефакт;
- `brew` и `curl` ставят опубликованный бинарь, а не development snapshot;
- tap `dapi/homebrew-ai-teamlead` обновляется из CI по каноническому auth-path;
- `curl` installer использует GitHub Releases как единственный trust path для
  latest stable и не ставит бинарь без checksum verification;
- release automation не требует ручной сборки артефактов;
- минимальная release/install документация синхронизирована с реальным
  pipeline;
- partial failure приводит к явному retry/recovery решению, а не к silent
  overwrite release artifacts.

## Открытые вопросы и риски

- первый публичный release может потребовать отдельного controlled rollout,
  если текущая version history в git еще не отражена в `CHANGELOG.md`;
- если выбранный release tool окажется слишком opinionated для нужного tap или
  installer contract, потребуется fallback без размывания agreed release model.

## Журнал изменений

### 2026-03-15

- создан начальный план имплементации для issue `#8`
