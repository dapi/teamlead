# Issue 49: Что строим

Статус: approved
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-14T23:11:41+03:00

## Problem

После появления ветки `launch_target = tab` все analysis-запуски начинают
создавать новые вкладки с одним и тем же именем `issue-analysis`.

Это неудобно для живой `zellij` session:

- невозможно быстро понять, какая вкладка относится к какой issue;
- одинаковые имена вкладок ухудшают навигацию при dogfooding и ручном `run`;
- попытка превратить существующий `zellij.tab_name` в template смешивает две
  разные семантики:
  - stable shared tab context для режима `pane`;
  - issue-aware naming для режима `tab`.

## Who Is It For

- оператор, который запускает analysis в режиме `tab` и хочет видеть связь
  вкладки с конкретной issue;
- владелец репозитория, который настраивает `./.ai-teamlead/settings.yml`;
- разработчик `ai-teamlead`, который поддерживает непротиворечивый launcher
  contract для `zellij`.

## Outcome

Нужен отдельный versioned naming contract, в котором:

- `zellij.tab_name` остается stable semantic именем shared tab context;
- появляется optional поле `zellij.tab_name_template`;
- `zellij.tab_name_template` используется только в ветке
  `launch_target = tab`;
- минимально поддерживается placeholder `${ISSUE_NUMBER}`;
- если `tab_name_template` отсутствует, режим `tab` fallback-ится на
  `zellij.tab_name`;
- неподдерживаемые placeholders дают явную ошибку конфигурации;
- runtime metadata и operator-visible output отражают фактическое имя вкладки,
  а не только raw значение из конфига.

## Scope

В текущую задачу входит:

- расширение versioned config contract новым полем `zellij.tab_name_template`;
- фиксация supported placeholder set для `tab_name_template`;
- фиксация runtime semantics для `pane` и `tab`;
- обновление launcher path так, чтобы effective имя вкладки рендерилось до
  передачи в `zellij`;
- выравнивание runtime manifest, launch log и CLI-output с effective tab name;
- обновление bootstrap templates, launcher docs и verification docs;
- фиксация ADR impact для нового naming contract.

## Non-Goals

В текущую задачу не входит:

- введение самого `zellij.launch_target` без issue `#47`;
- изменение default semantics режима `pane`;
- превращение `zellij.tab_name` в template-capable поле;
- расширение `tab_name_template` на `${REPO}`, `${BRANCH}` и другие
  placeholders без отдельного подтвержденного сценария;
- изменение контракта `zellij.session_name`;
- автоматическая миграция уже существующих runtime session/tab.

## Constraints And Assumptions

- `#49` расширяет контракт issue `#47`, где вводится различение режимов
  `pane` и `tab`;
- `pane` должен продолжать использовать stable `zellij.tab_name` как shared
  semantic context;
- `tab_name_template` должен оставаться optional полем и не ломать старые
  конфиги;
- строка без placeholders считается допустимым частным случаем template, но
  канонический issue-aware сценарий для `tab` использует `${ISSUE_NUMBER}`;
- ошибка конфигурации должна возникать до попытки открыть новый `zellij` tab;
- документация должна зафиксировать контракт раньше или одновременно с кодом.

## User Story

Как оператор, который использует `tab`-режим для изоляции analysis-запусков,
я хочу видеть issue-aware имя вкладки вроде `#42`, чтобы быстро находить
нужный analysis context, не ломая shared tab semantics режима `pane`.

## Use Cases

1. Репозиторий задает `zellij.launch_target: "tab"` и
   `zellij.tab_name_template: "#${ISSUE_NUMBER}"`, после чего запуск issue
   `#42` создает вкладку `#42`.
2. Репозиторий использует `launch_target: "tab"`, но не задает
   `tab_name_template`, и runtime fallback-ится на `zellij.tab_name`.
3. Репозиторий хранит `tab_name_template`, но запускается в режиме `pane`, и
   analysis по-прежнему использует stable shared tab `issue-analysis`.

## Dependencies

- `#47` — вводит `zellij.launch_target` и базовые `pane/tab` semantics,
  которые эта задача только расширяет naming-слоем.
- `#13` — фиксирует исходную проблему разделения `pane` и `tab` для analysis
  launch path.
- [../../../docs/adr/0014-zellij-launch-context-naming.md](../../../docs/adr/0014-zellij-launch-context-naming.md)
  и [../../../docs/adr/0023-zellij-session-target-resolution.md](../../../docs/adr/0023-zellij-session-target-resolution.md)
  задают существующий naming/session contract, который нужно расширить без
  противоречий.
