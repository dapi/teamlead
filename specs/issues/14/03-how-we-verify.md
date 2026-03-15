# Issue 14: Как проверяем

## Acceptance Criteria

1. `ProjectIssueItem` содержит список `assignees`.
2. `load_project_snapshot` получает assignee login-ы для issue из GitHub
   Project snapshot.
3. `poll` без `poll.assignee_filter` сохраняет текущее поведение без фильтрации
   по assignee.
4. `poll` с `poll.assignee_filter: "$me"` выбирает только issue текущего
   GitHub-пользователя.
5. `poll` с `poll.assignee_filter: "username"` выбирает только issue
   указанного пользователя.
6. `"$me"` резолвится один раз на старте процесса и переиспользуется в рамках
   этого `poll` или `loop`.
7. Issue без assignee не подхватываются, если фильтр задан.
8. Issue с несколькими assignees подхватываются, если совпадает хотя бы один
   login.
9. `run` не зависит от `assignee_filter`.
10. Старые `settings.yml` без секции `poll` продолжают загружаться без ошибок.

## Ready Criteria

- analysis-артефакты фиксируют scope без расширений сверх исходного issue;
- config contract, GitHub snapshot contract и selection logic описаны
  консистентно;
- test plan покрывает хотя бы один целевой use case и ключевые edge cases из
  issue;
- documented template `settings.yml` синхронизирован с optional-настройкой;
- проверка не требует host `zellij` пользователя.

## Invariants

- отсутствие active `poll.assignee_filter` не меняет текущее поведение `poll`;
- `assignee_filter` влияет только на selection path `poll`/`loop`;
- `run` остается независимым от assignee filtering;
- порядок backlog-выбора остается детерминированным среди already eligible
  issue;
- `"$me"` не должен резолвиться на каждом цикле `loop`;
- в login-based mode issue без assignee не могут матчиться.

## Happy Path

1. В `settings.yml` задан `poll.assignee_filter: "$me"`.
2. На старте `poll` или `loop` текущий GitHub login успешно резолвится через
   `gh api user`.
3. GitHub snapshot содержит backlog-issue текущего repo с разными assignees.
4. Selection logic выбирает первую issue, у которой есть совпадающий assignee.
5. Дальше issue передается в тот же общий `run`-path, что и раньше.

## Edge Cases

- `poll.assignee_filter` не задан;
- фильтр задан как `"$me"`;
- фильтр задан как literal username;
- issue без assignee;
- issue с несколькими assignees;
- backlog содержит подходящие issue другого repo;
- `gh api user` недоступен из-за отсутствующей авторизации;
- старый config не содержит блока `poll`.

## Test Plan

Unit tests:

- `Config` корректно загружает optional `poll.assignee_filter`;
- `Config` без блока `poll` остается валидным;
- `select_next_backlog_project_item` при `None` сохраняет старое поведение;
- `select_next_backlog_project_item` выбирает issue с совпадающим assignee;
- `select_next_backlog_project_item` игнорирует issue без assignee в
  login-based mode;
- `select_next_backlog_project_item` считает совпадение любого элемента в
  `assignees`;
- helper `resolve_current_user` вызывает `gh api user --jq '.login'` и
  возвращает login как строку;
- ошибка resolve current user поднимается как явная startup error.

Integration tests:

- `poll` без `assignee_filter` claim-ит первую подходящую backlog-issue без
  учета assignee;
- `poll` с `"$me"` использует gh stub для `api user` и claim-ит только issue
  текущего пользователя;
- `poll` с literal username claim-ит только issue нужного assignee;
- loop-path переиспользует тот же фильтр и не ломает текущее сообщение
  `no eligible backlog issues`.

## Verification Checklist

- unit tests для `config`, `github` и `domain` проходят;
- integration tests со stub `gh` покрывают режимы `unset`, `"$me"` и literal
  username;
- fixtures GitHub snapshot содержат `assignees` и multiple-assignee сценарий;
- `templates/init/settings.yml` содержит закомментированный пример
  `poll.assignee_filter`;
- manual smoke подтверждает, что `run` ведет себя как раньше.

## Failure Scenarios

- `gh api user` падает: `poll` или `loop` должны завершаться явной ошибкой до
  claim любой issue;
- GraphQL snapshot не содержит `assignees`: тесты должны ловить, что фильтрация
  не может работать корректно;
- фильтр случайно применяется к `run`: это считается нарушением scope issue;
- старый config перестает парситься без блока `poll`: это регрессия обратной
  совместимости.

## Observability

- startup error для `"$me"` должен явно указывать на проблему вызова
  `gh api user`;
- пустой результат `poll` должен оставаться диагностируемым как
  `no eligible backlog issues`;
- test stubs должны позволять отдельно проверить GraphQL snapshot и вызов
  `gh api user`, чтобы было видно, на каком слое произошел сбой.
