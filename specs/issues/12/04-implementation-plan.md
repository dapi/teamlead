# Issue 12: План имплементации

Статус: draft
Последнее обновление: 2026-03-14

## Назначение

Этот план задает порядок реализации поддержки `zellij.layout` и нового
fallback-path для создания новой `zellij` session без bare generated layout.

## Scope

В scope входит:

- добавить `zellij.layout` в versioned config contract;
- изменить launcher path для `session missing`;
- сохранить behavior для `session exists`;
- обновить template `settings.yml`;
- добавить unit и smoke coverage;
- синхронизировать issue-спеку, feature 0003 и ADR.

Вне scope:

- поддержка пути к `.kdl` файлу;
- redesign launcher под разные launch target modes;
- изменение session/tab naming contract.

## Связанные документы

- [README.md](./README.md)
- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/features/0003-agent-launch-orchestration/02-how-we-build.md](../../../docs/features/0003-agent-launch-orchestration/02-how-we-build.md)
- [../../../docs/features/0003-agent-launch-orchestration/03-how-we-verify.md](../../../docs/features/0003-agent-launch-orchestration/03-how-we-verify.md)
- [../../../docs/adr/0011-use-zellij-main-release-in-ci.md](../../../docs/adr/0011-use-zellij-main-release-in-ci.md)
- [../../../docs/adr/0022-zellij-layout-contract-for-new-sessions.md](../../../docs/adr/0022-zellij-layout-contract-for-new-sessions.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)

## Зависимости и предпосылки

- pinned `zellij` version в CI и локальном smoke должна поддерживать
  session-scoped `action new-tab --layout`;
- существующий path `session exists` уже стабилизирован и не должен менять
  observable behavior;
- реализация должна оставаться обратно совместимой для старых `settings.yml`.

## Порядок работ

### Этап 1. Config contract

Цель:

- добавить `layout: Option<String>` в `ZellijConfig`;
- обновить sample/template config и парсинг.

Основание:

- issue-spec и ADR фиксируют новое repo-level поле.

Результат этапа:

- конфиг грузится с `layout` и без него;
- старые YAML остаются валидными;
- `templates/init/settings.yml` содержит документированный пример поля.

Проверка:

- unit-тесты парсинга и backward compatibility.

### Этап 2. Launcher branch для `session missing`

Цель:

- разделить path создания новой session и path добавления analysis tab.

Основание:

- fallback без bare generated layout зафиксирован в spec и ADR.

Результат этапа:

- при `layout = Some(name)` launcher создает session через пользовательский
  layout, затем добавляет analysis tab;
- при `layout = None` launcher создает session без `-n <generated layout>`,
  затем добавляет analysis tab;
- при `session exists` сохраняется текущий path добавления tab.

Проверка:

- unit-тесты команд для трех веток:
  `existing session`, `custom layout`, `default fallback`.

### Этап 3. Диагностика и errors

Цель:

- сделать ветки launcher различимыми в логах и ошибках.

Основание:

- verification требует явной локализации шагов `create session` и
  `add analysis tab`.

Результат этапа:

- ошибки оборачиваются контекстом шага;
- в launcher-логах виден выбранный branch выполнения.

Проверка:

- unit-тесты или assertions на сообщения ошибок и вызванные команды.

### Этап 4. Regression и smoke

Цель:

- подтвердить, что новый path не ломает уже работающий orchestration contract.

Основание:

- feature 0003 требует сохранить behavior existing session и runtime artifacts.

Результат этапа:

- `cargo test` проходит;
- headless-friendly `zellij` coverage остается зеленой;
- ручной smoke с `layout` и без него подтверждает expected path.

Проверка:

- unit tests;
- `cargo test`;
- ручной smoke на pinned `zellij`.

## Критерий завершения

Issue можно считать реализованной, если:

- `zellij.layout` добавлен без регрессии старых конфигов;
- path `session missing` больше не использует bare generated layout как базовую
  session при `layout = None`;
- `layout = Some(name)` и `layout = None` покрыты тестами;
- issue, feature-спека и ADR синхронизированы.

## Риски и открытые вопросы

- CLI `zellij` может требовать отдельного ожидания между созданием session и
  `action new-tab`;
- smoke на “default UX” нельзя сводить к визуальной оценке, поэтому опорным
  контрактом остается форма команд и launcher path;
- если выяснится, что `zellij` не поддерживает нужный attach-path стабильно,
  придется заводить follow-up issue, а не расширять scope этой задачи.

## Журнал изменений

### 2026-03-14

- создан issue-level implementation plan для issue 12
