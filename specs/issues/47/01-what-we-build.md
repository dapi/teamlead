# Issue 47: Что строим

Статус: approved
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-15T11:48:33+03:00

## Problem

Сейчас `ai-teamlead run <issue>` фактически всегда идет по одному launcher path:
создает новую analysis tab в `zellij` session.

Это создает несколько пробелов в операторском контракте:

- repo не может зафиксировать удобный default mode между `pane` и `tab`;
- оператор не может разово переопределить способ запуска без правки
  `./.ai-teamlead/settings.yml`;
- поведение `run` остается partially implicit: неясно, что важнее между CLI,
  config и встроенным fallback;
- shared tab сценарий для dogfooding не описан как канонический runtime path;
- `pane`-ветка не имеет явного требования по reuse existing tab и защите от
  duplicate tab.

## Who Is It For

- оператор, который вручную запускает `ai-teamlead run <issue>` и хочет быстро
  переключаться между `pane` и `tab`;
- владелец репозитория, который задает repo-level launcher defaults в
  `./.ai-teamlead/settings.yml`;
- разработчик `ai-teamlead`, который поддерживает единый и проверяемый
  orchestration contract для `zellij`.

## Outcome

Нужен явный launcher contract, в котором:

- в `zellij` config появляется поле `launch_target`;
- поддерживаются только значения `pane` и `tab`;
- если поле не задано, runtime default равен `pane`;
- `run` принимает одноразовый override `--launch-target <pane|tab>`;
- effective precedence зафиксирован как
  `run --launch-target` -> `zellij.launch_target` -> runtime default;
- override влияет только на текущий запуск и не меняет repo-local config;
- `pane`-режим использует stable shared tab context;
- `tab`-режим сохраняет текущее поведение открытия отдельной вкладки;
- `poll` и `loop` остаются детерминированными и используют только config/default
  path без отдельного public override.

## Scope

В текущую задачу входит:

- расширение versioned config contract полем `zellij.launch_target`;
- фиксация runtime default для отсутствующего поля;
- добавление public CLI override для `run`;
- явное решение, что `poll` и `loop` не получают аналогичный public override;
- обновление общего launcher dispatch между `pane` и `tab`;
- фиксация `pane`-семантики:
  - reuse existing shared tab;
  - create tab first, если shared tab отсутствует;
  - запрет silent fallback в duplicate tab;
- сохранение и документирование `tab`-семантики как existing behavior;
- обновление bootstrap/template/docs/ADR/verification contract;
- добавление unit и headless integration coverage для precedence и обоих режимов.

## Non-Goals

В текущую задачу не входит:

- изменение precedence для `--zellij-session`, `ZELLIJ_SESSION_NAME` и
  `zellij.session_name`;
- редизайн `launch-agent.sh` или stage lifecycle;
- поддержка других multiplexer backend кроме `zellij`;
- изменение `poll` в интерактивный операторский инструмент с разовыми
  launch-override флагами;
- автоматическая правка `settings.yml` при использовании CLI override;
- новый naming contract для issue-aware tab сам по себе без уже принятого
  соседнего контракта;
- запуск `zellij`-проверок в host session пользователя.

## Constraints And Assumptions

- старые конфиги без `zellij.launch_target` должны оставаться валидными;
- `pane` становится каноническим runtime default и должен быть явно
  задокументирован в bootstrap template;
- `run` override носит ephemeral характер и не переживает конкретный запуск;
- public CLI surface не должен разъезжаться между `run` и внутренним launch
  dispatch: общая логика обязана получать уже resolved target;
- `pane` использует stable shared tab name и не должен подменяться
  `tab_name_template`;
- если в репозитории уже принят contract из issue `#49`, `tab`-ветка должна
  использовать его без отдельного forked поведения;
- verification path для `zellij` должен оставаться headless/Docker-based.

## User Story

Как оператор, который работает с analysis issue в `zellij`, я хочу иметь
repo-level default и разовый `run` override для выбора между `pane` и `tab`,
чтобы не редактировать `settings.yml` ради одного запуска и при этом сохранять
предсказуемый launcher contract для автоматических путей.

## Use Cases

1. Репозиторий хранит `zellij.launch_target: "pane"`, и `ai-teamlead run 42`
   открывает новую pane внутри existing shared tab `issue-analysis`.
2. Репозиторий хранит `zellij.launch_target: "tab"`, и `ai-teamlead run 42`
   открывает отдельную analysis tab, как и текущий launcher path.
3. Репозиторий использует `launch_target: "tab"`, но оператор запускает
   `ai-teamlead run 42 --launch-target pane` для одного конкретного сеанса.
4. Репозиторий использует `launch_target: "pane"`, но оператор запускает
   `ai-teamlead run 42 --launch-target tab` для изолированного разового анализа.
5. `ai-teamlead poll` запускается без дополнительных флагов и использует только
   repo-level default или встроенный runtime default.

## Dependencies

- `#13` задает исходную потребность разделить analysis launch path между
  `pane` и `tab`.
- [../49/README.md](../49/README.md) фиксирует naming-слой для `tab`-режима и
  должен использоваться, а не переопределяться, если уже принят.
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
  задает текущий orchestration contract вокруг `zellij`.
- [../../../docs/adr/0023-zellij-session-target-resolution.md](../../../docs/adr/0023-zellij-session-target-resolution.md)
  фиксирует precedence для target session и не должен быть затронут новым
  `launch_target`.
