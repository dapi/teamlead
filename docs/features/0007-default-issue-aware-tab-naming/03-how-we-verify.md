# Feature 0007: Как проверяем

## Критерии корректности

- при `launch_target = tab` и отсутствии active `tab_name_template` effective
  имя вкладки равно `#<ISSUE_NUMBER>`;
- при `launch_target = pane` effective имя вкладки остается `issue-analysis`;
- explicit literal `tab_name_template` переопределяет новый default;
- runtime manifest и diagnostics отражают resolved tab name, а не raw policy.

## Критерии готовности

- новый ADR принят и не конфликтует с ADR-0031/0032;
- `docs/config.md`, Feature 0002 и Feature 0003 синхронизированы;
- bootstrap template объясняет новую семантику без двусмысленности;
- unit tests и headless integration tests покрывают новый default и opt-out.

## Инварианты

- `pane`-режим продолжает использовать stable shared tab;
- placeholder policy не расширяется;
- launcher diagnostics и runtime manifests остаются согласованными;
- старое literal имя для `tab`-режима остается доступным только как explicit
  override.

## Сценарии проверки

1. Repo не задает `zellij.tab_name_template`, использует `launch_target = tab`,
   и запуск issue `42` создает вкладку `#42`.
2. Repo не задает `zellij.tab_name_template`, использует `launch_target = pane`,
   и запуск создает pane в shared tab `issue-analysis`.
3. Repo явно задает `zellij.tab_name_template: "issue-analysis"`, использует
   `launch_target = tab`, и runtime создает вкладку `issue-analysis`.
4. Repo задает `zellij.tab_name_template: "#${ISSUE_NUMBER}"`, и поведение
   совпадает с новым default.
5. Repo задает unsupported placeholder, и config validation по-прежнему падает
   с явной ошибкой.

## Диагностика и наблюдаемость

- stdout и launch log должны показывать resolved `tab_name`;
- unit tests должны явно различать `pane` и `tab` naming resolution;
- integration tests должны выполняться только в headless path, без host
  `zellij` пользователя.
