# ADR-0031: `zellij.tab_name_template` для issue-aware tab naming

Статус: accepted
Дата: 2026-03-14

## Контекст

`ai-teamlead` уже использует `zellij.tab_name` как versioned имя analysis tab.

Этого достаточно для stable semantic context, но недостаточно для сценария,
где analysis tab должен явно указывать на конкретную issue, например `#42`.

При этом нельзя просто превратить `zellij.tab_name` в template:

- это смешает stable semantic имя и issue-aware runtime naming;
- future `pane`-path должен сохранить shared tab context;
- runtime metadata и diagnostics начнут дрейфовать, если effective имя tab
  будет вычисляться только внутри generated layout.

## Решение

В `zellij` config contract добавляется optional поле:

```yaml
zellij:
  tab_name: "issue-analysis"
  tab_name_template: "#${ISSUE_NUMBER}"
```

Правила:

- `zellij.tab_name` остается stable semantic fallback-именем;
- `zellij.tab_name_template` используется только как issue-aware naming source
  для tab-launch path;
- `tab_name_template` поддерживает только `${ISSUE_NUMBER}`;
- literal строка без placeholders тоже допустима;
- если `tab_name_template` отсутствует, runtime использует `zellij.tab_name`;
- effective tab name вычисляется до генерации `launch-layout.kdl`;
- runtime manifest, launch log и operator-facing diagnostics используют уже
  resolved tab name, а не raw config template.

Bootstrap:

- `templates/init/settings.yml` и repo-local `settings.yml` показывают
  commented example `#${ISSUE_NUMBER}`, но не включают поле по умолчанию.

## Последствия

Плюсы:

- появляется versioned и явный контракт для issue-aware tab naming;
- stable `tab_name` не смешивается с template semantics;
- launcher diagnostics и runtime metadata остаются согласованными;
- change set остается локальным и совместимым с будущим `pane/tab` dispatch.

Минусы:

- у `zellij` config появляется еще одно optional поле;
- runtime получает отдельную policy-check логику для placeholder validation;
- до появления полного `pane/tab` dispatch часть смысла контракта остается
  подготовкой к следующему launcher split.

## Связанные документы

- [ADR-0014](./0014-zellij-launch-context-naming.md)
- [ADR-0023](./0023-zellij-session-target-resolution.md)
- [README.md](../../README.md)
- [Feature 0002](../features/0002-repo-init/README.md)
- [Feature 0003](../features/0003-agent-launch-orchestration/README.md)
- [specs/issues/49/README.md](../../specs/issues/49/README.md)

## Журнал изменений

### 2026-03-14

- добавлен optional config contract `zellij.tab_name_template`
- зафиксирован placeholder policy: только `${ISSUE_NUMBER}`
- зафиксировано требование пробрасывать effective tab name в runtime metadata и
  diagnostics
