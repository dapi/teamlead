# Issue 14: `poll` фильтрует backlog по assignee

## Issue

- GitHub issue: https://github.com/dapi/ai-teamlead/issues/14
- Тип: `feature`
- Размер: `medium`
- Тип проекта: `infra/platform`

## Summary

Issue добавляет в repo-local config опциональный фильтр
`poll.assignee_filter`, который ограничивает выбор backlog-issue для `poll` и
`loop` задачами, заасайненными на конкретного GitHub-пользователя.

Фильтр должен поддерживать три режима:

- отсутствие значения: текущее поведение без ограничений;
- `"$me"`: выбор только issue текущего GitHub-пользователя;
- `"username"`: выбор issue конкретного пользователя.

Для этого нужно расширить snapshot GitHub Project полем `assignees`,
добавить runtime-resolve текущего пользователя через `gh api user` и сохранить
инвариант: ручной `run` по-прежнему работает для любой issue независимо от
assignee.

## Status

Черновик анализа готов к human review и переводу issue в
`Waiting for Plan Review`.

## Artifacts

- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

## Related Context

- [../../../README.md](../../../README.md)
- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/features/0001-ai-teamlead-cli/README.md](../../../docs/features/0001-ai-teamlead-cli/README.md)
- [../../../docs/features/0002-repo-init/README.md](../../../docs/features/0002-repo-init/README.md)
- [../../../docs/adr/0001-repo-local-ai-config.md](../../../docs/adr/0001-repo-local-ai-config.md)
- [../../../docs/adr/0009-deterministic-backlog-ordering.md](../../../docs/adr/0009-deterministic-backlog-ordering.md)
- [../../../docs/adr/0021-cli-contract-poll-run-loop.md](../../../docs/adr/0021-cli-contract-poll-run-loop.md)
- [../../../docs/adr/0027-zero-config-settings-template-and-runtime-default-layer.md](../../../docs/adr/0027-zero-config-settings-template-and-runtime-default-layer.md)

## Open Questions

Блокирующих вопросов по текущему issue не выявлено.
