# Issue 47: План имплементации

Статус: approved
Последнее обновление: 2026-03-15
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-15T11:48:33+03:00

## Назначение

Этот план связывает analysis-решения по issue `#47` с конкретным порядком
реализации launcher contract для `pane`/`tab`, чтобы изменения в CLI, config,
runtime и verification были прослеживаемыми и не разъехались по разным слоям.

## Scope

В план входит:

- новый versioned config contract `zellij.launch_target`;
- public CLI override только для `run`;
- zellij dispatch между `pane` и `tab`;
- обновление документации, bootstrap template и тестов.

## Вне scope

- изменение session target resolution contract;
- отдельный redesign `launch-agent.sh`;
- новые multiplexer backend;
- host-side zellij e2e вне headless path.

## Связанные документы

- Issue: https://github.com/dapi/ai-teamlead/issues/47
- Feature / issue spec:
  - [README.md](./README.md)
  - [01-what-we-build.md](./01-what-we-build.md)
  - [02-how-we-build.md](./02-how-we-build.md)
  - [03-how-we-verify.md](./03-how-we-verify.md)
- SSOT:
  - [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
  - [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
- ADR:
  - [../../../docs/adr/0005-cli-contract-for-poll-and-run.md](../../../docs/adr/0005-cli-contract-for-poll-and-run.md)
  - [../../../docs/adr/0015-versioned-launch-agent-contract.md](../../../docs/adr/0015-versioned-launch-agent-contract.md)
  - [../../../docs/adr/0023-zellij-session-target-resolution.md](../../../docs/adr/0023-zellij-session-target-resolution.md)
- Verification:
  - [03-how-we-verify.md](./03-how-we-verify.md)
- Code quality:
  - [../../../docs/code-quality.md](../../../docs/code-quality.md)
- Зависимые планы или фичи:
  - [../49/README.md](../49/README.md)

## Зависимости и предпосылки

- нужно определить и зафиксировать новый ADR до или одновременно с изменением
  кода;
- текущий `tab` launcher path уже существует и должен остаться совместимым;
- verification для `zellij` должна выполняться только в headless-среде;
- если в основной ветке уже есть изменения по `#49`, они должны использоваться
  как dependency, а не переписываться.

## Порядок работ

### Этап 1. Зафиксировать документационный контракт

Цель:

- оформить канонический contract layer до правок runtime-кода.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [../../../docs/documentation-process.md](../../../docs/documentation-process.md)

Результат этапа:

- обновлены SSOT/feature docs вокруг launcher contract;
- создан новый ADR про `zellij.launch_target`, runtime default `pane` и
  precedence `CLI -> settings -> default`;
- синхронизированы ссылки на связь с issue `#49`.

Проверка:

- review versioned docs и ADR на непротиворечивость;
- no missing links между issue spec, SSOT и ADR.

### Этап 2. Реализовать config и CLI resolution layer

Цель:

- ввести единый resolved contract для `launch_target`.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- `src/config.rs` поддерживает `zellij.launch_target` с default `pane`;
- `src/cli.rs` поддерживает `run --launch-target`;
- общий app-layer резолвит precedence и не добавляет public override в
  `poll`/`loop`;
- diagnostics получают доступ к effective mode.

Проверка:

- unit-тесты на parsing и precedence;
- parser-тесты на CLI surface.

### Этап 3. Добавить zellij dispatch для `pane` path

Цель:

- реализовать runtime behavior для shared tab reuse без регрессии `tab` path.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)

Результат этапа:

- `src/zellij.rs` разделяет `tab` и `pane` launcher branches;
- `pane` branch умеет:
  - найти shared tab;
  - создать shared tab при отсутствии;
  - открыть новую pane в найденной tab;
  - завершиться явной ошибкой при duplicate tab;
- `tab` branch сохраняет существующий behavior.

Проверка:

- headless integration tests для `pane` create/reuse/failure;
- регрессионные проверки существующего `tab` path.

### Этап 4. Синхронизировать bootstrap и verification surface

Цель:

- сделать новый contract discoverable и проверяемым в dogfooding path.

Основание:

- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)

Результат этапа:

- обновлены `templates/init/settings.yml` и repo-local `./.ai-teamlead/settings.yml`;
- stdout, `launch.log` и runtime metadata отражают effective launch mode;
- integration suite покрывает обе ветки и precedence cases.

Проверка:

- headless test runner проходит;
- ручная проверка generated `settings.yml` и диагностических артефактов.

## Критерий завершения

- contract `zellij.launch_target` задокументирован, реализован и покрыт тестами;
- `run` поддерживает одноразовый override без мутации config;
- `poll` и `loop` остаются config-driven без нового public override;
- `pane` path корректно создает/reuse shared tab и не допускает silent fallback;
- `tab` path не регрессирует;
- docs, ADR и bootstrap template синхронизированы.

## Открытые вопросы и риски

- нужно аккуратно выбрать zellij IPC/path для создания pane в уже существующей
  tab, не ломая текущий session-scoped contract;
- если runtime metadata не будет отражать effective mode, диагностика окажется
  слабее, чем ожидает analysis;
- issue `#49` уже формализует часть tab semantics, поэтому реализация должна
  явно избежать дублирования или скрытого конфликта.

## Журнал изменений

### 2026-03-15

- создан начальный план имплементации для issue `#47`
