# Issue 14: `poll` фильтрует backlog по assignee

Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-15T17:20:01+03:00

## Issue

- GitHub issue: https://github.com/dapi/ai-teamlead/issues/14
- Тип: `feature`
- Размер: `medium`
- Тип проекта: `infra/platform`

## Summary

Issue добавляет optional-настройку `poll.assignee_filter` в repo-local
`settings.yml`, чтобы `poll` мог отбирать backlog-issue не только по
репозиторию и статусу, но и по assignee.

Контракт задачи ограничен исходным issue:

- если `poll.assignee_filter` не задан, `poll` сохраняет текущее поведение и не
  фильтрует backlog по assignee;
- если значение равно `"$me"`, eligible считаются только issue, назначенные на
  текущего GitHub-пользователя;
- если значение равно `"username"`, eligible считаются только issue,
  назначенные на указанного пользователя;
- `"$me"` резолвится через `gh api user --jq '.login'` один раз на старте
  процесса и затем переиспользуется в течение жизни `poll`/`loop`;
- ручной `run` не меняет поведение и не зависит от `assignee_filter`.

Для этого нужно расширить GitHub Project snapshot списком assignee login-ов,
добавить runtime-resolve текущего пользователя в `app/github` слое и передавать
в `domain` уже зарезолвленный `Option<&str>`.

## Status

Analysis-пакет выровнен с исходным scope issue и готов к human review.

## Artifacts

- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

## Related Context

- [../../../README.md](../../../README.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/features/0001-ai-teamlead-cli/README.md](../../../docs/features/0001-ai-teamlead-cli/README.md)
- [../../../docs/features/0002-repo-init/README.md](../../../docs/features/0002-repo-init/README.md)
- [../../../docs/adr/0001-repo-local-ai-config.md](../../../docs/adr/0001-repo-local-ai-config.md)
- [../../../docs/adr/0009-deterministic-backlog-ordering.md](../../../docs/adr/0009-deterministic-backlog-ordering.md)
- [../../../docs/adr/0021-cli-contract-poll-run-loop.md](../../../docs/adr/0021-cli-contract-poll-run-loop.md)
- [../../../docs/adr/0033-zero-config-settings-template-and-runtime-default-layer.md](../../../docs/adr/0033-zero-config-settings-template-and-runtime-default-layer.md)

## Open Questions

Блокирующих вопросов по этой задаче не осталось.
