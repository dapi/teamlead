# Issue 33: План имплементации

Статус: approved
Последнее обновление: 2026-03-14
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-14T23:06:53+03:00

## Назначение

Этот план задает порядок реализации zero-config контракта для
`./.ai-teamlead/settings.yml`, где bootstrap шаблон документирует настройки,
runtime defaults живут в canonical Rust-layer, а эволюция схемы защищена
guardrail-тестами.

## Scope

В scope входит:

- классификация schema keys на required/defaulted/example-only;
- canonical default-layer для runtime-конфига;
- новый loader path для missing defaulted-полей;
- обновление `templates/init/settings.yml` и `ai-teamlead init`;
- guardrail-тесты против drift между схемой, default-layer и шаблоном;
- синхронизация README, feature `0002-repo-init` и нового ADR.

Вне scope:

- интерактивное заполнение `github.project_id` во время `init`;
- перенос repo-local конфига в глобальный пользовательский слой;
- автоматическая миграция уже закоммиченных пользовательских `settings.yml`;
- добавление новых product-level настроек вне текущего zero-config контракта.

## Связанные документы

- [README.md](./README.md)
- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/implementation-plan.md](../../../docs/implementation-plan.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../README.md](../../../README.md)
- [../../../docs/features/0002-repo-init/README.md](../../../docs/features/0002-repo-init/README.md)
- [../../../docs/features/0002-repo-init/02-how-we-build.md](../../../docs/features/0002-repo-init/02-how-we-build.md)
- [../../../docs/features/0002-repo-init/03-how-we-verify.md](../../../docs/features/0002-repo-init/03-how-we-verify.md)
- [../../../docs/adr/0001-repo-local-ai-config.md](../../../docs/adr/0001-repo-local-ai-config.md)
- [../../../docs/adr/0012-repo-init-command-and-project-contract-layer.md](../../../docs/adr/0012-repo-init-command-and-project-contract-layer.md)

## Зависимости и предпосылки

- текущий repo-level контракт оставляет `settings.yml` repo-local versioned
  конфигом, поэтому zero-config модель должна строиться поверх этого решения, а
  не вместо него;
- required repo-specific значение `github.project_id` не получает скрытый
  fallback;
- существующие fully materialized конфиги должны оставаться обратно
  совместимыми;
- нужен отдельный ADR, который зафиксирует canonical default-layer и policy для
  `example-only extension`.

## Порядок работ

### Этап 1. Зафиксировать schema policy и ADR

Цель:

- определить категории config keys и запретить неявные решения по новым полям;
- зафиксировать zero-config contract на уровне архитектурного решения.

Основание:

- issue требует явного различия между required-полями, runtime defaults и
  template examples;
- без ADR policy быстро расползется между кодом, template и README.

Результат этапа:

- создан новый ADR по zero-config contract `settings.yml`;
- для текущей схемы определен перечень `required-without-default`,
  `defaulted-by-application` и допустимых `example-only extension`;
- README и feature `0002` знают о новой роли шаблона хотя бы на уровне
  summary/ссылок.

Проверка:

- ADR и issue-spec не противоречат ADR-0001 и ADR-0012;
- по документам можно восстановить, почему `github.project_id` остается
  required, а defaults живут в коде.

### Этап 2. Перестроить loader на canonical default-layer

Цель:

- сделать отсутствие defaulted-полей допустимым runtime-path.

Основание:

- zero-config невозможен, пока loader ожидает материализованный активный YAML
  почти для всех секций.

Результат этапа:

- введена raw-модель override-конфига;
- появился canonical default-layer для defaulted-полей;
- итоговый `Config` собирается как `defaults + overrides`;
- required validation вынесена в отдельный явный шаг.

Проверка:

- unit-тесты на empty/comment-only YAML, partial override и required failure;
- backward-compatible YAML из старой формы продолжает грузиться.

### Этап 3. Обновить bootstrap template и `init`

Цель:

- привести generated `settings.yml` к documented zero-config форме.

Основание:

- runtime defaults должны жить в приложении, а template должен стать обзором
  доступных настроек и текущих default-значений.

Результат этапа:

- `templates/init/settings.yml` показывает defaulted-поля комментариями;
- required-поля и placeholders остаются явно видимыми;
- `src/init.rs` больше не материализует runtime defaults как обязательный
  активный YAML;
- integration-тест `init` знает про новую форму файла.

Проверка:

- `ai-teamlead init` создает `settings.yml` ожидаемой структуры;
- generated файл проходит читаемый обзор человеком и пригоден как zero-config
  стартовый контракт.

### Этап 4. Ввести guardrail на эволюцию схемы

Цель:

- сделать drift между кодом, template и документацией тестируемым сбоем, а не
  ручным ревью-замечанием.

Основание:

- issue прямо требует жесткую проверку, которая ломается при неполном
  обновлении схемы.

Результат этапа:

- есть metadata-слой классификации schema keys;
- есть parity-test между canonical default-layer и template;
- есть отдельная проверка для required keys и допустимых example-only
  исключений.

Проверка:

- искусственное добавление нового key без обновления metadata/template/defaults
  роняет тесты;
- drift в template default-примере тоже роняет тест.

### Этап 5. Закрыть docs sync и regression coverage

Цель:

- завершить изменение без расхождения верхнего слоя документации и реального
  runtime поведения.

Основание:

- `docs/documentation-process.md` требует обновлять профильные документы вместе
  с кодом;
- `docs/code-quality.md` требует тест на каждое значимое изменение конфига.

Результат этапа:

- README и feature `0002` описывают zero-config модель корректно;
- unit/integration tests зеленые;
- issue-спека, ADR и код используют одинаковую терминологию:
  `required-without-default`, `defaulted-by-application`, `example-only
  extension`.

Проверка:

- `cargo test` проходит;
- документация больше не описывает bootstrap `settings.yml` как единственный
  активный источник default-значений;
- проверен хотя бы один ручной smoke-сценарий с минимальным repo-local
  конфигом.

## Критерий завершения

Issue можно считать реализованной, если:

- defaulted-поля можно не активировать в `settings.yml`, а runtime все равно
  подставляет canonical defaults;
- `github.project_id` и другие required-поля дают явную диагностику без
  скрытого fallback;
- bootstrap шаблон отражает доступные настройки комментариями и не дублирует
  runtime defaults как второй источник истины;
- guardrail-тесты ломаются при неполной эволюции схемы;
- README, feature docs и ADR синхронизированы с кодом.

## Открытые вопросы и риски

- нужно аккуратно выбрать формат metadata-слоя, чтобы он был достаточно явным
  для guardrail, но не создавал лишний ручной реестр;
- если `example-only extension` появятся в текущем шаблоне, их маркировка
  должна быть машинно различимой, иначе guardrail станет неоднозначным;
- comment-only YAML может вести себя нетривиально в `serde_yaml`, поэтому
  merge-path стоит тестировать отдельно, а не считать очевидным;
- при неполной синхронизации docs можно случайно оставить старый bootstrap
  narrative в README даже после исправления кода.

## Журнал изменений

### 2026-03-14

- создан issue-level implementation plan для issue 33
