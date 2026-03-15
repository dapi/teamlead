# Feature 0007: Как строим

## Архитектура

Feature расширяет существующий launcher contract из Feature 0003, но не
заменяет его.

Слои изменения:

1. documentation layer
   - новый ADR, который supersede-ит opt-in semantics из ADR-0031/0032;
   - config docs и feature-docs получают новый default contract;
2. config/runtime resolution
   - tab-launch path получает default naming source `#${ISSUE_NUMBER}`;
   - pane-launch path продолжает использовать `zellij.tab_name`;
3. bootstrap and user guidance
   - templates и summary-документы перестают обещать `issue-analysis` как
     default имя issue-specific tab.

## Данные и состояния

Существенные runtime сущности не меняются:

- `zellij.launch_target`
- `zellij.tab_name`
- `zellij.tab_name_template`
- resolved `tab_name` в runtime manifest

Меняется только семантика default resolution для `tab`-режима.

Новый expected contract:

- `pane`
  - effective tab name = `zellij.tab_name`
- `tab`
  - если `zellij.tab_name_template` явно задан, используется он;
  - если поле не задано в active YAML, application default равен
    `#${ISSUE_NUMBER}`;
  - fallback на stable `zellij.tab_name` больше не является default path.

## Интерфейсы

Затрагиваемые интерфейсы:

- `./.ai-teamlead/settings.yml`
- `templates/init/settings.yml`
- `src/config.rs`
- `src/app.rs`
- `src/runtime.rs`
- operator-facing docs в `docs/config.md` и Feature 0003

Наружный CLI contract не меняется: новые флаги не добавляются.

## Технические решения

Предпочтительное направление:

- оставить `tab_name_template` optional в active YAML, но перевести его в
  canonical defaulted-by-application поле;
- explicit literal override вроде `issue-analysis` считать валидным opt-out;
- runtime manifests хранить только resolved имя вкладки.

Причина такого выбора:

- active config не засоряется обязательным полем ради нового default;
- behavior меняется predictably даже для старых repo-local configs;
- distinction между stable pane tab и issue-specific tab path остается явным.

## Конфигурация

После изменения contract будет таким:

```yaml
zellij:
  tab_name: "issue-analysis"
  launch_target: "tab"
  tab_name_template: "#${ISSUE_NUMBER}"
```

Семантика:

- `tab_name` больше не описывает default tab title для issue-specific tab path;
- `tab_name_template` становится canonical naming policy для `tab`-ветки:
  defaulted-by-application поле с возможностью explicit override;
- placeholder set остается прежним: только `${ISSUE_NUMBER}`.

## Ограничения реализации

- нужно явно разрешить backward-compatible opt-out;
- документация не должна одновременно обещать оба default behavior;
- если меняется application default, regression tests на legacy fallback
  придется переписать, а не просто расширить.
