# Issue 49: Как строим

Статус: approved
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-14T23:11:41+03:00

## Approach

Изменение делаем как локальное расширение существующего `zellij` config
contract, а не как redesign всей template-системы.

Технический подход:

- добавить в `ZellijConfig` optional поле `tab_name_template`;
- рендерить effective tab name из issue context только для ветки
  `launch_target = tab`;
- сохранить `zellij.tab_name` literal stable именем для shared `pane` context;
- если `tab_name_template` отсутствует, использовать `zellij.tab_name` как
  fallback и в режиме `tab`;
- валидировать `tab_name_template` как поле, допускающее только
  `${ISSUE_NUMBER}` и literal текст без placeholders;
- вычислять effective tab name один раз до запуска `zellij` и передавать его
  во все места, где нужен фактический tab identity.

## Affected Areas

- `templates/init/settings.yml` и bootstrap project-local `settings.yml`;
- `src/config.rs` и unit-тесты конфигурации;
- `src/templates.rs` или соседний shared render-layer для
  `tab_name_template`;
- path, который вычисляет effective `zellij` launch context для issue;
- `src/zellij.rs`, где analysis tab layout получает имя вкладки;
- `src/runtime.rs` и operator-visible output, если там должен храниться или
  печататься фактический tab name;
- README, feature docs, verification docs и ADR-слой.

## Interfaces And Data

Входные данные:

- `zellij.tab_name` как stable repo-level имя shared tab context;
- `zellij.tab_name_template` как optional issue-aware naming template;
- `zellij.launch_target` из issue `#47`;
- `issue_number`, который уже известен launcher path;
- `analysis-tab.kdl`, где имя вкладки подставляется через `${TAB_NAME}`.

Выходные данные:

- `effective_tab_name`, которое:
  - в режиме `pane` всегда равно `zellij.tab_name`;
  - в режиме `tab` равно rendered `tab_name_template`, если поле задано;
  - в режиме `tab` равно `zellij.tab_name`, если template отсутствует;
- runtime manifest и launch diagnostics, отражающие фактически использованное
  имя вкладки.

Контракт поля `tab_name_template`:

- literal строка без `${...}` допустима;
- `${ISSUE_NUMBER}` поддерживается и рендерится из issue context;
- любые другие placeholders недопустимы;
- остаток `${...}` после рендера считается config error.

## Configuration And Runtime Assumptions

- issue `#47` уже вводит `zellij.launch_target` и делает `pane` runtime default;
- `tab_name_template` не влияет на выбор target session и не меняет
  `zellij.session_name`;
- `tab_name_template` игнорируется в режиме `pane`, но все равно должен
  проходить валидацию как часть config contract;
- старые конфиги без `tab_name_template` остаются валидными;
- bootstrap template должен явно показывать, что поле относится только к
  `tab`-режиму и является optional.

## Risks

- если effective tab name вычисляется только внутри layout render path, а
  runtime manifest остается на raw `zellij.tab_name`, появится дрейф между
  фактической вкладкой и metadata;
- если попытаться использовать `zellij.tab_name` как template вместо нового
  поля, режим `pane` потеряет stable shared context;
- если validation будет неполной, `zellij` может получить буквальное имя вида
  `${BRANCH}`;
- частичное обновление документации создаст конфликт между issue `#47`,
  existing ADR и новым naming contract.

## Architecture Notes

- вычисление `effective_tab_name` должно жить рядом с launcher decision, а не в
  `init` и не в project-local shell script;
- `runtime.create_claim_binding(...)` должен получать уже resolved tab name или
  иметь отдельный update-path, иначе `session.json` сохранит не ту вкладку, в
  которую реально запустили analysis;
- `print_zellij_launch_target(...)` и похожие operator-facing сообщения должны
  печатать effective имя вкладки, а не raw `zellij.tab_name`;
- `analysis-tab.kdl` продолжает использовать `${TAB_NAME}`, но source of truth
  для подставляемого значения становится resolved runtime tab name;
- template policy для `tab_name_template` лучше реализовать отдельной функцией,
  а не расширять special-case для `zellij.session_name`.

## ADR Impact

По правилам [../../../docs/documentation-process.md](../../../docs/documentation-process.md)
это изменение затрагивает versioned config contract, runtime semantics и
launcher metadata, поэтому решение должно быть явно зафиксировано на уровне
ADR.

Предпочтительный вариант:

- создать отдельный ADR для `zellij.tab_name_template` и split semantics между
  `tab_name` и `tab_name_template`.

Допустимый компромисс:

- если issue `#47` еще не оформлена и обе задачи сознательно реализуются как
  одно решение, объединить их в один новый ADR без молчаливого смешения.

## Alternatives Considered

1. Сделать `zellij.tab_name` template-capable полем.

   Отклонено: это ломает stable semantic роль `tab_name` для режима `pane`.

2. Сразу поддержать `${REPO}` и `${BRANCH}` в `tab_name_template`.

   Отклонено: в текущем issue подтвержден только сценарий с
   `${ISSUE_NUMBER}`, а расширение contract без отдельной необходимости
   увеличит verification surface.

3. Рендерить issue-aware имя только в `launch-layout.kdl`, не меняя runtime
   manifest и вывод CLI.

   Отклонено: это создает несогласованность между реальным runtime state и
   диагностическими артефактами.

## Migration Or Rollout Notes

- существующие конфиги без `tab_name_template` не требуют миграции;
- freshly initialized репозитории могут получить commented example для поля в
  `templates/init/settings.yml`, но fallback на `tab_name` должен оставаться
  работоспособным;
- rollout документации лучше выполнять после или вместе с фиксацией issue
  `#47`, чтобы не публиковать неполный launcher contract;
- headless integration coverage должна обновляться без запуска host `zellij`
  пользователя.
