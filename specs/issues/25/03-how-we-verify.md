# Issue 25: Как проверяем

Статус: draft
Последнее обновление: 2026-03-15

## Acceptance Criteria

- повторный `run` не создает вторую независимую agent session, если связанная
  session еще жива;
- если связанная pane все еще существует, `run` не запускает новый agent
  process и сообщает, что использован existing live context;
- если связанная pane пропала, но есть валидный resume handle, `run` создает
  новую pane и в ней восстанавливает существующую agent session;
- если resume handle отсутствует или invalid, `run` явно идет по path создания
  новой session;
- `session.json` хранит agent-specific resume metadata отдельно от
  `session_uuid`;
- stdout и `launch.log` явно различают сценарии `created`, `reused`, `restored`;
- restore path уважает текущие launcher semantics для `pane` и `tab`;
- сценарий `pane удалили, session осталась, run восстановил session` покрыт
  тестом или headless integration check.

## Ready Criteria

- issue классифицирована как `medium feature` для `infra/platform`;
- зафиксирован новый runtime contract для agent-specific resume metadata;
- зафиксирован fallback policy для manifests без новых metadata;
- отдельно описан риск и adapter boundary для `codex` vs `claude`;
- verification path не требует host `zellij` и использует только headless
  окружение;
- в implementation plan включены обновления docs, runtime schema и тестов.

## Invariants

- source of truth по issue status остается в GitHub Project;
- `session_uuid` сохраняет роль issue/stage binding и не подменяется
  backend-specific native session id;
- одна issue/stage не должна порождать две живые независимые agent session из-за
  потери pane;
- `reuse_live_pane` не должен запускать новый agent process;
- restore path должен сначала делать live-check текущего `zellij` context, а
  не blindly доверять старому `pane_id`;
- отсутствие новых agent metadata в старых runtime files не ломает `run`;
- все `zellij`-related автоматические проверки выполняются только в headless
  sandbox.

## Test Plan

Unit tests:

- runtime schema корректно читает и пишет optional block `agent`;
- старый `session.json` без блока `agent` остается валидным;
- decision layer выбирает `create_new_session`, если binding отсутствует;
- decision layer выбирает `reuse_live_pane`, если pane жива и resume metadata
  валидны;
- decision layer выбирает `restore_in_new_pane`, если pane пропала, но
  `resume_handle` есть;
- decision layer делает fallback на `create_new_session`, если resume metadata
  отсутствуют или повреждены;
- diagnostics formatter различает `created`, `reused`, `restored`;
- `pane` и `tab` restore paths корректно резолвят effective launch context.

Integration/headless tests:

- существующая pane жива, `run` повторно заходит в issue и не запускает второй
  agent process;
- pane удалена, `run` создает новую pane и запускает agent в resume mode;
- отсутствует valid resume metadata, `run` создает новую session и явно пишет
  `created`;
- restore path в `launch_target = pane` использует shared tab semantics;
- restore path в `launch_target = tab` использует expected tab naming semantics;
- runtime metadata после restore обновляет `pane_id` и при необходимости
  `tab_id`;
- `launch.log` содержит branch decision и restore outcome;
- негативный сценарий: stored pane id указывает на уже несуществующий context,
  но `run` не падает silently и не создает дубль при наличии resume path.

Manual validation:

- прогнать headless runner или Docker-based integration suite;
- проверить `session.json`, `launch.log` и stdout после сценариев
  `created/reused/restored`;
- убедиться, что ни один тест не трогает host `zellij`.

## Verification Checklist

- runtime schema расширена backward compatible способом;
- decision matrix `new/reuse/restore` покрыт unit-тестами;
- launcher не запускает второй agent process в сценарии live pane;
- restore path покрыт headless integration tests;
- diagnostics явно различают outcome;
- feature/runtime docs и новый ADR синхронизированы с кодом;
- проверки не требуют host-level `zellij`.

## Happy Path

1. Issue уже имеет `session_uuid` и валидный agent resume handle.
2. Старая pane недоступна, но сама agent session еще жива.
3. Оператор запускает `ai-teamlead run <issue>`.
4. `run` определяет, что нужен outcome `restore_in_new_pane`.
5. Launcher создает новую pane в корректном `zellij` context.
6. `launch-agent.sh` запускает соответствующий agent CLI в resume mode.
7. Runtime обновляет `pane_id`, а diagnostics явно показывают `restored`.

## Edge Cases

- pane жива, но tab уже переименован или перемещен;
- session существует, но stored `pane_id` больше не найден;
- `launch_target = pane`, и restore должен вернуться в shared tab;
- `launch_target = tab`, и restore должен использовать issue-aware tab naming;
- old manifest не содержит новых agent metadata;
- resume handle есть, но CLI resume path возвращает ошибку.

## Failure Scenarios

- `run` определяет живую pane, но все равно стартует новый agent process;
- runtime хранит `resume_handle`, который больше невалиден, а diagnostics этого
  не показывают;
- restore path silently превращается в `new session created`;
- restore path ломает existing `pane` / `tab` launcher semantics;
- schema migration ломает старые `session.json`;
- тесты случайно взаимодействуют с host `zellij`.

## Observability

- stdout должен явно печатать outcome:
  `agent_session=created|reused|restored`;
- `launch.log` должен показывать:
  - branch decision;
  - использованный agent kind;
  - был ли выполнен native resume;
  - какие `session/tab/pane` ids стали актуальными после запуска;
- `session.json` должен отражать agent-specific resume metadata и
  `last_launch_outcome`;
- при restore failure лог должен объяснять, почему был выбран fallback на
  создание новой session или почему запуск завершился ошибкой.
