# Issue 14: `poll` фильтрует backlog по assignee

## Issue

- GitHub issue: https://github.com/dapi/ai-teamlead/issues/14
- Тип: `feature`
- Размер: `medium`
- Тип проекта: `infra/platform`

## Summary

Issue добавляет в repo-local config настройку `poll.assignee_filter`, которая
по умолчанию работает как `"$me"` и ограничивает выбор backlog-issue для
`poll` и `loop` задачами текущего GitHub-пользователя.

Контракт должен поддерживать как минимум четыре режима:

- отсутствие значения: effective default `"$me"`;
- `"$me"`: выбор только issue текущего GitHub-пользователя;
- `"$all"`: явное отключение фильтра по assignee;
- `"$unassigned"`: выбор только issue без assignee;
- `"username"`: выбор issue конкретного пользователя.

Для этого нужно расширить snapshot GitHub Project полем `assignees`,
добавить runtime-resolve текущего пользователя через `gh api user` и изменить
контракт ручного `run`: при несоответствии effective filter команда выводит
warning и просит approve пользователя, а `--force` оставляет warning, но
обходит approve.

Текущее развитие задачи заблокировано issue `#11`, потому что новая семантика
`run` должна строиться поверх более общего слоя user-friendly диагностики и
operator interaction для `run`.

## Status

Черновик анализа обновлен по review, но дальнейшая разработка заблокирована
issue `#11` до ее реализации.

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
- https://github.com/dapi/ai-teamlead/issues/11

## Open Questions

Блокирующих вопросов по продуктовой формулировке не осталось, но реализация
должна ждать завершения issue `#11`.
