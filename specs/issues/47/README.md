# Issue 47: `run`: configurable launch target defaults and CLI override for `pane`/`tab`

Статус: draft
Тип задачи: `feature`
Тип проекта: `infra/platform`
Размер: `medium`
Последнее обновление: 2026-03-15
Статус согласования: pending human review

## Issue

- GitHub issue: https://github.com/dapi/ai-teamlead/issues/47
- Тип: `feature`
- Размер: `medium`
- Тип проекта: `infra/platform`

## Summary

Issue вводит явный launcher contract для выбора analysis launch target внутри
`zellij`:

- versioned repo-level default через `zellij.launch_target`;
- per-run override через `ai-teamlead run <issue> --launch-target <pane|tab>`;
- precedence order `CLI override -> settings.yml -> runtime default`;
- runtime default = `pane`, если поле не задано;
- `pane`-режим обязан переиспользовать stable shared tab и не создавать
  duplicate tab молча;
- `tab`-режим сохраняет текущее поведение открытия отдельной analysis tab;
- `poll` и `loop` остаются config-driven и не получают отдельный public
  `--launch-target` override.

## Status

Черновик анализа готов к human review и переводу issue в
`Waiting for Plan Review`.

## Artifacts

- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [04-implementation-plan.md](./04-implementation-plan.md)

## Related Context

- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/implementation-plan.md](../../../docs/implementation-plan.md)
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
- [../../../docs/adr/0005-cli-contract-for-poll-and-run.md](../../../docs/adr/0005-cli-contract-for-poll-and-run.md)
- [../../../docs/adr/0015-versioned-launch-agent-contract.md](../../../docs/adr/0015-versioned-launch-agent-contract.md)
- [../../../docs/adr/0023-zellij-session-target-resolution.md](../../../docs/adr/0023-zellij-session-target-resolution.md)
- [../49/README.md](../49/README.md)

## Open Questions

Блокирующих вопросов по текущему issue не выявлено.

Решение, зафиксированное этим analysis-комплектом:

- public override по `launch_target` добавляется только в `run`;
- `poll` и `loop` используют repo-level default из конфига или встроенный
  runtime default, без отдельного операторского флага;
- `tab`-ветка должна использовать уже существующий effective tab naming
  contract, если в репозитории принят `zellij.tab_name_template`.
