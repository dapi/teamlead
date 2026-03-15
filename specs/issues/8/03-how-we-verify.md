# Issue 8: Как проверяем

## Acceptance Criteria

- в проекте есть отдельный release workflow, который запускается по semver tag;
- в проекте есть один public release entrypoint;
- в проекте есть простой и явно задокументированный path для bump
  `major` / `minor` / `patch`;
- explicit-version path, если он поддерживается, ограничен bootstrap/recovery
  сценариями и не подменяет штатный bump contract;
- release workflow публикует GitHub Release с бинарными артефактами и
  checksum-файлами;
- version из `Cargo.toml`, tag `vX.Y.Z`, changelog и release notes обязаны
  совпадать по версии;
- versioning contract соответствует Semantic Versioning 2.0.0, а правила bump
  не противоречат ему;
- Release Notes являются отдельным артефактом, не дублирующим `CHANGELOG.md`;
- Release Notes генерируются локально до publish и затем попадают в GitHub
  Release из versioned файла репозитория;
- release entrypoint завершается success только после успешного publish path в
  GitHub Releases, а не после одного лишь push tag/commit;
- install path через `brew` устанавливает опубликованную release-версию, а не
  development snapshot;
- install path через `curl` устанавливает опубликованную release-версию для
  поддерживаемой платформы;
- `curl` installer выбирает `latest stable` только через GitHub Releases и
  проверяет checksum перед установкой;
- release contract не требует ручной сборки бинарей на машине владельца;
- минимальная release/install документация синхронизирована с реальным publish
  path;
- отсутствие changelog-секции или mismatch версии блокирует публикацию.

## Ready Criteria

- выбран и задокументирован release tooling approach;
- зафиксирован единый public release entrypoint;
- зафиксирован version/tag/changelog contract;
- зафиксирован operator-facing bump contract для `major` / `minor` / `patch`;
- зафиксирован guide по составлению Release Notes и место их хранения;
- зафиксирован минимальный обязательный формат Release Notes;
- подтвержден выбранный минимальный release matrix первой версии;
- определен канонический Homebrew tap contract;
- выбран канонический tap update path: direct commit из CI;
- определен формат `curl` installer path и поддержка latest/explicit version;
- определено, какие документы и summary-слои меняются вместе с release flow.

## Invariants

- source of truth для publishable version остается `Cargo.toml`;
- штатный release path всегда выбирает версию через явный тип
  `major` / `minor` / `patch`, а exact version допустим только в документированном
  bootstrap/recovery режиме;
- publish tag всегда имеет вид `vX.Y.Z` и совпадает с `Cargo.toml`;
- правила versioning соответствуют Semantic Versioning 2.0.0;
- один release запускается одним entrypoint, а не несколькими ручными
  операциями;
- success entrypoint означает завершенный и проверенный publish, а не только
  старт release workflow;
- `brew` и `curl` используют один и тот же published asset contract;
- changelog является обязательным release input, а не post-factum заметкой;
- Release Notes хранятся отдельно от changelog и публикуются в GitHub Release
  из versioned файла;
- Release Notes имеют минимальный обязательный format contract и проходят
  human-readable review по нему;
- обычный PR CI и release CI остаются разными pipeline;
- publish automation не требует ручной сборки бинарей или ручного publish на
  host-машине, даже если локальный entrypoint выполняет preflight и orchestration;
- release assets и checksums детерминированы по версии и platform target.

## Test Plan

### Unit tests

- проверка парсинга и сравнения версии между `Cargo.toml` и semver tag;
- проверка CLI/script contract единого release entrypoint;
- проверка bump logic для `patch`, `minor` и `major`, если она оформляется в
  script/tooling;
- проверка guardrails для `--version <X.Y.Z>`:
  bootstrap первого релиза разрешен, обычный post-bootstrap release отклоняется;
- проверка, что invalid SemVer value или несоответствующий bump contract
  отклоняется до publish;
- проверка, что локальная генерация Release Notes использует нужную секцию
  `CHANGELOG.md` как вход;
- проверка генерации `docs/releases/vX.Y.Z.md` и соответствия guide/template;
- проверка asset naming и platform mapping для installer path;
- проверка выбора latest vs explicit version для `curl` installer logic, если
  эта логика реализуется в versioned script/tooling.
- проверка, что `curl` installer валидирует checksum и не продолжает install при
  отсутствии checksum-файла.

### Integration tests

- dry-run release pipeline собирает ожидаемый набор assets без публикации;
- workflow/fake-release path валидирует mismatch:
  - tag != `Cargo.toml`;
  - отсутствует changelog-секция;
  - отсутствует checksum;
- bump path корректно обновляет версию и связанные release metadata для
  сценариев `patch`, `minor`, `major`;
- bootstrap path первого публичного релиза корректно публикует стартовую
  version без обязательного backfill прошлых версий;
- локальный release path создает versioned файл Release Notes до push/tag;
- generated Homebrew formula ссылается на asset и checksum нужной версии;
- tap update path делает direct commit в `dapi/homebrew-ai-teamlead`, а не
  оставляет незавершенный PR-step;
- release workflow читает Release Notes из versioned файла и публикует их в
  GitHub Release без повторной облачной генерации;
- release entrypoint не завершает работу green status раньше, чем release
  workflow закончит publish и GitHub Release станет наблюдаемым;
- partial-failure path ведет себя fail-closed:
  - до push допускается безопасный rerun той же версии;
  - после появления неполного GitHub Release automation не делает silent
    overwrite и требует явного operator recovery;
- `curl` installer берет latest stable только из GitHub Releases и скачивает
  checksum из того же release;
- `curl` installer smoke path скачивает корректный asset для Linux/macOS test
  target и раскладывает бинарь в ожидаемое место;
- повторный запуск release job для той же версии не приводит к silent
  расхождению артефактов.

### Smoke tests

- контролируемый выпуск первой тестовой версии в GitHub Releases;
- ручная проверка `brew install` из published formula/tap;
- ручная проверка `curl ... | sh` или эквивалентного documented install path на
  чистом окружении;
- ручная проверка, что tap `dapi/homebrew-ai-teamlead` обновился на правильный
  URL и checksum;
- проверка, что опубликованный бинарь печатает ожидаемую версию.

## Happy Path

1. Разработчик запускает единый release entrypoint.
2. Entry point выбирает `patch`, `minor` или `major` bump и получает
   согласованное обновление version/release metadata.
3. Entry point создает или обновляет секцию версии в `CHANGELOG.md`.
4. Локально генерируется `docs/releases/vX.Y.Z.md`.
5. Создается semver tag `vX.Y.Z`.
6. Release workflow валидирует совпадение tag, package version, changelog и
   Release Notes.
7. CI собирает matrix бинарей, публикует checksums и создает GitHub Release.
8. GitHub Release использует body из `docs/releases/vX.Y.Z.md`.
9. Entry point дожидается green publish и подтверждает наличие опубликованного
   релиза.
10. Homebrew formula и `curl` installer начинают указывать на новые assets.
11. Пользователь устанавливает новую версию через `brew` или `curl`.

## Edge Cases

- tag создан, но `Cargo.toml` не обновлен;
- выбран `patch` bump, но tooling пытается изменить `minor` или `major`;
- version string перестает быть валидной по Semantic Versioning 2.0.0;
- версия есть в `Cargo.toml`, но отсутствует в `CHANGELOG.md`;
- changelog обновлен, но Release Notes файл не создан;
- Release Notes просто копируют changelog без user-facing summary и структуры;
- release workflow уже публиковал часть артефактов и упал на tap update;
- exact-version release запускается после того, как публичная release-history
  уже существует;
- GitHub Release существует, но checksum-файл для `curl` installer отсутствует;
- release существует, но installer path не находит asset для конкретной
  платформы;
- пользователь хочет установить не latest, а конкретную версию.

## Failure Scenarios

- GitHub Release создан без полного набора assets;
- operator-facing bump path обновил только `Cargo.toml`, но не синхронизировал
  changelog/release metadata;
- локальная генерация Release Notes отработала, но файл не попал в release
  commit/tag;
- Homebrew formula обновилась на неверный checksum;
- release workflow обновил GitHub Release, но не смог синхронизировать tap
  `dapi/homebrew-ai-teamlead`;
- `curl` installer скачал asset не той архитектуры;
- `curl` installer не смог подтвердить checksum latest stable release;
- release notes и `CHANGELOG.md` указывают на разные версии;
- повторный publish той же версии silently заменил артефакт вместо явной ошибки.

## Observability

Нужны диагностические сигналы минимум по следующим точкам:

- какая версия публикуется и каким tag она была вызвана;
- какой тип bump был выбран: `major`, `minor` или `patch`;
- какой local script/entrypoint сгенерировал Release Notes и где лежит файл;
- какая changelog section использовалась как вход для генерации Release Notes;
- по какому release/tag installer определил `latest stable`;
- какой release matrix реально собран;
- какие asset names и checksums опубликованы;
- какой URL formula/tap и какой installer endpoint относятся к этой версии;
- какой commit/PR в `dapi/homebrew-ai-teamlead` соответствует текущему релизу;
- каким observed signal entrypoint подтвердил успешный publish;
- на каком шаге release flow остановился: build, publish, tap update,
  installer validation или changelog gate.

## Verification Checklist

- release workflow отделен от обычного CI и задокументирован;
- public release entrypoint задокументирован и проверяем;
- bump contract для `major` / `minor` / `patch` задокументирован и проверяем;
- version/tag/changelog contract проверяется автоматически;
- Semantic Versioning 2.0.0 соблюдается и не подменен локальными правилами;
- Release Notes не путаются с changelog, имеют guide и versioned storage;
- формат Release Notes проверяется на обязательные секции и не деградирует в
  копию `CHANGELOG.md`;
- semantics успеха entrypoint задокументирована и проверяема;
- GitHub Release содержит ожидаемые assets и checksums;
- `brew` path проверен на актуальную версию и checksum;
- tap contract `dapi/homebrew-ai-teamlead` и его auth/update path задокументированы;
- `curl` path проверен для поддерживаемых платформ;
- документация не обещает install-команд, которых нет в реальном publish path;
- первая release-версия проходит controlled smoke validation;
- partial failure не оставляет систему без явной диагностики.
