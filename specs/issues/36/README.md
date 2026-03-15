# Issue 36: `launch-agent` per-agent global args для `claude` и `codex`

## Issue

- GitHub issue: https://github.com/dapi/ai-teamlead/issues/36
- Тип: `feature`
- Размер: `medium`
- Тип проекта: `infra/platform`

## Summary

Issue добавляет в repo-local config явный контракт для global args, которые
применяются только к конкретному агенту:

- отдельный список аргументов для `claude`
- отдельный список аргументов для `codex`
- осмысленные runtime defaults для достаточно автономного запуска

Bootstrap-шаблон `templates/init/settings.yml` должен показывать оба примера в
активном default-layer и отдельно показывать более агрессивные opt-in примеры.
Launcher должен передавать args только в ветку реально запускаемого агента и не
ломать degraded mode fallback.

## Status

Черновик анализа готов к human review и переводу issue в
`Waiting for Plan Review`.

## Artifacts

- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

## Related Context

- [../../../specs/issues/12/README.md](../../../specs/issues/12/README.md)
- [../../../docs/features/0002-repo-init/README.md](../../../docs/features/0002-repo-init/README.md)
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
- [../../../docs/adr/0015-versioned-launch-agent-contract.md](../../../docs/adr/0015-versioned-launch-agent-contract.md)
- [../../../docs/adr/0016-configurable-analysis-workspace-templates.md](../../../docs/adr/0016-configurable-analysis-workspace-templates.md)

## Open Questions

Блокирующих вопросов по текущему issue не выявлено.
