# Issue 8: Как строим

## Approach

Изменение нужно оформлять как отдельный release/distribution contract поверх
существующего development CI, а не как набор ad-hoc shell-шагов для публикации
версии.

Базовый технический подход:

- ввести один public entrypoint релиза, например
  `./scripts/release.sh --bump <major|minor|patch>` или
  `./scripts/release.sh --version <X.Y.Z>`;
- добавить отдельный tag-driven release workflow в GitHub Actions;
- зафиксировать единый versioning contract:
  `Cargo.toml version` -> `git tag vX.Y.Z` -> `CHANGELOG.md` -> `GitHub Release`;
- добавить простой operator-facing bump path, который явно принимает один из
  режимов `major`, `minor`, `patch` и обновляет release metadata согласованно;
- отделить `CHANGELOG.md` от Release Notes:
  changelog остается cumulative history, а Release Notes становятся отдельным
  versioned документом релиза;
- генерировать draft Release Notes локально скриптами до tag/push;
- публиковать release assets и checksum-файлы в GitHub Releases;
- использовать один packaging layer для генерации release artifacts,
  Homebrew formula и `curl` installer path;
- держать install channels (`brew` и `curl`) производными от тех же published
  assets, а не отдельными независимыми сборками;
- оставить текущий `ci.yml` как validation path для PR и main, а release path
  вынести в отдельный workflow;
- синхронизировать minimal release docs и install snippets без попытки в этой же
  задаче полностью переписать user-facing README.

Предпочтительный путь первой версии:

- использовать `cargo-dist` или эквивалентный declarative release tool для Rust
  CLI, потому что issue требует связать release assets, checksums, Homebrew,
  shell installer и changelog в один повторяемый pipeline;
- если implementation выявит жесткий blocker у конкретного toolchain, допустим
  fallback на hand-written GitHub Actions workflow, но без изменения целевого
  контракта.

## Affected Areas

- `.github/workflows/`
  потребуется новый release workflow и, возможно, небольшая синхронизация
  текущего `ci.yml`;
- `Cargo.toml`
  станет явным source of truth для publishable version и release metadata;
- `CHANGELOG.md`
  нужно добавить как versioned документ release history;
- `docs/releases/`
  потребуется как versioned storage для отдельных Release Notes по версиям,
  например `docs/releases/vX.Y.Z.md`;
- `docs/release-notes.md` или соседний guide-документ
  потребуется как канонический стандарт составления Release Notes;
- release bump tooling
  потребуется для простого обновления `major` / `minor` / `patch` версии и
  синхронизации changelog/release metadata;
- release tooling config
  потребуется для matrix, asset naming, installer generation и Homebrew output;
- scripts или generated installer artifacts
  нужны для `curl` install path;
- release docs
  должны получить минимальные install/release instructions;
- возможно, отдельный ADR
  потребуется для выбора release tool и канонического versioning contract.

## Interfaces And Data

### Source of truth для версии

Минимальный безопасный contract:

- source of truth для версии остается `Cargo.toml`;
- public release entrypoint один:
  оператор не должен вручную собирать release из нескольких команд;
- bump версии допускается только как осознанный выбор одного из трех типов:
  `major`, `minor`, `patch`;
- bump path должен быть оформлен одной понятной командой или script entrypoint,
  а не требовать ручной правки нескольких файлов в произвольном порядке;
- правила bump обязаны соответствовать Semantic Versioning 2.0.0
  (`https://semver.org/`);
- публикуемый Git tag должен иметь вид `vX.Y.Z`;
- `X.Y.Z` из tag обязан совпадать с `package.version`;
- в `CHANGELOG.md` обязана существовать секция для этой версии;
- для версии обязан существовать отдельный файл Release Notes;
- GitHub Release создается только для версии, прошедшей эти проверки.

Это устраняет drift между кодом, release notes и install channels.

Минимальная семантика bump:

- `patch` используется только для backward-compatible bug fixes;
- `minor` используется для backward-compatible additions;
- `major` используется для backward-incompatible changes;
- если реализация поддержит prerelease/build metadata, они тоже обязаны
  соответствовать Semantic Versioning 2.0.0, а не вводить локальный dialect.

### Release assets

Минимальный publish contract:

- GitHub Release публикует бинарные артефакты для поддерживаемого matrix;
- рядом публикуются checksum-файлы;
- asset naming детерминирован и пригоден для installer automation;
- install channels `brew` и `curl` используют именно эти assets, а не скрытую
  альтернативную сборку.

Предлагаемый matrix первой версии:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

Windows и дополнительные packaging targets остаются вне первой версии.

### Changelog и release notes

Нужен один version-aware changelog contract:

- в репозитории появляется `CHANGELOG.md`;
- каждая publishable версия получает отдельную секцию;
- release workflow валидирует наличие версии в changelog;
- changelog section является входом для локальной генерации Release Notes, но
  не прямым publish body для GitHub Release.

Нужен отдельный contract для Release Notes:

- Release Notes не равны `CHANGELOG.md` и не являются его дословной копией;
- Release Notes хранятся как отдельный versioned файл:
  `docs/releases/vX.Y.Z.md`;
- локальный release entrypoint до создания tag генерирует draft Release Notes
  скриптом из шаблона, changelog section и release context;
- если локально доступен LLM, он может использоваться только внутри локального
  script path и не должен быть обязательным единственным способом генерации;
- после локальной генерации Release Notes файл коммитится вместе с версией и
  changelog;
- release workflow в GitHub Actions читает именно этот versioned файл и
  публикует его body в GitHub Release.

### Release entrypoint

Главный operator-facing contract первой версии:

- один entrypoint `./scripts/release.sh`;
- оператор передает либо `--bump <major|minor|patch>`, либо
  `--version <X.Y.Z>`;
- script:
  - вычисляет target version;
  - обновляет `Cargo.toml`;
  - создает или обновляет секцию версии в `CHANGELOG.md`;
  - генерирует `docs/releases/vX.Y.Z.md`;
  - запускает локальные проверки;
  - создает release commit и tag;
  - пушит commit и tag;
  - запускает или ожидает соответствующий release workflow в GitHub Actions;
  - опрашивает статус публикации;
  - завершается success только если GitHub Release создан и содержит ожидаемые
    assets, checksums и body из `docs/releases/vX.Y.Z.md`.

Это дает одну точку входа при сохранении воспроизводимой CI-публикации.

### Install path через `brew`

Homebrew path должен быть частью release automation, а не ручной правкой
formula после публикации релиза.

Минимальный contract:

- publish workflow обновляет Homebrew formula в выделенном tap contract;
- formula ссылается на published release asset и checksum этой же версии;
- `brew install` и `brew upgrade` приходят к тому же version/tag contract, что
  и GitHub Release.

### Install path через `curl`

`curl` installer должен оставаться thin bootstrap layer:

- скрипт определяет платформу пользователя;
- скачивает соответствующий published asset нужной версии;
- поддерживает install latest stable и explicit version;
- не компилирует проект из исходников на машине пользователя по умолчанию.

## Configuration And Runtime Assumptions

- release workflow запускается по semver tag и при необходимости вручную через
  `workflow_dispatch` для dry-run/diagnostics;
- для публикации GitHub Release достаточно GitHub Actions runtime и стандартного
  токена, кроме части с push в Homebrew tap, где может понадобиться отдельный
  token/secret;
- release tooling config должна быть versioned и жить в репозитории;
- operator-facing bump path должен быть достаточно простым, чтобы владелец
  репозитория мог безопасно поднять версию без ручного знания внутренней
  раскладки release metadata;
- release entrypoint должен локально уметь создавать Release Notes даже без
  внешнего облачного LLM доступа;
- `curl` installer должен быть POSIX-shell friendly и не тянуть repo-local
  runtime state;
- первый публичный release обязан явно выбрать стартовую publishable версию,
  даже если `Cargo.toml` уже содержит development version `0.1.0`.

## External Interfaces

- GitHub Actions
  исполняет release pipeline;
- GitHub Releases
  хранит published binaries, checksums и release notes;
- Homebrew tap
  получает formula-обновления под опубликованные версии;
- `curl`
  используется только как transport для bootstrap installer path;
- Cargo / Rust toolchain
  остается build source для release artifacts.

## Architecture Notes

### Release flow отделен от обычного CI

Текущий `ci.yml` проверяет код, но не является release lifecycle.

Поэтому нужно разделить:

- `validation CI` для PR и branch pushes;
- `release CI` для semver tags и публикации.

Это упрощает диагностику и делает publish-событие явно наблюдаемым.

### Один packaging layer для всех install channels

Если `brew` и `curl` будут собирать бинарь независимо друг от друга, проект
быстро получит расхождение:

- разные asset names;
- разные checksums;
- разные правила выбора платформы;
- разные источники release truth.

Поэтому нужен единый publish layer, от которого зависят оба install path.

### Changelog как обязательный gate

Issue прямо включает changelog, значит он не должен остаться best-effort
документом.

Release без changelog-секции считается недоготовленным и должен блокироваться
до публикации.

## ADR Impact

По правилам
[../../../docs/documentation-process.md](../../../docs/documentation-process.md)
изменение затрагивает:

- публичный install contract;
- versioning/source-of-truth contract;
- operator-facing contract инкремента `major` / `minor` / `patch`;
- contract единого release entrypoint;
- contract хранения и генерации Release Notes;
- release automation path;
- интеграцию с внешними distribution channels.

Поэтому нужен как минимум один новый ADR, который зафиксирует:

- выбранный release tooling approach;
- канонический version/tag/changelog contract;
- operator-facing способ bump версии и его связь с SemVer 2.0.0;
- format, guide и lifecycle Release Notes;
- стратегию публикации Homebrew formula и `curl` installer;
- минимальный поддерживаемый release matrix.

## Alternatives Considered

1. Полностью ручной release через локальные shell-команды и GitHub UI.

   Отклонено: слишком высокий риск drift между version, changelog, assets и
   install-инструкциями.

2. Только GitHub Release без `brew` и `curl`.

   Отклонено: это не закрывает исходный scope issue.

3. Публикация только исходников или `cargo install --git`.

   Отклонено: это не дает user-facing бинарный install path и делает установку
   зависимой от локального toolchain.

## Migration Or Rollout Notes

- первый rollout нужно делать как controlled release с ручной human-проверкой
  published assets, install channels и качества сгенерированных Release Notes;
- guide по Release Notes и versioned шаблон нужно подготовить до первого
  публичного релиза, иначе operator-facing entrypoint останется недоопределен;
- `README.md` должен получить только минимальный release/install summary, а
  полный user-facing onboarding остается задачей `#9`;
- если release tooling генерирует формульные или installer-файлы, нужно
  решить, какие из них versioned в repo, а какие публикуются только как
  generated artifacts;
- текущая версия в `Cargo.toml` не должна автоматически считаться уже
  опубликованной: факт релиза возникает только после успешного tag-driven
  publish.

## Risks

- drift между `Cargo.toml`, tag и changelog приведет к ложным или битым релизам;
- если bump path останется слишком ручным, владелец репозитория начнет
  обходить release contract и drift вернется уже на уровне версий;
- если Release Notes не будут отдельным versioned артефактом, GitHub Release
  быстро начнет расходиться с локально проверенным содержимым;
- Homebrew tap может потребовать отдельную auth-механику и аккуратный contract
  обновления formula;
- `curl` installer легко сделать хрупким по shell portability и platform
  detection;
- неудачный partial publish может оставить GitHub Release, formula и docs в
  разных состояниях, если rollback/diagnostics не будут описаны явно;
- отсутствие проверки asset naming и checksums сломает install channels без
  явной ошибки на этапе сборки.
