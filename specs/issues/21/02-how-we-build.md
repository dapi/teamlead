# Issue 21: Как строим

Статус: draft
Последнее обновление: 2026-03-15

## Approach

Изменение нужно реализовать как явный app-level preflight layer внутри `run`,
а не как разрозненные ad hoc side effects в разных ветках.

Технический подход:

- сохранить `run` каноническим issue-level entrypoint;
- перед stage dispatch и launcher orchestration выполнить единый helper
  нормализации issue;
- использовать уже существующий partial path для add-to-project и set-status
  как базу, но сделать порядок шагов и error semantics явными;
- дополнить GitHub adapter недостающими операциями для assignee-path;
- возвращать из preflight уже нормализованную issue-модель, пригодную для
  дальнейшего `run_issue_entrypoint`.

Каноническая последовательность:

1. Загрузить issue из текущего repo и убедиться, что она существует и `open`.
2. Найти project item в snapshot; если item отсутствует, добавить issue в
   default project.
3. Если у project item нет status, выставить `Backlog`.
4. Если у issue нет assignee, получить current `gh` login и назначить issue на
   этого пользователя.
5. Передать нормализованную issue в существующий stage-aware `run`.

## Affected Areas

- `src/app.rs`:
  - `prepare_manual_run_issue(...)` или соседний helper должен стать явной
    preflight normalization boundary;
  - нужно обновить order of operations и operator-visible diagnostics;
- `src/github.rs`:
  - расширить `load_repo_issue(...)` данными по assignee;
  - добавить `assign_issue(...)`;
  - переиспользовать `resolve_current_user(...)` как часть `run` preflight, а
    не только `poll.assignee_filter`;
- тесты app-layer и GitHub adapter:
  - happy path для attach/status/assign;
  - no-op path при уже существующем assignee;
  - fatal errors на GitHub mutations;
  - regression для `poll`;
- каноническая документация:
  - `docs/issue-analysis-flow.md`;
  - `docs/features/0003-agent-launch-orchestration/02-how-we-build.md`;
  - новый ADR про run preflight normalization;
  - repo-level summary, если после обновления канонических документов behavior
    `run` заметно меняет верхнеуровневое описание CLI.

## Interfaces And Data

Входные данные preflight:

- `issue_ref`, из которого уже выводится `issue_number`;
- `github.project_id` и status names из `settings.yml`;
- project snapshot c `item_id`, `status_name`, `status_option_id`,
  `assignees`;
- repo issue metadata из GitHub, включая:
  - `id`;
  - `number`;
  - `state`;
  - `url`;
  - `assignees`.

Выходные данные preflight:

- нормализованный `ProjectIssueItem`, у которого:
  - есть `item_id` в default project;
  - есть `status_name = Backlog`, если раньше status отсутствовал;
  - есть актуальный список `assignees` после assignment или no-op;
- operator-visible сообщения о выполненных auto-normalize шагах.

Policy по данным:

- если issue уже есть в project и status задан, `run` не меняет status;
- если assignees не пустой, `run` не вызывает `resolve_current_user()` и не
  делает `assign_issue(...)`;
- если issue была добавлена в project во время preflight, in-memory модель
  должна сразу содержать новый `item_id` и согласованное состояние status /
  assignees без обязательного повторного full snapshot reload.

## Configuration And Runtime Assumptions

- новые поля в `settings.yml` не требуются;
- источником стартового status остается
  `issue_analysis_flow.statuses.backlog`;
- preflight выполняется до любых действий с `zellij`, `session_uuid` и
  launcher path;
- `run` продолжает использовать существующий stage-aware dispatch после
  завершения preflight;
- `poll` по-прежнему использует snapshot selection и не получает этот
  normalization layer;
- `gh` context на host-машине должен иметь права:
  - читать issue;
  - читать и менять project item;
  - назначать assignee на issue;
- ошибки прав доступа считаются штатно диагностируемым blocker, а не поводом
  на частичное silent continue.

## Risks

- если add-to-project и set-status выполнятся успешно, а assign завершится
  ошибкой, issue останется частично нормализованной; это допустимо, но ошибка
  должна быть явной и останавливать запуск;
- если assignee проверяется только по project snapshot, ветка для issue вне
  project может ошибочно выполнить reassign; поэтому repo issue metadata должна
  содержать assignees сама по себе;
- если preflight будет размазан между `app.rs` и launcher path, оператору
  станет трудно понять, на каком шаге произошла GitHub-side мутация;
- если documentation delta обновить частично, `run` и `poll` снова начнут
  выглядеть как одинаково строгие команды, хотя их operator semantics уже
  различаются;
- GitHub-side concurrency может приводить к stale snapshot между чтением и
  мутацией, поэтому ошибки должны bubble-up без маскировки.

## External Interfaces

GitHub integration должна использовать существующий adapter boundary и
оставаться явно отделенной от orchestration logic.

Минимально нужны такие операции:

- `gh api graphql` для чтения repo issue и project snapshot;
- `gh api graphql` для `addProjectV2ItemById`;
- `gh api graphql` для `updateProjectV2ItemFieldValue`;
- `gh api user --jq ".login"` для resolve current user;
- `gh issue edit <number> --repo <owner>/<repo> --add-assignee <login>` или
  эквивалентный GitHub API path для assignment.

Требование:

- shell execution, parsing ответа GitHub и app-level policy не должны быть
  смешаны в одном helper.

## Architecture Notes

- лучше выделить helper уровня `normalize_manual_run_issue(...)`, чем дальше
  расширять `prepare_manual_run_issue(...)` набором несвязанных side effects;
- GitHub adapter должен инкапсулировать детали `gh` команд, а app-layer должен
  оперировать сущностями `RepoIssue`, `ProjectIssueItem` и outcome-policy;
- preflight должен завершаться до stage decision, чтобы `run` всегда принимал
  решение по уже нормализованной issue;
- launcher и runtime state не должны становиться источником истины для того,
  была ли issue назначена или прикреплена к project;
- при обновлении diagnostics полезно различать:
  - `issue added to project`;
  - `missing status set to Backlog`;
  - `issue assigned to current gh user`;
  - `existing assignee preserved`.

## ADR Impact

По правилам
[../../../docs/documentation-process.md](../../../docs/documentation-process.md)
это изменение затрагивает operator contract `run`, GitHub-side side effects,
ownership policy и различие между `run` и `poll`.

Поэтому нужен отдельный ADR, который явно фиксирует:

- почему preflight normalization относится именно к `run`, а не к `poll`;
- почему assignment считается частью operator-driven claim;
- почему identity берется из текущего `gh` context;
- почему в MVP assignment выполняется только при пустом `assignee`, без
  automatic reassign.

## Alternatives Considered

1. Оставить `run` строгим и требовать ручной подготовки issue в GitHub.

   Отклонено: это делает явный ручной запуск слишком хрупким и противоречит
   operator-driven semantics команды.

2. Применить такую же auto-normalization к `poll`.

   Отклонено: `poll` должен оставаться строгим backlog-selector и не делать
   скрытых claim-like side effects для случайно найденных issue.

3. Всегда делать reassign на текущего `gh` user при каждом `run`.

   Отклонено: это перетирает уже существующее ownership и повышает риск
   нежелательных side effects без явного запроса.

4. Брать identity из OS-user или git config.

   Отклонено: источником прав и фактического GitHub actor должен оставаться
   именно текущий `gh` context.

## Migration Or Rollout Notes

- конфигурационная миграция не требуется;
- уже нормализованные issue должны проходить preflight без дополнительных
  мутаций;
- partial existing behavior для add-to-project и set-status нужно сохранить, а
  не переписать заново без причины;
- rollout документации лучше делать в порядке:
  SSOT -> feature doc -> ADR -> summary layers;
- verification path не требует host `zellij`, потому что preflight касается
  GitHub layer и app orchestration до launcher stage.
