# Issue 14: Как проверяем

## Acceptance Criteria

1. `ProjectIssueItem` содержит список `assignees`.
2. `load_project_snapshot` получает assignee login-ы для issue из GitHub
   Project snapshot.
3. `poll` без `poll.assignee_filter` сохраняет текущее поведение.
4. `poll` с `poll.assignee_filter: "$me"` выбирает только issue текущего
   GitHub-пользователя.
5. `poll` с `poll.assignee_filter: "username"` выбирает только issue
   указанного пользователя.
6. `"$me"` резолвится один раз на старте процесса и переиспользуется в рамках
   этого `poll` или `loop`.
7. Issue без assignee не подхватываются, если фильтр активен.
8. Issue с несколькими assignees подхватываются, если совпадает хотя бы один
   login.
9. Ручной `run` не зависит от `poll.assignee_filter`.
10. Старые `settings.yml` без секции `poll` продолжают загружаться без ошибок.

## Ready Criteria

- analysis-артефакты фиксируют scope, non-goals и verification contract без
  пробелов;
- config contract, GitHub snapshot contract и poll-selection logic описаны
  консистентно;
- есть тесты хотя бы на один целевой use case изменения и на ключевые edge
  cases из issue;
- documented template `settings.yml` синхронизирован с новой optional
  настройкой;
- проверка не требует host `zellij` пользователя.

## Invariants

- отсутствие `poll.assignee_filter` полностью сохраняет старую selection
  semantics;
- `poll.assignee_filter` влияет только на команды `poll` и `loop`;
- `run` остается явным ручным entrypoint без фильтрации по assignee;
- порядок backlog-выбора остается детерминированным среди уже eligible issue;
- issue без assignee не могут матчиться при активном фильтре;
- resolved current user не должен вычисляться на каждом poll-cycle.

## Happy Path

1. В `settings.yml` задан `poll.assignee_filter: "$me"`.
2. На старте `poll` или `loop` текущий GitHub login успешно резолвится через
   `gh api user`.
3. GitHub snapshot содержит backlog-issue из текущего repo с разными
   assignees.
4. Selection logic выбирает верхнюю issue, у которой есть совпадающий assignee.
5. Дальше issue передается в тот же общий `run`-path, что и раньше.

## Edge Cases

- фильтр не задан совсем;
- фильтр задан как literal username;
- issue без assignee;
- issue с несколькими assignees;
- backlog содержит подходящие issue другого repo;
- `gh api user` недоступен из-за отсутствующей авторизации;
- старый config не содержит блока `poll`.

## Test Plan

Unit tests:

- `Config` корректно загружает `poll.assignee_filter` и не требует блока
  `poll`;
- `select_next_backlog_project_item` сохраняет старое поведение без фильтра;
- `select_next_backlog_project_item` выбирает issue с совпадающим assignee;
- `select_next_backlog_project_item` игнорирует issue без assignee при активном
  фильтре;
- `select_next_backlog_project_item` считает совпадение любого элемента в
  `assignees`;
- helper для resolve current user вызывает `gh api user --jq '.login'` и
  возвращает login как строку;
- ошибка resolve current user поднимается как явная startup error.

Integration tests:

- `poll` без фильтра продолжает claim-ить первую подходящую backlog-issue;
- `poll` с literal username claim-ит только backlog-issue нужного assignee;
- `poll` с `"$me"` использует gh stub для `api user` и claim-ит только issue
  текущего пользователя;
- `poll` с активным фильтром не claim-ит issue без assignee;
- `run <issue>` проходит для issue без учета `poll.assignee_filter`.

Operational validation:

- zellij-touching integration path, если он затрагивается новыми тестами,
  запускать только в headless-окружении;
- при ручной проверке достаточно увидеть, что `poll` либо claim-ит нужную
  issue, либо заканчивается сообщением `no eligible backlog issues`.

## Verification Checklist

- unit tests для `config`, `github` и `domain` проходят;
- integration tests со stub `gh` покрывают режимы `no filter`, `"$me"` и
  literal username;
- fixtures GitHub snapshot содержат assignees и проверяют multiple-assignee
  сценарий;
- `templates/init/settings.yml` содержит закомментированный пример
  `poll.assignee_filter`;
- `run`-path не получил неявную зависимость от нового config key.

## Failure Scenarios

- `gh api user` падает: `poll` или `loop` должны завершаться явной ошибкой до
  claim любой issue;
- GraphQL snapshot не содержит assignees: тесты должны ловить, что фильтрация
  не может работать корректно;
- фильтр ошибочно применяется к `run`: это считается функциональной
  регрессией;
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
