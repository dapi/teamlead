# Issue 25: Как строим

Статус: draft
Последнее обновление: 2026-03-15

## Approach

Изменение оформляется как расширение текущего launcher/runtime contract, а не
как замена `session_uuid` другой моделью.

Базовый технический подход:

- сохранить существующий issue-level binding `issue/stage -> session_uuid`;
- добавить в `session.json` отдельный agent-specific слой metadata, который
  хранит:
  - `agent_kind`;
  - `resume_handle` или эквивалентный deterministic resume token;
  - `resume_strategy`;
  - `last_launch_outcome`;
- перед запуском launcher path выполнять reconcile существующего binding:
  - проверить, жив ли ранее сохраненный `zellij` context;
  - проверить, есть ли валидный agent resume handle;
  - выбрать один из трех outcomes:
    - `create_new_session`;
    - `reuse_live_pane`;
    - `restore_in_new_pane`;
- для `reuse_live_pane` запрещать запуск второго agent process;
- для `restore_in_new_pane` создавать новую pane, но запускать агента в
  explicit resume mode;
- оставить существующий path создания новой session fallback-веткой, если
  валидного restore context не найдено.

## Affected Areas

- `src/runtime.rs` и runtime schema в `.git/.ai-teamlead/sessions/<session_uuid>/session.json`;
- `src/app.rs` и общий pre-launch reconcile внутри `run_issue_entrypoint(...)`;
- `src/zellij.rs` для live-check ранее записанной pane/tab/session и
  best-effort focus/reuse behavior;
- `./.ai-teamlead/launch-agent.sh` для разделения `new` и `resume` path;
- agent-specific launch logic для `codex` и `claude`;
- stdout/`launch.log` и operator-facing diagnostics;
- headless integration tests и unit tests;
- docs/ADR вокруг runtime artifacts и launcher orchestration.

## Interfaces And Data

Входные данные:

- существующий issue/session binding из `issues/<issue>.json`;
- `session.json` с уже сохраненными `zellij` ids;
- новый agent-specific resume metadata block в `session.json`;
- effective launcher config:
  - `zellij.session_name`;
  - `zellij.launch_target`;
  - `zellij.tab_name`;
  - optional `zellij.tab_name_template`;
- фактическое состояние `zellij`, получаемое через `list-panes`, `list-tabs` и
  другие session-scoped действия.

Предлагаемое расширение `session.json`:

```json
{
  "agent": {
    "kind": "codex",
    "resume_strategy": "native_resume_handle",
    "resume_handle": "019a7702-3f76-7f32-b1ea-ceb523057c35",
    "last_launch_outcome": "restored"
  }
}
```

Поля должны быть optional, чтобы старые manifests оставались валидными.

Решение pre-launch reconcile:

1. Если binding для issue/stage отсутствует, создать новую session как сейчас.
2. Если binding существует, загрузить `session.json`.
3. Проверить `zellij`-состояние:
   - существует ли session;
   - существует ли tab;
   - существует ли pane;
   - соответствует ли найденный context ожидаемому repo scope.
4. Проверить наличие agent resume metadata.
5. Выбрать outcome:
   - `reuse_live_pane`, если pane жива и agent session считается активной;
   - `restore_in_new_pane`, если pane пропала, но есть валидный resume handle;
   - `create_new_session`, если resume metadata нет или session уже не может
     быть продолжена.

Выходные данные:

- обновленный `session.json` с новыми agent metadata и новым
  `last_launch_outcome`;
- явные operator diagnostics:
  - `agent_session=new`;
  - `agent_session=reused`;
  - `agent_session=restored`;
- актуальные `zellij.tab_id` и `zellij.pane_id` после successful restore path.

## Configuration And Runtime Assumptions

- versioned `settings.yml` не должен получать repo-level флаг
  `restore_existing_session`; для первой версии restore является стандартной
  частью `run`, а не optional behavior;
- source of truth по issue status остается в GitHub Project;
- runtime остается source of truth только для session binding, launch metadata
  и agent resume handle;
- `zellij.launch_target` по-прежнему определяет, где должна появиться новая
  pane при restore path;
- `zellij.tab_name_template` по-прежнему влияет только на `tab`-ветку;
- отсутствие agent metadata в старом `session.json` не считается ошибкой:
  система делает controlled fallback на `create_new_session`;
- host `zellij` пользователя по-прежнему off-limits для автоматических тестов и
  destructive диагностики.

## Risks

- если `run` будет опираться только на наличие `session_uuid`, а не на
  agent-specific resume metadata, проблема дублей останется;
- если restore-path начнет blindly верить сохраненному `pane_id` без
  актуального `zellij`-introspection, runtime может ссылаться на уже мертвую
  pane;
- если `reuse_live_pane` все равно запускает `launch-agent.sh`, получится
  второй agent process поверх живой session;
- если `codex` resume handle нельзя детерминированно получить после первого
  старта, понадобится отдельный adapter-layer или fallback policy;
- если diagnostics не будут различать `created` и `restored`, оператор не
  сможет понять, была ли session продолжена или создана заново;
- если schema change в `session.json` не будет backward compatible, старые
  issues сломают re-entry path.

## Architecture Notes

- полезно ввести отдельную сущность вроде `ExistingSessionDecision`, чтобы
  branch logic `new/reuse/restore` не была размазана между `app.rs`,
  `runtime.rs` и `zellij.rs`;
- `zellij` adapter должен получить read-only introspection helpers, а не
  смешивать launch path и state detection в одной процедуре;
- `launch-agent.sh` нужно разделить на:
  - initial launch path;
  - resume launch path;
  при этом issue-level prompt, worktree prep и `bind-zellij-pane` остаются тем
  же contract boundary;
- для `claude` можно использовать deterministic session contract через
  `--session-id` / `--resume`;
- для `codex` нужен отдельный `AgentResumeAdapter`, который изолирует способ
  получения и последующего использования native session id;
- `reuse_live_pane` должен быть no-new-process path: максимум переключение на
  связанный tab и печать diagnostics, но не новый `exec codex`/`exec claude`.

## ADR Impact

По правилам
[../../../docs/documentation-process.md](../../../docs/documentation-process.md)
изменение затрагивает runtime contract, launcher semantics и взаимодействие с
внешними agent CLI, поэтому нужен отдельный ADR.

Предлагаемое решение для ADR:

- зафиксировать различие между issue-level `session_uuid` и agent-level
  resume metadata;
- зафиксировать decision matrix `new` / `reuse live pane` / `restore in new pane`;
- зафиксировать обязательное требование `run` не запускать второй agent process
  при живой уже связанной session;
- отдельно описать agent-specific adapter boundary для `codex` и `claude`.

Дополнительно в implementation потребуется синхронизировать:

- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md);
- [../../../docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md](../../../docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md).

## Alternatives Considered

1. Считать потерю pane эквивалентной смерти agent session.

   Отклонено: это противоречит user story issue и приводит к потере
   интерактивной истории, которая по ADR-0013 является значимой частью UX.

2. Хранить только `zellij` ids и не вводить agent-specific resume metadata.

   Отклонено: `zellij` ids позволяют найти pane, но не восстановить собственную
   session конкретного agent CLI в новой pane.

3. Использовать `session_uuid` как universal native agent session id для всех
   backend.

   Отклонено: `claude` и `codex` имеют разные resume contracts, и текущий
   внешний CLI-контракт не гарантирует универсальность такого подхода.

4. Делать restore только для `claude`, а `codex` всегда перезапускать заново.

   Пока отклонено как целевой контракт: issue описывает общесистемное поведение
   `run`, а не частный backend-specific workaround. Но это остается техническим
   риском на этапе реализации, если у `codex` не удастся получить
   deterministic resume handle приемлемым способом.

## Migration Or Rollout Notes

- schema `session.json` должна расширяться только optional-полями;
- старые issues без agent metadata должны оставаться runnable и получать
  metadata при первом новом успешном запуске;
- реализация должна появиться вместе с headless integration coverage, потому
  что вручную проверять restore path на host `zellij` нельзя;
- документация должна быть обновлена раньше или одновременно с кодом:
  сначала ADR и feature/runtime docs, затем код и тесты.
