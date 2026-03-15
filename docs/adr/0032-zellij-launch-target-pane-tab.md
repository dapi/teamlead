# ADR-0032: `zellij.launch_target` и CLI override для `pane` / `tab`

Статус: accepted
Дата: 2026-03-15

## Контекст

После ADR-0023 и ADR-0027 у `ai-teamlead` уже были:

- versioned fallback для target session через `zellij.session_name`;
- stable shared имя tab через `zellij.tab_name`;
- optional issue-aware `zellij.tab_name_template` для tab-launch path.

Но launcher по-прежнему не фиксировал явный операторский контракт для выбора
между двумя режимами запуска внутри выбранной `zellij` session:

- открыть новую pane в общей shared tab;
- открыть отдельный analysis tab.

Из-за этого repo owner не мог задать versioned default mode, а оператор не мог
одноразово переопределить поведение `run` без правки `settings.yml`.

## Решение

В versioned config contract добавляется поле:

```yaml
zellij:
  launch_target: "tab"
```

Поддерживаются только значения:

- `pane`
- `tab`

Правила:

1. runtime default при отсутствии поля = `tab`;
2. public CLI override добавляется только для `run`:
   `ai-teamlead run <issue> --launch-target <pane|tab>`;
3. precedence order фиксируется как:
   `run --launch-target` -> `zellij.launch_target` -> runtime default `tab`;
4. `poll` и `loop` остаются config-driven и не получают отдельный public
   `--launch-target` override;
5. `pane`-режим использует stable shared tab `zellij.tab_name`:
   - при единственном совпадении переиспользует его и открывает новую pane;
   - при отсутствии shared tab создает его через versioned
     `.ai-teamlead/zellij/analysis-tab.kdl`;
   - при нескольких совпадениях завершает запуск ошибкой;
6. `tab`-режим сохраняет existing behavior создания отдельной analysis tab;
7. `zellij.tab_name_template` влияет только на `tab`-режим и не применяется к
   `pane`-ветке.

## Последствия

Плюсы:

- repo получает versioned default launcher mode;
- оператор получает одноразовый override без мутации config;
- `pane` и `tab` semantics становятся явно различимыми в docs, diagnostics и
  runtime metadata;
- `poll` и `loop` сохраняют детерминированный config-driven behavior.

Минусы:

- launcher orchestration становится ветвящимся по mode;
- появляется новый runtime guard на duplicate shared tabs;
- diagnostics и integration coverage должны различать оба launcher path.

## Связанные документы

- [ADR-0023](./0023-zellij-session-target-resolution.md)
- [ADR-0031](./0031-zellij-issue-aware-tab-name-template.md)
- [Feature 0003](../features/0003-agent-launch-orchestration/README.md)
- [Issue Analysis Flow](../issue-analysis-flow.md)
- [specs/issues/47/README.md](../../specs/issues/47/README.md)

## Журнал изменений

### 2026-03-15

- добавлен config contract `zellij.launch_target`
- зафиксирован runtime default `tab`
- добавлен public CLI override `run --launch-target`
- зафиксировано решение не добавлять public override в `poll` и `loop`
