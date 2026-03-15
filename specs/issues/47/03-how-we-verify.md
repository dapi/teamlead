# Issue 47: Как проверяем

Статус: approved
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-15T11:48:33+03:00

## Acceptance Criteria

- в versioned config contract есть поле `zellij.launch_target`;
- поле принимает только `pane` и `tab`;
- при отсутствии поля runtime использует default `pane`;
- `ai-teamlead run <issue> --launch-target pane` запускает analysis через
  `pane` path;
- `ai-teamlead run <issue> --launch-target tab` запускает analysis через
  `tab` path;
- precedence `CLI override -> settings.yml -> runtime default` покрыт тестами;
- override не меняет `settings.yml`;
- `poll` и `loop` не получают отдельный public `--launch-target` flag;
- `pane`-режим переиспользует existing shared tab, а при его отсутствии создает
  его до запуска pane;
- duplicate tab в `pane`-режиме приводит к явной ошибке, а не к silent
  fallback;
- `tab`-режим сохраняет текущее behavior создания отдельной analysis tab;
- bootstrap template, repo-local docs и launcher docs синхронизированы с новым
  контрактом.

## Ready Criteria

- issue классифицирована как `medium feature` для `infra/platform`;
- принято явное решение, что public override добавляется только в `run`;
- зафиксирован runtime default `pane`;
- зафиксировано взаимодействие с уже существующим tab naming contract;
- verification path использует только headless/Docker-based `zellij` checks для
  опасных launcher сценариев;
- ADR на launcher contract включен в план реализации.

## Invariants

- resolution target session через `--zellij-session`, env и settings не меняется;
- `launch_target` влияет только на способ открытия launch context внутри уже
  выбранной session;
- `pane` всегда использует stable shared tab context;
- `tab_name_template` не должен менять `pane`-ветку;
- `run` override не персистится в config;
- `launch-agent.sh` и `bind-zellij-pane` остаются неизменным project-local
  execution contract;
- ambiguous duplicate tabs в `pane` path недопустимы;
- проверки не должны трогать host `zellij` пользователя.

## Test Plan

Unit tests:

- `Config` успешно парсится с `zellij.launch_target: pane`;
- `Config` успешно парсится с `zellij.launch_target: tab`;
- отсутствие `zellij.launch_target` приводит к default `pane`;
- невалидное значение `launch_target` отклоняется как invalid config;
- CLI parser принимает `run --launch-target pane`;
- CLI parser принимает `run --launch-target tab`;
- resolution precedence покрыт case-ами:
  - CLI override поверх `settings = pane`;
  - CLI override поверх `settings = tab`;
  - fallback на settings без CLI override;
  - fallback на runtime default при отсутствии поля;
- в режиме `pane` effective tab name остается равным stable `zellij.tab_name`
  даже при наличии `tab_name_template`;
- `poll` и `loop` не экспонируют public `launch_target` override.

Integration/headless tests:

- `config default = pane` создает или находит shared tab и запускает новую pane;
- `config default = tab` использует existing `new-tab` launcher path;
- `run --launch-target pane` поверх `config default = tab` идет по `pane` path;
- `run --launch-target tab` поверх `config default = pane` идет по `tab` path;
- в `pane`-режиме при отсутствии shared tab launcher сначала создает tab, затем
  запускает pane;
- в `pane`-режиме при наличии duplicate tabs запуск завершается ошибкой;
- `tab_name_template`, если настроен, влияет только на `tab`-режим;
- stdout и `launch.log` показывают выбранный `launch_target`.

Manual validation:

- прогнать headless runner или Docker-based integration suite;
- проверить `session.json`, `launch.log` и stdout после запусков в режимах
  `pane` и `tab`;
- проверить generated `settings.yml` после `init` и убедиться, что default
  `launch_target: "pane"` задокументирован.

## Verification Checklist

- config contract обновлен и покрыт unit-тестами;
- CLI contract для `run` обновлен и покрыт parser-тестами;
- precedence resolution покрыт unit-тестами;
- `pane` path покрыт headless integration test-ами на create/reuse/failure;
- `tab` path не регресснул относительно существующего launcher behavior;
- diagnostics отражают effective launch mode и effective tab context;
- docs и bootstrap template обновлены;
- ни одна проверка не требует запуска опасных zellij helper в host-окружении.

## Happy Path

1. Репозиторий хранит `zellij.launch_target: "pane"`.
2. Оператор запускает `ai-teamlead run 42`.
3. Runtime выбирает `pane` как effective target.
4. Launcher находит shared tab `issue-analysis` или создает его.
5. В target tab открывается новая pane и запускает `launch-agent.sh`.
6. `launch.log` и stdout показывают `launch_target=pane`.

## Edge Cases

- `zellij.launch_target` отсутствует полностью;
- `run --launch-target ...` конфликтует с opposite config default;
- `tab_name_template` задан, но запуск идет в `pane`;
- shared tab отсутствует и должен быть создан на лету;
- shared tab найден больше одного раза.

## Failure Scenarios

- config содержит невалидное значение `launch_target`;
- `pane` path не может однозначно определить target tab;
- `pane` path silently fallback-ится в `new-tab`, хотя ожидался reuse shared tab;
- diagnostics печатают session/tab ids, но не печатают выбранный mode;
- docs обновлены частично, и оператор не понимает, почему `poll` не принимает
  `--launch-target`.

## Observability

- stdout `run` и `poll` должен явно показывать выбранный `launch_target`;
- `launch.log` должен позволять восстановить, по какой ветке launcher пошел:
  `pane` или `tab`;
- `session.json` должен хранить effective `tab_name`, а не raw template path;
- при ошибке duplicate tab диагностика должна содержать имя проблемного tab
  context.
