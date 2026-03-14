# Issue 51: определить post-merge lifecycle после merge implementation PR

Статус: draft
Тип задачи: `feature`
Тип проекта: `infra/platform`
Размер: `medium`
Последнее обновление: 2026-03-14
Статус согласования: pending human review

## Контекст

Issue: `Определить post-merge lifecycle после merge implementation PR`

- GitHub: https://github.com/dapi/ai-teamlead/issues/51
- Analysis branch: `analysis/issue-51`
- Session UUID: `a4a13c67-9fa3-42d4-ae69-f74192c47b5d`

После merge PR [#48](https://github.com/dapi/ai-teamlead/pull/48) проект уже
умеет доводить задачу до `Waiting for Code Review`, но дальше lifecycle
обрывается:

- `docs/issue-implementation-flow.md` явно оставляет merge automation и
  post-merge path вне scope;
- issue может остаться `OPEN` после merge implementation PR;
- GitHub Project status не получает канонического terminal state;
- не определено, как очищать implementation runtime/worktree/branches;
- не отделен минимальный post-merge contract от будущих release/deploy flow.

Цель анализа: зафиксировать минимальный, но достаточный контракт post-merge
lifecycle без ввода лишнего третьего flow для MVP.

## Approval

Пакет анализа считается approved только после двух событий одновременно:

- владелец репозитория или явно назначенный reviewer подтверждает план в
  агентской сессии;
- issue переводится из `Waiting for Plan Review` в `Ready for Implementation`.

В этот момент пакет должен поменять `Статус согласования` с
`pending human review` на `approved` и зафиксировать:

- `Approved By`
- `Approved At`

До этого момента документы комплекта остаются draft и не считаются
утвержденным входом для implementation stage.

## Артефакты

## Что строим

- [01-what-we-build.md](./01-what-we-build.md)

## Как строим

- [02-how-we-build.md](./02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## План имплементации

- [04-implementation-plan.md](./04-implementation-plan.md)

## Связанный контекст

- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/issue-implementation-flow.md](../../../docs/issue-implementation-flow.md)
- [../../../docs/implementation-plan.md](../../../docs/implementation-plan.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/features/0004-issue-implementation-flow/README.md](../../../docs/features/0004-issue-implementation-flow/README.md)
- [../../../docs/adr/0024-stage-aware-run-dispatch.md](../../../docs/adr/0024-stage-aware-run-dispatch.md)
- [../../../docs/adr/0025-stage-aware-runtime-bindings.md](../../../docs/adr/0025-stage-aware-runtime-bindings.md)
- [../../../docs/adr/0026-stage-aware-complete-stage.md](../../../docs/adr/0026-stage-aware-complete-stage.md)
- [../5/README.md](../5/README.md)

## Вывод анализа

Информации в issue достаточно, чтобы готовить план реализации без
дополнительных вопросов пользователю.

Предлагаемый контракт:

- post-merge lifecycle остается частью `issue-implementation-flow`, а не
  выделяется в отдельный третий flow для MVP;
- merge tracked implementation PR становится явным trigger для terminal
  finalization;
- после успешной post-merge finalization issue закрывается, а GitHub Project
  status переводится в `Done`;
- cleanup implementation runtime/worktree/local branch выполняется как
  idempotent best-effort path и не должен откатывать terminal business result;
- release, deploy и другие post-merge operation flow остаются отдельным
  возможным follow-up, но не входят в текущий scope.

Блокирующих вопросов по текущему issue не выявлено.
