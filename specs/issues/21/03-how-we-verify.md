# Issue 21: Как проверяем

Статус: draft
Последнее обновление: 2026-03-15

## Acceptance Criteria

- `run <issue>` для open issue вне default project не падает на отсутствии
  project item, а сначала добавляет issue в project;
- если после attach у project item отсутствует status, `run` выставляет
  `Backlog`;
- если у issue нет assignee, `run` назначает issue на текущего пользователя из
  `gh` context;
- если assignee уже есть, `run` не перетирает существующее назначение;
- после preflight `run` продолжает обычный stage-aware flow по правилам
  `issue-analysis-flow`;
- ошибки `add-to-project`, `set-status`, `resolve-current-user` и `assign`
  видны оператору и не приводят к тихому продолжению flow;
- `poll` не меняет свое поведение и по-прежнему работает только с уже
  нормализованными backlog issue;
- SSOT, feature doc и ADR синхронизированы с новым contract.

## Ready Criteria

- issue классифицирована как `medium feature` для `infra/platform`;
- policy по assignee зафиксирована явно:
  - identity из `gh api user --jq ".login"`;
  - assignment только при пустом `assignee`;
  - assignment failure останавливает `run`;
- определен канонический порядок preflight шагов до stage dispatch;
- понятно, какие документы обновляются как канонические, а какие только как
  summary-layer;
- нет блокирующих открытых вопросов по scope и ownership policy.

## Invariants

- `run` остается единственным public issue-level entrypoint;
- `poll` не получает auto-normalization side effects;
- preflight выполняется до launcher orchestration и до обычного stage dispatch;
- `run` не меняет уже существующий assignee;
- `run` не меняет уже существующий status только ради нормализации;
- identity для assignment берется только из текущего `gh` context;
- при ошибке любой GitHub-side mutation flow не продолжается скрыто;
- runtime и `zellij` не используются как источник истины о project/status/
  assignee состоянии issue.

## Test Plan

Unit tests:

- `prepare_manual_run_issue(...)` или новый preflight helper:
  - возвращает ошибку для отсутствующей issue;
  - возвращает ошибку для closed issue;
  - для issue вне project вызывает add-to-project и set-status;
  - для issue в project без status вызывает только set-status;
  - для issue без assignee вызывает `resolve_current_user()` и `assign_issue()`;
  - для issue с assignee не вызывает `resolve_current_user()` и
    `assign_issue()`;
  - не продолжает stage path после ошибки assignment;
- `GhProjectClient`:
  - парсит assignees из repo issue response;
  - корректно строит команду assignment;
  - корректно bubble-up ошибки `gh`;
- regression tests для `poll` подтверждают отсутствие новой normalization
  логики в selection path.

Integration tests:

- `run` на fake `gh` shell для issue вне project проходит sequence:
  `load issue -> add to project -> set backlog -> resolve current user ->
  assign -> continue`;
- `run` для issue в project с existing assignee не делает reassign и
  продолжает обычный flow;
- `run` при ошибке `assign` завершается ошибкой и не доходит до launcher path;
- `run` при ошибке `resolve_current_user` завершается ошибкой без assignment;
- `poll` остается совместимым с прежним behavior и не вызывает новые GitHub
  mutations.

Manual or headless validation:

- прогнать unit и integration suite с fake GitHub shell / stub;
- при наличии отдельного headless runner проверить operator-visible stdout
  `run`, чтобы шаги auto-normalization были различимы;
- не использовать host `zellij` пользователя для проверки, потому что задача не
  требует живого multiplexer behavior.

## Verification Checklist

- app-layer preflight helper выделен и покрыт тестами;
- GitHub adapter умеет читать assignees у repo issue и выполнять assignment;
- для existing assignee есть явный no-op test;
- failure paths по add/status/assign/resolve покрыты тестами;
- `poll` regression не меняется;
- SSOT, feature doc, ADR и при необходимости summary-layer синхронизированы;
- operator-visible diagnostics позволяют понять, какой именно normalize-step
  выполнился или упал.

## Happy Path

1. Оператор запускает `ai-teamlead run 21`.
2. `run` находит open issue в текущем repo.
3. Issue при необходимости добавляется в default project.
4. При отсутствии status выставляется `Backlog`.
5. При отсутствии assignee issue назначается на текущего пользователя `gh`.
6. После этого `run` продолжает обычный stage-aware flow и может перевести
   issue в `Analysis In Progress`.

## Edge Cases

- issue уже находится в project и уже имеет status `Backlog`;
- issue уже имеет assignee, но status отсутствует;
- issue отсутствует в project, но уже имеет assignee;
- `gh` user успешно определяется, но assignment запрещен repo policy;
- concurrent actor меняет status или assignee между snapshot и mutation.

## Failure Scenarios

- `gh api user --jq ".login"` возвращает ошибку или пустой login;
- `addProjectV2ItemById` не проходит из-за прав или сетевой ошибки;
- `updateProjectV2ItemFieldValue` не проходит и `Backlog` не выставляется;
- assignment не проходит из-за прав, недопустимого login или ограничений
  репозитория;
- diagnostics скрывают ошибку preflight и оператор не понимает, почему `run`
  остановился до launcher stage.

## Observability

- stdout `run` должен явно показывать выполненные auto-normalization шаги;
- ошибка должна указывать, какой именно GitHub-side шаг не удался:
  attach, set-status, resolve-current-user или assign;
- тестовый fake/stub layer должен позволять восстановить порядок GitHub-вызовов
  до перехода к launcher logic.
