# Issue 8: Что строим

## Problem

Сейчас `ai-teamlead` можно собрать и запустить только как development-инструмент
из репозитория.

Пробелы текущего состояния:

- нет канонического release flow, который публикует готовые бинарные артефакты;
- version в `Cargo.toml` не связана с semver tag и release lifecycle;
- пользователь не имеет стабильного install path через `brew` или `curl`;
- changelog не является обязательным и проверяемым release-входом;
- user-facing release notes и GitHub Release могут легко разойтись с реальным
  содержимым версии.

В результате проект остается пригодным для dogfooding, но не для нормального
публичного распространения и повторяемой установки.

## Who Is It For

- владелец репозитория, которому нужен предсказуемый способ выпускать версии;
- пользователь CLI, который хочет установить `ai-teamlead` через `brew` или
  одной `curl`-командой без локальной сборки;
- сопровождающий проекта, которому нужен единый контракт версионирования,
  changelog и release assets;
- будущая user-facing документация из issue `#9`, которая должна опираться на
  реальный install contract, а не на временные команды из development-режима.

## Outcome

Нужен минимальный публичный release contract, в котором:

- у владельца есть одна понятная точка входа для запуска релиза;
- каждая публикуемая версия оформляется как semver release `vX.Y.Z`;
- существует простой и понятный способ поднять следующую
  `major` / `minor` / `patch` версию без ручного редактирования нескольких
  несвязанных файлов;
- versioning contract полностью соответствует Semantic Versioning 2.0.0:
  https://semver.org/;
- `Cargo.toml`, Git tag, changelog и GitHub Release не противоречат друг другу;
- Release Notes существуют как отдельный per-release артефакт и не подменяются
  `CHANGELOG.md`;
- CI собирает и публикует бинарные артефакты для поддерживаемых платформ;
- install path через `brew` и `curl` использует те же опубликованные артефакты;
- release не требует ручной сборки, ручного пересчета checksums и ручного
  составления release package;
- changelog становится обязательной частью подготовки версии.

## Scope

В текущую задачу входит:

- единый operator-facing entrypoint релиза;
- tag-driven release flow в GitHub Actions;
- канонический versioning contract для `ai-teamlead`;
- operator-facing механизм bump версии по типу изменения:
  `major`, `minor`, `patch`;
- ограниченный explicit-version path только для bootstrap первого публичного
  релиза и controlled recovery до завершенного publish;
- contract для отдельных Release Notes;
- локальная генерация Release Notes скриптами без требования внешнего LLM;
- публикация GitHub Release с бинарями и checksum-артефактами;
- install path через `brew`;
- install path через `curl`;
- changelog contract и связка changelog с release notes;
- bootstrap-контракт первого публичного semver-релиза;
- retry/failure contract для partial publish;
- минимальные user-facing install-инструкции, достаточные для release-пакета;
- verification contract для dry-run, smoke и реального release path.

## Non-Goals

В текущую задачу не входит:

- публикация в `crates.io`;
- поддержка `apt`, `yum`, `nix`, `winget` и других package manager;
- автоматический deploy или отдельный post-release operation flow;
- redesign полного user-facing `README.md` сверх минимально нужных install и
  release summary;
- code signing, notarization и другие advanced supply-chain меры первой версии,
  если они не требуются для базового install contract;
- несколько release channels (`stable`, `nightly`, `beta`) в первой версии.

## Constraints And Assumptions

- `ai-teamlead` остается Rust CLI, поэтому source of truth для продуктовой
  версии должен быть привязан к Rust package metadata, а не к отдельному
  произвольному файлу;
- оператор должен запускать release через один public entrypoint, а не через
  набор неявных команд `git`, `gh`, редактора и ручных upload-действий;
- штатный release path должен быть выражен через явный выбор `major` / `minor`
  / `patch`, чтобы версия поднималась предсказуемо и без ручной синхронизации;
- explicit version допустим только как отдельный исключительный режим:
  bootstrap первого публичного релиза или controlled recovery до появления
  завершенного GitHub Release этой версии;
- contract версии должен соблюдать Semantic Versioning 2.0.0 полностью, а не
  использовать `major` / `minor` / `patch` только как нестрогие ярлыки;
- `CHANGELOG.md` и Release Notes должны быть разными сущностями:
  changelog хранит кумулятивную историю версий, а Release Notes описывают
  конкретный релиз в user-facing форме;
- Release Notes должны уметь генерироваться локально скриптами; использование
  локального LLM допустимо только как внутренний помощник этого локального шага;
- до публикации Release Notes должны сохраняться как versioned файл в
  репозитории, чтобы CI публиковал в GitHub Release уже проверенный текст, а не
  генерировал его заново в облаке.
- release flow должен запускаться в CI и быть воспроизводимым без ручной сборки
  на машине владельца;
- локальный operator entrypoint допустим как preflight/control layer, но
  успешный publish не должен зависеть от ручной сборки бинарей или ручного
  пересчета checksum на host-машине;
- install paths через `brew` и `curl` должны потреблять один и тот же
  опубликованный набор release assets;
- changelog должен быть version-aware и пригодным как для репозитория, так и
  как структурированный вход для локальной генерации Release Notes, но не как
  прямой publish body GitHub Release;
- success единого release entrypoint должен означать не просто успешный local
  handoff в CI, а появление проверенного релиза в GitHub Releases с ожидаемыми
  assets и checksums;
- для первой версии Homebrew install contract фиксируется через отдельный tap
  `dapi/homebrew-ai-teamlead`, который получает formula-обновления только из
  release automation;
- первый публичный релиз начинает публичную semver-history проекта; backfill
  предыдущих не-публичных версий в GitHub Releases и `CHANGELOG.md` не является
  обязательным;
- public install/distribution path должен учитывать security baseline из
  `untrusted-input-security` и feature `0006-public-repo-security`, особенно для
  `curl` bootstrap path и внешних download endpoints;
- full user-facing onboarding остается отдельной задачей `#9`, поэтому в этой
  задаче достаточно release-oriented install contract и минимальной документации;
- текущий проект уже использует GitHub Actions и GitHub Releases как допустимый
  operational baseline, поэтому новый flow должен ложиться на существующую
  GitHub-first модель.

## User Story

Как владелец `ai-teamlead`, я хочу выпускать версию по явному semver tag, чтобы
я запускал один понятный entrypoint релиза, который сам поднимет нужную
`major` / `minor` / `patch` версию, обновит changelog, локально подготовит
отдельные Release Notes, запустит проверки и доведет публикацию до GitHub
Releases, чтобы на выходе я получал проверенную собранную версию с правильными
assets, changelog и Release Notes без ручной координации нескольких шагов.

## Use Cases

1. Разработчик запускает единый release entrypoint, выбирает `major`, `minor`
   или `patch` и получает согласованное обновление `Cargo.toml`,
   `CHANGELOG.md`, Release Notes, tag `vX.Y.Z` и дальнейшую публикацию в
   GitHub Release с артефактами и checksums.
2. Пользователь на macOS или Linux выполняет install через `curl` и получает
   бинарь именно той версии, которая опубликована в GitHub Release.
3. Пользователь выполняет `brew install ...` и получает ту же версию по
   стабильному formula/tap contract.
4. Сопровождающий выбирает `patch`, `minor` или `major` bump по характеру
   изменений и получает предсказуемое обновление version/changelog/release
   metadata без ручной синхронизации нескольких мест.
5. Сопровождающий запускает один release entrypoint, а дальше flow сам
   выполняет локальную подготовку Release Notes, проверки, tag/push, дожидается
   завершения release workflow и завершает работу только после появления
   опубликованного релиза.
6. Поддерживающий релиз проверяет changelog и release notes по конкретной
   версии без ручного сравнения между tag, binary assets и историей коммитов.
7. Владелец проекта выполняет первый публичный релиз без backfill прошлых
   не-публичных версий: выбирает стартовую publishable version, получает для нее
   первый `vX.Y.Z` tag, `CHANGELOG.md` section и versioned Release Notes.
8. После частичного сбоя publish flow повторный запуск либо безопасно
   продолжает публикацию той же версии, либо останавливается на явном
   операторском блокере без silent overwrite артефактов.

## Dependencies

- [../../../README.md](../../../README.md) задает текущую repo-level картину
  проекта и подтверждает, что release/user docs остаются отдельным кластером
  roadmap;
- [../../../ROADMAP.md](../../../ROADMAP.md) фиксирует issue `#8` как часть
  кластера `Release и user docs`, а issue `#9` как зависимую user-facing
  документацию;
- [../../../docs/features/0001-ai-teamlead-cli/README.md](../../../docs/features/0001-ai-teamlead-cli/README.md)
  задает базовый CLI-контракт и продуктовую рамку распространяемого бинаря;
- [../../../docs/features/0004-issue-implementation-flow/README.md](../../../docs/features/0004-issue-implementation-flow/README.md)
  и [../51/README.md](../51/README.md) явно оставляют release/deploy вне scope
  coding lifecycle, поэтому release contract нужно оформлять отдельно;
- [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md)
  и
  [../../../docs/features/0006-public-repo-security/README.md](../../../docs/features/0006-public-repo-security/README.md)
  задают security baseline для public distribution и внешних install endpoints;
- [../../../ROADMAP.md](../../../ROADMAP.md) отдельно фиксирует, что security
  review issue `#56` влияет на release/onboarding/public repo adoption и должен
  учитываться как cross-cutting риск;
- [../../../docs/adr/0011-use-zellij-main-release-in-ci.md](../../../docs/adr/0011-use-zellij-main-release-in-ci.md)
  подтверждает, что проект уже использует GitHub Release как допустимый способ
  доставки бинарей в CI.
