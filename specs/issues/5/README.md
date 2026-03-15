# Issue 5: `issue-implementation-flow` как отдельный stage после analysis

Статус: draft
Тип задачи: `feature`
Тип проекта: `infra/platform`
Размер: `medium`
Последнее обновление: 2026-03-14
Статус согласования: approved
Approved By: dapi
Approved At: 2026-03-14T19:14:28+03:00

## Контекст

Issue: `Спроектировать и реализовать issue-implementation-flow как следующий stage после analysis`

- GitHub: https://github.com/dapi/ai-teamlead/issues/5
- Analysis branch: `analysis/issue-5`
- Session UUID: `085e127c-1969-4e80-9c1a-66bc0d32711d`

Сейчас `ai-teamlead` канонически покрывает только analysis stage: issue
попадает в `Backlog`, проходит `issue-analysis-flow`, получает versioned
SDD-комплект и после human gate доходит до `Ready for Implementation`.

После этого у проекта нет отдельного SSOT и нет канонического execution path
для следующего шага:

- как именно стартует реализация из `Ready for Implementation`;
- как implementation stage использует принятые analysis artifacts;
- какой branch/worktree lifecycle считается правильным для кода;
- какой контракт обязателен для commit, push, PR и quality gates;
- как не смешивать analysis flow и implementation flow в одну размытую схему.

Цель анализа: зафиксировать минимальный, но достаточный контракт отдельного
`issue-implementation-flow`, который можно будет реализовывать без архитектурной
каши и без скрытых решений в коде.

## Approval

Пакет анализа считается approved только после двух событий одновременно:

- владелец репозитория или явно назначенный reviewer явно подтверждает план в
  агентской сессии;
- `ai-teamlead` переводит issue из `Waiting for Plan Review` в
  `Ready for Implementation`.

В этот же момент пакет должен менять `Статус согласования` с `pending human review`
на `approved` и фиксировать:

- `Approved By`
- `Approved At`

До этого момента все документы комплекта остаются draft и не считаются
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
- [../../../docs/features/0001-ai-teamlead-cli/README.md](../../../docs/features/0001-ai-teamlead-cli/README.md)
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
- [../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md](../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md)
- [../../../docs/adr/0015-versioned-launch-agent-contract.md](../../../docs/adr/0015-versioned-launch-agent-contract.md)
- [../../../docs/adr/0016-configurable-analysis-workspace-templates.md](../../../docs/adr/0016-configurable-analysis-workspace-templates.md)
- [../../../docs/adr/0017-minimal-sdd-artifact-set-for-issue-analysis.md](../../../docs/adr/0017-minimal-sdd-artifact-set-for-issue-analysis.md)
- [../../../docs/adr/0020-agent-session-completion-signal.md](../../../docs/adr/0020-agent-session-completion-signal.md)

## Вывод анализа

Информации в issue достаточно, чтобы готовить план реализации без
дополнительных вопросов пользователю.

Предлагаемый контракт:

- implementation stage оформляется как отдельный SSOT и отдельная feature, но
  issue-level CLI entrypoint остается единым;
- оператор по-прежнему вызывает `run <issue>`, а система сама определяет
  текущую стадию issue и выбирает нужный flow;
- approved analysis artifacts становятся обязательным входным контрактом для
  реализации;
- implementation branch/worktree lifecycle отделяется от analysis branch;
- commit/push/PR/status transitions для implementation stage инкапсулируются в
  stage-aware CLI-контракт, а не размазываются по prompt;
- quality gates включают минимум локальные тесты, draft PR и явную проверку
  CI/review-перехода.

Блокирующих вопросов по текущему issue не выявлено.
