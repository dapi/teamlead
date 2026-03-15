# Issue 11: План имплементации

Статус: draft
Последнее обновление: 2026-03-15

## Назначение

Этот план задает порядок реализации понятных operator-facing ошибок для
`ai-teamlead run <issue>` в ситуациях `issue not found`, `issue closed`,
`issue not in target project` и `status denied`.

## Scope

В scope входит:

- добавить repo-level lookup для одной issue вне `ProjectSnapshot`;
- ввести структурированную доменную модель отказа для ручного `run`;
- реализовать канонический formatter user-facing сообщений;
- обновить `run_manual_run` под новый decision path;
- добавить unit и integration tests;
- сохранить stage-aware behavior для implementation entry statuses.

Вне scope:

- изменение allowed statuses или status transitions;
- автоматическое изменение project membership из CLI;
- изменение публичного синтаксиса команды `run`;
- новый ADR;
- полная система локализации всего CLI.

## Связанные документы

- [README.md](./README.md)
- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/issue-implementation-flow.md](../../../docs/issue-implementation-flow.md)
- [../../../docs/implementation-plan.md](../../../docs/implementation-plan.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/features/0001-ai-teamlead-cli/README.md](../../../docs/features/0001-ai-teamlead-cli/README.md)
- [../../../docs/features/0004-issue-implementation-flow/README.md](../../../docs/features/0004-issue-implementation-flow/README.md)
- [../../../docs/adr/0003-github-project-status-as-source-of-truth.md](../../../docs/adr/0003-github-project-status-as-source-of-truth.md)
- [../../../docs/adr/0006-use-gh-cli-as-github-integration-layer.md](../../../docs/adr/0006-use-gh-cli-as-github-integration-layer.md)
- [../../../docs/adr/0024-stage-aware-run-dispatch.md](../../../docs/adr/0024-stage-aware-run-dispatch.md)

## Зависимости и предпосылки

- текущий `run` уже является stage-aware dispatcher и это поведение нельзя
  сломать;
- `gh`-adapter слой остается канонической интеграцией с GitHub;
- `ProjectSnapshot` сам по себе не покрывает случаи `not found` и `closed`,
  поэтому нужен отдельный lookup-path;
- configured `project_id` не дает готовый numeric project number для
  `gh project item-add`, поэтому подсказку нужно проектировать честно:
  placeholder или явный дополнительный lookup;
- качество изменения подтверждается не только unit-тестами formatter-а, но и
  хотя бы одним integration сценарием реального CLI-вывода.

## Порядок работ

### Этап 1. Repo-level lookup issue

Цель:

- добавить в GitHub adapter минимальный lookup одной issue по owner/repo и
  номеру.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [../../../docs/adr/0006-use-gh-cli-as-github-integration-layer.md](../../../docs/adr/0006-use-gh-cli-as-github-integration-layer.md)

Результат этапа:

- adapter умеет возвращать минимум `number`, `state`, `url` или явный
  `not found`;
- `run_manual_run` получает данные, которых не было в `ProjectSnapshot`.

Проверка:

- unit-тесты adapter parsing;
- при необходимости fake-shell тест на `not found` и `closed`.

### Этап 2. Domain-решение и formatter

Цель:

- ввести структурированный отказной контракт для ручного `run`.

Основание:

- [01-what-we-build.md](./01-what-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- domain-слой различает `IssueNotFound`, `IssueClosed`, `IssueNotInProject`,
  `StatusDenied`;
- formatter строит user-facing сообщение из структурированных данных;
- `allowed_statuses` выводятся из канонической dispatch-логики.

Проверка:

- unit-тесты formatter-а и списка допустимых статусов;
- отдельная регрессия на `Ready for Implementation`.

### Этап 3. Wiring в `run_manual_run`

Цель:

- перестроить orchestration ручного `run` на новый decision path.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [../../../docs/features/0001-ai-teamlead-cli/README.md](../../../docs/features/0001-ai-teamlead-cli/README.md)

Результат этапа:

- ранний blanket-error `issue is not linked to the project` убран;
- `run_manual_run` пошагово различает `not found`, `closed`,
  `not in project`, `status denied`;
- успешные ветки analysis и implementation работают как раньше.

Проверка:

- unit-тесты decision flow, если выделена отдельная функция;
- integration test на отказной CLI-path.

### Этап 4. Verification и регрессии

Цель:

- подтвердить, что UX ошибок улучшился без поломки stage-aware dispatch.

Основание:

- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)

Результат этапа:

- `cargo test` проходит;
- есть integration coverage на новый operator-facing текст;
- старые успешные stage-aware сценарии не ломаются.

Проверка:

- `cargo test`;
- целевой integration scenario для `run`;
- при возможности ручной smoke на issue вне project или с недопустимым
  статусом.

## Критерий завершения

Issue можно считать реализованной, если:

- оператор по сообщению понимает, какая именно из четырех отказных веток
  сработала;
- `project_id`, issue URL, current status и allowed statuses выводятся там, где
  это нужно для ручного исправления;
- `Ready for Implementation` и остальные implementation entry statuses не
  ошибочно попадают в deny-path;
- новый formatter покрыт unit-тестами;
- есть integration test, подтверждающий реальный CLI-вывод.

## Открытые вопросы и риски

- нужно не допустить drift между dispatch-логикой и отдельно собранным списком
  allowed statuses;
- важно не сделать integration assertions слишком хрупкими по форматированию;
- если adapter выберет неустойчивый внешний CLI-output вместо структурированного
  JSON, тестируемость и надежность ухудшатся;
- при расширении списка допустимых статусов в будущем formatter должен
  автоматически наследовать новое множество без ручного редактирования строк.

## Журнал изменений

### 2026-03-15

- создан issue-level implementation plan для issue 11
