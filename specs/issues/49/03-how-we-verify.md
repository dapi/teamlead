# Issue 49: Как проверяем

Статус: approved
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-14T23:11:41+03:00

## Acceptance Criteria

- в versioned config contract есть optional поле `zellij.tab_name_template`;
- `zellij.tab_name` и `zellij.tab_name_template` имеют разные семантики и не
  конфликтуют между режимами `pane` и `tab`;
- в режиме `tab` можно получить issue-aware имя вкладки вида `#<ISSUE_NUMBER>`;
- если `tab_name_template` отсутствует, режим `tab` fallback-ится на
  `zellij.tab_name`;
- unsupported placeholders в `tab_name_template` приводят к явной ошибке
  конфигурации до запуска `zellij`;
- runtime manifest, launch log и operator-visible output отражают effective
  имя вкладки;
- bootstrap и связанная документация синхронизированы с новым контрактом.

## Ready Criteria

- issue классифицирована как `medium feature` для `infra/platform`;
- зафиксировано, что `#49` зависит от контракта `launch_target` из issue `#47`
  или реализуется вместе с ним;
- placeholder set для `tab_name_template` ограничен `${ISSUE_NUMBER}` и literal
  текстом без placeholders;
- принято явное решение по ADR для нового naming contract;
- verification path спроектирован так, чтобы не трогать host `zellij`
  пользователя вне headless-среды.

## Invariants

- `zellij.tab_name` остается stable semantic именем shared tab context;
- `zellij.tab_name_template` не влияет на режим `pane`;
- `zellij.tab_name_template` не меняет `zellij.session_name` и session target
  resolution;
- runtime не передает в `zellij` неразрешенные `${...}`;
- runtime metadata и CLI diagnostics не должны расходиться с фактически
  созданным tab name.

## Test Plan

Unit tests:

- `Config` успешно парсится с `tab_name_template` и без него;
- пустой `tab_name_template`, если поле задано, отклоняется как invalid config;
- renderer превращает `#${ISSUE_NUMBER}` в `#42`;
- literal `tab_name_template` проходит без изменений;
- `${BRANCH}` и другие неподдерживаемые placeholders приводят к явной ошибке;
- выбор `effective_tab_name` покрыт тестами для:
  - `pane` -> всегда `zellij.tab_name`;
  - `tab + template` -> rendered template;
  - `tab + no template` -> fallback на `zellij.tab_name`;
- runtime manifest получает effective tab name, а не raw config value.

Integration/headless tests:

- `launch_target = tab`, `tab_name_template = "#${ISSUE_NUMBER}"` создает
  вкладку `#42`;
- `launch_target = tab`, template отсутствует, и launcher использует fallback
  `issue-analysis`;
- `launch_target = pane`, даже при заданном `tab_name_template`, запуск
  переиспользует stable `issue-analysis` context;
- невалидный placeholder завершает запуск до первого IPC/CLI-вызова `zellij`;
- повторные запуски в `tab`-режиме для разных issue дают разные effective имена
  вкладок, а не дублирующий `issue-analysis`.

Manual validation:

- прогнать headless test runner или Docker-based `zellij` integration path;
- проверить `session.json`, `launch.log` и stdout после запуска в режимах
  `tab` и `pane`;
- проверить generated `settings.yml` после `init` и убедиться, что новое поле
  описано как optional и `tab`-specific.

## Verification Checklist

- config contract обновлен и покрыт unit-тестами;
- renderer/validator для `tab_name_template` покрыт positive и negative cases;
- effective tab name проброшен в runtime manifest;
- launch diagnostics печатают фактическое имя вкладки;
- bootstrap template и docs обновлены;
- есть headless integration coverage для template path и fallback semantics;
- проверки не запускают опасные `zellij` сценарии в host-окружении.

## Happy Path

1. Репозиторий задает:
   `zellij.launch_target: "tab"` и
   `zellij.tab_name_template: "#${ISSUE_NUMBER}"`.
2. Оператор запускает `ai-teamlead run 42`.
3. Runtime рендерит `effective_tab_name = "#42"`.
4. Launcher создает analysis tab с именем `#42`.
5. `session.json`, `launch.log` и stdout показывают ту же вкладку `#42`.

## Edge Cases

- `tab_name_template` отсутствует полностью;
- `tab_name_template` содержит несколько вхождений `${ISSUE_NUMBER}`;
- `tab_name_template` задан literal строкой без placeholders;
- `tab_name_template` задан, но режим запуска остается `pane`.

## Failure Scenarios

- в `tab_name_template` указан `${BRANCH}` или другой неподдерживаемый
  placeholder;
- validation пропущена и runtime пытается создать вкладку с буквальным
  `${...}`;
- effective tab name корректно уходит в `zellij`, но не попадает в
  `session.json` или CLI-output;
- документация обновлена частично и оператор не понимает, какое поле относится
  к `pane`, а какое к `tab`.

## Observability

- `session.json` в `.git/.ai-teamlead/sessions/<session_uuid>/` должен хранить
  effective `tab_name`;
- `launch.log` должен позволять восстановить, какое имя вкладки было выбрано;
- stdout `run`/`poll` должен печатать фактический launch target с корректным
  tab name для операторской диагностики.
