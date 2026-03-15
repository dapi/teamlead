# Issue 14: Как проверяем

## Acceptance Criteria

1. `ProjectIssueItem` содержит список `assignees`.
2. `load_project_snapshot` получает assignee login-ы для issue из GitHub
   Project snapshot.
3. `poll` без active override для `poll.assignee_filter` использует effective
   default `"$me"`.
4. `poll` с `poll.assignee_filter: "$me"` выбирает только issue текущего
   GitHub-пользователя.
5. `poll` с `poll.assignee_filter: "$all"` отключает фильтрацию по assignee.
6. `poll` с `poll.assignee_filter: "$unassigned"` выбирает только issue без
   assignee.
7. `poll` с `poll.assignee_filter: "username"` выбирает только issue
   указанного пользователя.
8. `"$me"` резолвится один раз на старте процесса и переиспользуется в рамках
   этого `poll` или `loop`.
9. Issue с несколькими assignees подхватываются, если совпадает хотя бы один
   login.
10. `run` при mismatch с effective filter показывает warning и требует approve
    пользователя.
11. `run --force` при mismatch показывает warning, но не требует approve.
12. Старые `settings.yml` без секции `poll` продолжают загружаться без ошибок.
13. Реализация `run`-ветки этой задачи не начинается до закрытия issue `#11`.

## Ready Criteria

- analysis-артефакты фиксируют scope, non-goals и verification contract без
  пробелов;
- config contract, GitHub snapshot contract и poll-selection logic описаны
  консистентно;
- есть тесты хотя бы на один целевой use case изменения и на ключевые edge
  cases из issue;
- documented template `settings.yml` синхронизирован с новой optional
  настройкой;
- проверка не требует host `zellij` пользователя;
- dependency на issue `#11` явно зафиксирована в analysis docs и GitHub links.

## Invariants

- отсутствие active `poll.assignee_filter` означает effective mode `"$me"`;
- `"$all"` является единственным явным способом отключить фильтрацию по
  assignee;
- `"$unassigned"` выбирает только issue без assignee;
- `run` использует ту же effective policy, что и `poll`;
- порядок backlog-выбора остается детерминированным среди уже eligible issue;
- в login-based modes issue без assignee не могут матчиться;
- resolved current user не должен вычисляться на каждом poll-cycle.

## Happy Path

1. В `settings.yml` задан `poll.assignee_filter: "$me"`.
2. На старте `poll` или `loop` текущий GitHub login успешно резолвится через
   `gh api user`.
3. GitHub snapshot содержит backlog-issue из текущего repo с разными
   assignees.
4. Selection logic выбирает верхнюю issue, у которой есть совпадающий assignee.
5. Дальше issue передается в тот же общий `run`-path, что и раньше.
6. При ручном `run` для matching issue дополнительных предупреждений нет.

## Edge Cases

- active override не задан;
- фильтр задан как `"$all"`;
- фильтр задан как `"$unassigned"`;
- фильтр задан как literal username;
- issue без assignee;
- issue с несколькими assignees;
- backlog содержит подходящие issue другого repo;
- `gh api user` недоступен из-за отсутствующей авторизации;
- старый config не содержит блока `poll`;
- `run` вызывается для issue, которая не проходит policy;
- `run --force` вызывается для issue, которая не проходит policy.

## Test Plan

Unit tests:

- `Config` корректно загружает `poll.assignee_filter` и не требует блока
  `poll`;
- `Config` без active override дает effective default `"$me"`;
- `select_next_backlog_project_item` при mode `"$all"` сохраняет старое
  поведение;
- `select_next_backlog_project_item` корректно обрабатывает `"$unassigned"`;
- `select_next_backlog_project_item` выбирает issue с совпадающим assignee;
- `select_next_backlog_project_item` игнорирует issue без assignee в
  login-based modes;
- `select_next_backlog_project_item` считает совпадение любого элемента в
  `assignees`;
- helper для resolve current user вызывает `gh api user --jq '.login'` и
  возвращает login как строку;
- ошибка resolve current user поднимается как явная startup error;
- unit tests для `run` policy-check и `--force` должны проектироваться после
  завершения issue `#11`.

Integration tests:

- `poll` без active override claim-ит только backlog-issue текущего
  пользователя;
- `poll` с `"$all"` claim-ит первую подходящую backlog-issue без учета
  assignee;
- `poll` с `"$unassigned"` claim-ит только backlog-issue без assignee;
- `poll` с literal username claim-ит только backlog-issue нужного assignee;
- `poll` с `"$me"` использует gh stub для `api user` и claim-ит только issue
  текущего пользователя;
- `run` mismatch warning/approve path добавляется только после реализации
  issue `#11`.

Operational validation:

- zellij-touching integration path, если он затрагивается новыми тестами,
  запускать только в headless-окружении;
- при ручной проверке достаточно увидеть, что `poll` либо claim-ит нужную
  issue, либо заканчивается сообщением `no eligible backlog issues`.

## Verification Checklist

- unit tests для `config`, `github` и `domain` проходят;
- integration tests со stub `gh` покрывают режимы default `"$me"`, `"$all"`,
  `"$unassigned"` и literal username;
- fixtures GitHub snapshot содержат assignees и проверяют multiple-assignee
  сценарий;
- `templates/init/settings.yml` содержит закомментированный пример
  `poll.assignee_filter`;
- blocking dependency на issue `#11` отражена в docs и GitHub issue graph.

## Failure Scenarios

- `gh api user` падает: `poll` или `loop` должны завершаться явной ошибкой до
  claim любой issue;
- GraphQL snapshot не содержит assignees: тесты должны ловить, что фильтрация
  не может работать корректно;
- `run` реализует mismatch semantics отдельно от issue `#11`: это считается
  нарушением dependency contract;
- фильтр меняет порядок среди eligible issue: это нарушение ADR-0009;
- старый config перестает парситься без блока `poll`: это регрессия
  обратной совместимости.

## Observability

- startup error для `"$me"` должен явно указывать на проблему вызова
  `gh api user`;
- пустой результат `poll` должен оставаться диагностируемым как
  `no eligible backlog issues`;
- test stubs должны позволять отдельно проверить GraphQL snapshot и вызов
  `gh api user`, чтобы было видно, на каком слое произошел сбой.
- после реализации issue `#11` наблюдаемость `run` warning/approve path должна
  проверяться уже в общем user-facing UX-контракте `run`.
