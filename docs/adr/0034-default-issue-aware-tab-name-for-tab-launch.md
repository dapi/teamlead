# ADR-0034: default issue-aware tab name for `launch_target = tab`

Статус: accepted
Дата: 2026-03-15

## Контекст

После ADR-0031 и ADR-0032 launcher уже умеет строить issue-aware имя вкладки
через `zellij.tab_name_template`, но только как opt-in behavior.

Фактический default contract выглядит так:

- `launch_target` по умолчанию равен `tab`;
- `zellij.tab_name` по умолчанию равен `issue-analysis`;
- если `tab_name_template` отсутствует, runtime использует `issue-analysis`.

Это создает UX-разрыв:

- отдельная tab issue выглядит как общий технический context, а не как рабочая
  вкладка конкретной задачи;
- оператор не видит номер issue без ручной донастройки `settings.yml`;
- stable shared tab semantics для `pane` режима протекают в `tab`-режим, хотя
  у этих веток разная цель.

## Решение

Для `launch_target = tab` issue-aware имя вкладки становится application
default.

Новый contract:

```yaml
zellij:
  tab_name: "issue-analysis"
  launch_target: "tab"
  tab_name_template: "#${ISSUE_NUMBER}"
```

Правила:

1. `zellij.tab_name` сохраняет роль stable shared tab name для `pane`-режима.
2. `zellij.tab_name_template` остается naming source для `tab`-режима.
3. Если active YAML не задает `zellij.tab_name_template`, runtime default для
   этого поля считается `#${ISSUE_NUMBER}`.
4. Literal override, например `issue-analysis`, остается допустимым explicit
   opt-out для `tab`-режима.
5. Placeholder policy не расширяется: поддерживается только
   `${ISSUE_NUMBER}`.
6. Runtime manifest, launch log и operator-facing diagnostics продолжают
   хранить уже resolved `tab_name`.

## Последствия

Плюсы:

- default behavior соответствует операторскому ожиданию для issue-specific tab;
- shared `pane` tab и отдельная `tab` issue получают разные naming semantics;
- zero-config template и runtime default снова совпадают;
- opt-out path остается простым и versioned.

Минусы:

- меняется application default для существующих repo-local configs без active
  `tab_name_template`;
- часть документации и тестов, завязанных на fallback `issue-analysis` в
  `tab`-ветке, нужно синхронно обновить;
- старые issue-level analysis artifacts по `#49` остаются историческими и
  частично superseded новым contract.

## Supersedes

Этот ADR supersede-ит только следующие части ранее принятых решений:

- в [ADR-0031](./0031-zellij-issue-aware-tab-name-template.md) пункт
  `если tab_name_template отсутствует, runtime использует zellij.tab_name`;
- в [ADR-0032](./0032-zellij-launch-target-pane-tab.md) implicit assumption,
  что `tab`-ветка без active template наследует stable `tab_name`.

Все остальные части ADR-0031 и ADR-0032 остаются в силе.

## Связанные документы

- [ADR-0031](./0031-zellij-issue-aware-tab-name-template.md)
- [ADR-0032](./0032-zellij-launch-target-pane-tab.md)
- [ADR-0033](./0033-zero-config-settings-template-and-runtime-default-layer.md)
- [docs/config.md](../config.md)
- [Feature 0002](../features/0002-repo-init/README.md)
- [Feature 0003](../features/0003-agent-launch-orchestration/README.md)
- [Feature 0007](../features/0007-default-issue-aware-tab-naming/README.md)

## Журнал изменений

### 2026-03-15

- issue-aware имя вкладки переведено из opt-in semantics в runtime default для
  `launch_target = tab`
