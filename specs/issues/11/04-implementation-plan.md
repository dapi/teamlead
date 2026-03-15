# Issue 11: План имплементации

Статус: approved
Последнее обновление: 2026-03-15
Approved By: dapi
Approved At: 2026-03-15T11:52:35+03:00

## Назначение

Этот план задает порядок реализации понятных operator-facing ошибок для
`ai-teamlead run <issue>` в ситуациях `issue not found`, `issue closed`,
`issue not in target project` и `status denied`, с приоритетом
`auto-remediation first`.

## Scope

В scope входит:

- добавить repo-level lookup для одной issue вне `ProjectSnapshot`;
- добавить project mutation path для auto-remediation;
- ввести структурированную remediation-aware доменную модель для ручного `run`;
- реализовать канонический formatter user-facing сообщений;
- обновить `run_manual_run` под новый decision path;
- добавить unit и integration tests;
- сохранить stage-aware behavior для implementation entry statuses.

Вне scope:

- изменение allowed statuses или status transitions;
- изменение публичного синтаксиса команды `run`;
- новый ADR;
- полная система локализации всего CLI;
- автоматическое переоткрытие закрытых issue.

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
- auto-remediation должен использовать канонический GitHub adapter, а не
  shell-snippets с ручными командами для пользователя;
- автоисправление допустимо только при детерминированном выборе target status;
- качество изменения подтверждается не только unit-тестами formatter-а, но и
  хотя бы одним integration сценарием реального CLI-вывода.

## Порядок работ

### Этап 1. Repo-level lookup и mutation prerequisites

Цель:

- добавить в GitHub adapter минимальный lookup одной issue по owner/repo и
  номеру;
- подготовить данные и команды, достаточные для project mutation.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [../../../docs/adr/0006-use-gh-cli-as-github-integration-layer.md](../../../docs/adr/0006-use-gh-cli-as-github-integration-layer.md)

Результат этапа:

- adapter умеет возвращать минимум `id`, `number`, `state`, `url` или явный
  `not found`;
- `run_manual_run` получает данные, которых не было в `ProjectSnapshot`.

Проверка:

- unit-тесты adapter parsing;
- при необходимости fake-shell тест на `not found`, `closed` и project
  mutation payload.

### Этап 2. Domain-решение, remediation-plan и formatter

Цель:

- ввести структурированный remediation-aware контракт для ручного `run`.

Основание:

- [01-what-we-build.md](./01-what-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- domain-слой различает `IssueNotFound`, `IssueClosed`, `AttachToProject`,
  `NormalizeStatus`, `ExplainOnlyStatusDenied`;
- formatter строит user-facing сообщение из структурированных данных;
- `allowed_statuses` выводятся из канонической dispatch-логики;
- remediation-plan определяет, когда система исправляет проблему сама, а когда
  эскалирует на пользователя.

Проверка:

- unit-тесты formatter-а и списка допустимых статусов;
- unit-тесты remediation decision;
- отдельная регрессия на `Ready for Implementation`.

### Этап 3. Wiring и auto-remediation в `run_manual_run`

Цель:

- перестроить orchestration ручного `run` на новый decision path.

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [../../../docs/features/0001-ai-teamlead-cli/README.md](../../../docs/features/0001-ai-teamlead-cli/README.md)

Результат этапа:

- ранний blanket-error `issue is not linked to the project` убран;
- `run_manual_run` пошагово различает `not found`, `closed`,
- `not in project`, `status denied`;
- для `AttachToProject` выполняется автоматическое добавление issue в project и
  установка стартового status;
- для `NormalizeStatus` выполняется автоматический перевод issue в корректный
  status перед повторным dispatch;
- успешные ветки analysis и implementation работают как раньше.

Проверка:

- unit-тесты decision flow, если выделена отдельная функция;
- integration test на auto-remediation path и explain-only path.

### Этап 4. Verification и регрессии

Цель:

- подтвердить, что UX ошибок улучшился без поломки stage-aware dispatch.

Основание:

- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)

Результат этапа:

- `cargo test` проходит;
- есть integration coverage на новый operator-facing текст и remediation path;
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
- там, где система может безопасно исправить проблему сама, она делает это и
  явно сообщает пользователю о результате;
- `project_id`, issue URL, current status и allowed statuses выводятся там, где
  это нужно для explain-only fallback;
- `Ready for Implementation` и остальные implementation entry statuses не
  ошибочно попадают в deny-path;
- новый formatter покрыт unit-тестами;
- есть integration test, подтверждающий реальный CLI-вывод и mutation path.

## Открытые вопросы и риски

- нужно не допустить drift между dispatch-логикой и отдельно собранным списком
  allowed statuses;
- важно не сделать integration assertions слишком хрупкими по форматированию;
- если adapter выберет неустойчивый внешний CLI-output вместо структурированного
  JSON, тестируемость и надежность ухудшатся;
- при расширении списка допустимых статусов в будущем formatter должен
  автоматически наследовать новое множество без ручного редактирования строк;
- auto-remediation нельзя расширять на случаи, где target status не
  определяется однозначно.

## Журнал изменений

### 2026-03-15

- создан issue-level implementation plan для issue 11
