# Issue 33: Как проверяем

Статус: approved
Последнее обновление: 2026-03-14
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-14T23:06:53+03:00

## Acceptance Criteria

1. `ai-teamlead init` по-прежнему создает `./.ai-teamlead/settings.yml`, но
   defaulted-поля в bootstrap шаблоне представлены в закомментированном виде.
2. Runtime корректно подставляет canonical defaults для отсутствующих
   defaulted-полей без требования вручную разкомментировать шаблон.
3. Comment-only или почти пустой `settings.yml` успешно проходит этап парсинга
   и merge для defaulted-полей.
4. `github.project_id` и другие `required-without-default` поля не получают
   скрытый fallback и дают понятную диагностику при отсутствии или пустом
   значении.
5. Старые fully materialized конфиги продолжают загружаться без миграции и без
   поведенческой регрессии.
6. Guardrail ломает тесты, если новый schema key добавлен без явной
   классификации, без default-layer или без отражения в шаблоне.
7. Template, default-layer и документация синхронизированы по смыслу и не
   расходятся относительно роли `settings.yml`.

## Ready Criteria

- issue-spec, implementation plan, README и feature `0002-repo-init` согласованы
  по модели `documented template + runtime defaults + required fields`;
- для schema keys существует одна проверяемая классификация;
- реализация loader clearly separates `parse`, `merge defaults` и
  `validate required fields`;
- предусмотрен один явный test path для drift между Rust defaults и
  `templates/init/settings.yml`;
- будущий ADR на zero-config contract описан и включен в план реализации.

## Invariants

- canonical runtime defaults живут в одном кодовом слое;
- отсутствие defaulted-поля в активном YAML эквивалентно использованию
  canonical default-значения;
- required-поля не маскируются под defaults и не теряют operator-facing
  диагностику;
- каждый schema key относится ровно к одной категории:
  `required-without-default`, `defaulted-by-application` или
  `example-only extension`;
- `templates/init/settings.yml` остается versioned обзором доступных настроек,
  а не вторым источником runtime-истины;
- допустимое отличие template от runtime default возможно только для явно
  помеченного `example-only extension`.

## Happy Path

1. Пользователь выполняет `ai-teamlead init` и получает `settings.yml`, где
   defaulted-поля показаны комментариями.
2. Пользователь задает только `github.project_id`, не активируя остальные
   значения.
3. Runtime загружает конфиг, подставляет missing defaults и проходит валидацию.
4. Поведение приложения совпадает с тем, как если бы все defaulted-поля были
   явно записаны в YAML.

## Edge Cases

- файл `settings.yml` содержит только комментарии;
- присутствует только часть секций, например один `zellij` override;
- required-поле присутствует, но пусто или состоит из пробелов;
- в template есть commented example, который не является runtime default;
- разработчик добавил новое поле в Rust-модель, но не обновил template или
  guardrail metadata.

## Test Plan

Unit tests:

- загрузка empty/comment-only YAML как допустимого override-слоя для
  defaulted-полей;
- merge-path: частично заданный YAML переопределяет только указанные поля, а
  остальные берет из canonical defaults;
- required validation: отсутствие и пустое значение `github.project_id`
  приводят к ожидаемой диагностике;
- backward compatibility: старый fully materialized YAML продолжает
  десериализоваться;
- schema-classification test: каждый config key присутствует в metadata-слое
  классификации;
- template/default parity test: runtime defaults совпадают с commented defaults
  в `templates/init/settings.yml`, кроме явно разрешенных example-only полей.

Integration tests:

- `ai-teamlead init` создает `settings.yml` в новой zero-config форме;
- generated `settings.yml` содержит комментарии для defaulted-полей и не
  материализует их как активный YAML;
- runtime path, использующий `Config::load_from_repo_root`, работает с минимальным
  repo-local конфигом, где активным задано только required-поле;
- regression для существующих integration helper'ов не ломается из-за новой
  формы шаблона.

Manual / smoke:

- выполнить `ai-teamlead init` в тестовом репозитории и визуально подтвердить,
  что `settings.yml` читается как обзор настроек, а не как заполненный dump;
- задать только `github.project_id` и убедиться, что runtime использует
  defaults без ручного раскомментирования остальных полей;
- искусственно внести drift между template и default-layer и убедиться, что
  guardrail-тест падает.

## Verification Checklist

- `templates/init/settings.yml` показывает defaulted-поля комментариями;
- код содержит один canonical default-layer для runtime;
- required-поля валидируются отдельной диагностикой;
- unit-тесты для `src/config.rs` покрывают empty YAML, partial override,
  required failure и backward compatibility;
- integration-тесты `init` проверяют новую форму bootstrap шаблона;
- есть тест, который ломается при добавлении schema key без обновления
  metadata/template/default-layer;
- README и feature docs больше не описывают `settings.yml` как единственный
  активный источник default-значений.

## Failure Scenarios

- Пустой YAML ошибочно трактуется как поломанный файл, хотя должен быть
  допустимым bootstrap-состоянием.
- `github.project_id` случайно начинает default-иться и скрывает обязательный
  операторский шаг.
- Новый key добавлен в `Config`, но не отражен в template, из-за чего `init`
  перестает документировать часть схемы.
- Template показывает один default, а runtime использует другой, и guardrail не
  замечает расхождения.

## Observability

- Ошибки загрузки конфига должны различать этапы `read`, `parse`, `merge` и
  `validate`;
- сообщения о required-полях должны указывать точный key, а не абстрактную
  ошибку десериализации;
- guardrail failure должен явно говорить, что сломалось:
  классификация схемы, parity defaults/template или required contract;
- integration assertions по `init` должны проверять содержимое созданного
  `settings.yml`, а не только сам факт существования файла.
