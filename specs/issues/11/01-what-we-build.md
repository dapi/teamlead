# Issue 11: Что строим

Статус: draft
Последнее обновление: 2026-03-15

## Problem

Сейчас ручной запуск `ai-teamlead run <issue>` дает слишком бедные ошибки в
ситуациях, которые для оператора являются ожидаемыми и исправимыми:

- issue существует, но не добавлена в configured GitHub Project;
- issue есть в проекте, но ее статус не допускает повторный `run`;
- issue уже закрыта;
- пользователь передал номер или URL несуществующей issue.

Текущее поведение смешивает разные причины отказа в две неинформативные
формулировки:

- `issue #N is not linked to the project`
- `run denied for issue #N: status is not a valid run entry point`

Такая диагностика не говорит оператору:

- какой проект ожидается;
- существует ли issue вообще;
- открыта ли она;
- какой у нее текущий статус;
- какие статусы допустимы для stage-aware `run`;
- какое следующее действие нужно сделать вручную.

## Who Is It For

Изменение нужно двум ролям:

- оператору репозитория, который вручную запускает `ai-teamlead run <issue>` и
  ожидает быстро понять причину отказа и следующий шаг;
- разработчику `ai-teamlead`, которому нужен стабильный и тестируемый контракт
  user-facing ошибок без дублирования ad-hoc форматирования по коду.

## Outcome

После изменения пользователь получает детерминированные и actionable ошибки для
ручного `run`:

- если issue не найдена, сообщение явно говорит, что номер или URL не
  разрешились в существующую issue текущего репозитория;
- если issue закрыта, сообщение сообщает текущее состояние и объясняет, что
  `run` работает только для открытых issue;
- если issue не входит в configured проект, сообщение указывает
  `github.project_id` и подсказывает, как добавить issue в проект;
- если статус не подходит для запуска, сообщение показывает фактический статус
  и список допустимых entry status для current `run` contract.

## Scope

В scope первой версии входят:

- улучшение operator-facing ошибок в ручном пути `run <issue>`;
- явное различение случаев `not found`, `closed`, `not in project`,
  `invalid entry status`;
- включение в сообщение project context из `settings.yml`;
- включение в сообщение текущего статуса и списка допустимых статусов;
- вынос форматирования отказов в отдельный доменный контракт;
- unit-тесты на формирование сообщений и integration coverage на CLI-path.

## Non-Goals

В эту задачу не входят:

- изменение разрешенных статусов для `run`;
- возврат к analysis-only модели `run`;
- автоматическое добавление issue в GitHub Project;
- автоматическое переоткрытие закрытой issue;
- изменение GitHub Project status через `run` только ради исправления ошибки
  пользователя;
- локализационная платформа или многоязычный runtime beyond конкретных
  сообщений об ошибках.

## Constraints And Assumptions

- Канонический источник истины о stage и допустимых переходах остается в
  GitHub Project status, как зафиксировано в
  [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md),
  [../../../docs/issue-implementation-flow.md](../../../docs/issue-implementation-flow.md)
  и
  [../../../docs/adr/0024-stage-aware-run-dispatch.md](../../../docs/adr/0024-stage-aware-run-dispatch.md).
- `run <issue>` должен оставаться stage-aware entrypoint и принимать статусы
  implementation lifecycle, которые уже разрешены проектом.
- Сообщение должно быть user-facing, но не должно скрывать технический факт,
  который нужен для ручного исправления: `project_id`, current status,
  issue state, issue URL.
- Поскольку `settings.yml` хранит GraphQL `project_id`, а не numeric project
  number для `gh project item-add`, первая версия может показывать либо
  команду с placeholder `<PROJECT_NUMBER>`, либо дополнительную подсказку как
  найти project number; главное, чтобы оператор понимал target project и
  следующий ручной шаг.
- Для различения `not in project` и `closed/not found` одного `ProjectSnapshot`
  недостаточно; реализация может потребовать отдельный GitHub lookup вне
  project snapshot.
- Сообщения должны оставаться пригодными для детерминированного тестирования и
  не зависеть от нестабильного форматирования внешнего CLI.

## Observed Behavior

По текущему коду:

- `run_manual_run` сначала грузит `ProjectSnapshot`;
- затем ищет issue только среди `snapshot.items` текущего repo;
- если item не найден, немедленно возвращает
  `issue #N is not linked to the project`;
- если item найден, но `decide_run_stage()` вернул отказ, пользователь видит
  только `status is not a valid run entry point`.

Из-за этого оператор не может отличить:

- закрытую issue вне snapshot;
- несуществующую issue;
- issue другого проекта;
- issue в проекте с неподходящим статусом.

## Expected Behavior

Ручной `run <issue>` должен сначала установить реальный статус issue в
репозитории, а уже потом выбирать одно из человеко-понятных объяснений:

1. issue не существует;
2. issue существует, но закрыта;
3. issue открыта, но не связана с target project;
4. issue открыта и связана с project, но ее project status не допускает `run`.

Для каждого случая сообщение должно содержать минимальный набор данных,
достаточный для ручного исправления без чтения исходников.

## Impact

Без этой задачи ручной операторский цикл остается хрупким:

- пользователь тратит время на догадку, что именно сломано;
- приходится читать код или логи вместо явной инструкции;
- пример из issue про `Ready for Implementation` может привести к неверной
  реализации, если не учитывать текущий stage-aware контракт;
- отсутствие явного доменного формата ошибок затрудняет unit-test coverage и
  создает риск повторного drift между ветками отказов.

## Dependencies

- stage-aware контракт `run` из
  [../../../docs/adr/0024-stage-aware-run-dispatch.md](../../../docs/adr/0024-stage-aware-run-dispatch.md);
- source of truth по статусам из
  [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
  и
  [../../../docs/issue-implementation-flow.md](../../../docs/issue-implementation-flow.md);
- GitHub integration layer через `gh` из
  [../../../docs/adr/0006-use-gh-cli-as-github-integration-layer.md](../../../docs/adr/0006-use-gh-cli-as-github-integration-layer.md);
- текущие точки изменения в `src/app.rs`, `src/domain.rs`, возможно `src/github.rs`
  и CLI/integration tests;
- quality bar из [../../../docs/code-quality.md](../../../docs/code-quality.md),
  который требует тест на каждое значимое изменение поведения `run`.
