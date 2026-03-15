# Issue 11: `run` и понятные ошибки для issue вне проекта, закрытых issue и недопустимых entry status

Статус: draft
Тип задачи: bug
Размер: medium
Последнее обновление: 2026-03-15

## Контекст

Issue: `run: улучшить обработку issue вне проекта или с неожиданным статусом`

- GitHub: https://github.com/dapi/ai-teamlead/issues/11
- Analysis branch: `analysis/issue-11`
- Session UUID: `01550517-3359-437a-b827-527b1afe2baa`

Сейчас `ai-teamlead run <issue>` плохо различает несколько операторских
ошибочных ситуаций:

1. issue не входит в целевой GitHub Project;
2. issue существует, но закрыта;
3. issue находится в проекте, но ее текущий статус не подходит для запуска;
4. пользователь передал номер или URL несуществующей issue.

На практике оператор получает короткие технические сообщения вроде
`issue #42 is not linked to the project` или
`status is not a valid run entry point`, но не понимает:

- какой именно проект ожидается;
- какой статус у issue сейчас;
- какие статусы допустимы для повторного `run`;
- что нужно сделать руками, чтобы перейти к следующему шагу.

Отдельно зафиксирован конфликт между issue и текущим project-specific
контрактом:

- в issue приведен пример, где статус `Ready for Implementation` считается
  недопустимым для `run`;
- это больше не соответствует accepted ADR
  [../../../docs/adr/0024-stage-aware-run-dispatch.md](../../../docs/adr/0024-stage-aware-run-dispatch.md)
  от `2026-03-14`, по которому `run <issue>` является stage-aware entrypoint и
  должен принимать `Ready for Implementation` как вход в implementation stage.

Следовательно, реализация этой задачи не должна возвращать старое поведение
"analysis-only run". Она должна улучшить диагностику, сохранив stage-aware
контракт.

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

- [../../../README.md](../../../README.md)
- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/issue-implementation-flow.md](../../../docs/issue-implementation-flow.md)
- [../../../docs/implementation-plan.md](../../../docs/implementation-plan.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/features/0001-ai-teamlead-cli/README.md](../../../docs/features/0001-ai-teamlead-cli/README.md)
- [../../../docs/features/0004-issue-implementation-flow/README.md](../../../docs/features/0004-issue-implementation-flow/README.md)
- [../../../docs/adr/0003-github-project-status-as-source-of-truth.md](../../../docs/adr/0003-github-project-status-as-source-of-truth.md)
- [../../../docs/adr/0006-use-gh-cli-as-github-integration-layer.md](../../../docs/adr/0006-use-gh-cli-as-github-integration-layer.md)
- [../../../docs/adr/0024-stage-aware-run-dispatch.md](../../../docs/adr/0024-stage-aware-run-dispatch.md)

## Вывод анализа

Информации в issue достаточно, чтобы готовить план реализации без
дополнительных вопросов пользователю.

План может идти в `Waiting for Plan Review`, если реализация зафиксирует
следующий контракт:

- `run <issue>` различает минимум четыре operator-facing отказных ветки:
  `issue not found`, `issue closed`, `issue not in target project`,
  `issue status is not a valid run entry point`;
- сообщение для `issue not in target project` содержит номер issue, configured
  `github.project_id`, по возможности human-readable имя проекта из snapshot,
  URL issue и короткую командную подсказку, как добавить issue в проект;
- сообщение для недопустимого статуса содержит текущий статус issue и полный
  список допустимых статусов для stage-aware `run`, а не только analysis-stage
  статусы;
- доменный слой формирует user-facing сообщение детерминированно и отдельно от
  orchestration-кода, чтобы его можно было покрыть unit-тестами;
- `run_manual_run` перестает полагаться только на `ProjectSnapshot` для
  диагностики и получает отдельный lookup-path для issue вне проекта или вне
  состояния `OPEN`.

## Открытые вопросы

Блокирующих вопросов по текущему issue не выявлено.
