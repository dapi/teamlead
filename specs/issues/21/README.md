# Issue 21: `run`: auto-normalize issue into project, status and assignee before analysis start

Статус: draft
Тип задачи: `feature`
Тип проекта: `infra/platform`
Размер: `medium`
Последнее обновление: 2026-03-15

## Контекст

Issue: `run: auto-normalize issue into project, status and assignee before analysis start`

- GitHub: https://github.com/dapi/ai-teamlead/issues/21
- Analysis branch: `analysis/issue-21`
- Session UUID: `5f76c5d2-8dc6-4f1c-8fd7-c70648fc1bb1`

В текущем репозитории уже виден частичный auto-heal path для `run`: код умеет
добавлять issue в project и выставлять `Backlog`, если status отсутствует.
Но этот behavior еще не оформлен как явный канонический preflight-contract и
не закрывает policy для assignee, ошибки assignment и обязательную
синхронизацию документации.

Цель анализа: зафиксировать единый contract layer для operator-driven `run`,
в котором GitHub-side preflight normalization происходит до обычной проверки
stage и launcher orchestration.

## Approval

Пакет анализа считается approved только после двух событий одновременно:

- владелец репозитория или явно назначенный reviewer подтверждает план в
  агентской сессии;
- issue переводится из `Waiting for Plan Review` в `Ready for Implementation`.

В этот момент пакет должен поменять `Статус` на `approved` и зафиксировать:

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
- [../../../docs/implementation-plan.md](../../../docs/implementation-plan.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
- [../../../docs/adr/0021-cli-contract-poll-run-loop.md](../../../docs/adr/0021-cli-contract-poll-run-loop.md)
- [../../../docs/adr/0024-stage-aware-run-dispatch.md](../../../docs/adr/0024-stage-aware-run-dispatch.md)
- [../14/README.md](../14/README.md)

## Вывод анализа

Информации в issue и комментариях достаточно, чтобы готовить план реализации
без дополнительных вопросов пользователю.

Предлагаемый контракт:

- `run` получает явный GitHub-side preflight до обычной проверки допустимого
  stage;
- preflight гарантирует: open issue существует в текущем repo, issue
  прикреплена к default project, при отсутствии status установлен `Backlog`,
  при отсутствии assignee issue назначена на текущего пользователя `gh`;
- assignment выполняется только если assignee сейчас отсутствует;
- identity берется только из `gh api user --jq ".login"`, а не из OS-user или
  shell env;
- ошибки `add-to-project`, `set-status`, `resolve-current-user` и `assign`
  считаются фатальными и не допускают тихого продолжения flow;
- `poll` сохраняет строгий behavior и не получает auto-normalization;
- implementation scope должен учитывать, что add-to-project и set-status path
  уже частично есть в коде, поэтому основной delta лежит в явной preflight
  структуре, assignee-path, тестах и documentation/ADR-sync.

Блокирующих вопросов по текущему issue не выявлено.
