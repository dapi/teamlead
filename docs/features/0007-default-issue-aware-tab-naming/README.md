# Feature 0007: default issue-aware tab naming

Статус: draft
Владелец: владелец репозитория
Последнее обновление: 2026-03-15

## Контекст

Текущий launcher contract уже умеет давать issue-aware имя вкладке в
`zellij`, но только как opt-in через `zellij.tab_name_template`.

Это создает разрыв между ожиданием оператора и фактическим default behavior:

- в режиме `launch_target = tab` пользователь ожидает видеть вкладку вида
  `#42`;
- фактический runtime default оставляет вкладку `issue-analysis`;
- `issue-analysis` семантически полезен как shared tab только для `pane`
  режима, а не как default имя отдельной issue-specific вкладки.

Эта feature выносит follow-up change set в отдельный contract layer и задает
новый default для tab-launch path.

## Что строим

- [01-what-we-build.md](./01-what-we-build.md)

## Как строим

- [02-how-we-build.md](./02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## План реализации

- [04-implementation-plan.md](./04-implementation-plan.md)

## Связанные документы

- [README.md](../../../README.md)
- [docs/config.md](../../config.md)
- [docs/code-quality.md](../../code-quality.md)
- [docs/issue-analysis-flow.md](../../issue-analysis-flow.md)
- [docs/features/0002-repo-init/README.md](../0002-repo-init/README.md)
- [docs/features/0003-agent-launch-orchestration/README.md](../0003-agent-launch-orchestration/README.md)
- [docs/adr/0031-zellij-issue-aware-tab-name-template.md](../../adr/0031-zellij-issue-aware-tab-name-template.md)
- [docs/adr/0032-zellij-launch-target-pane-tab.md](../../adr/0032-zellij-launch-target-pane-tab.md)

## Открытые вопросы

- нужен ли отдельный CLI-visible diagnostics hint, объясняющий distinction
  между `pane` и `tab` naming semantics.

## Журнал изменений

### 2026-03-15

- создана feature 0007 для смены default tab naming в `tab`-режиме
- выбран путь через canonical application default-layer в `Config`, а не через
  ad-hoc runtime-only fallback
