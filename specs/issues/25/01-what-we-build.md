# Issue 25: Что строим

Статус: draft
Последнее обновление: 2026-03-15

## Problem

Сейчас у `ai-teamlead` уже есть durable binding `issue <-> session_uuid`, но
этот binding фиксирует только локальный issue/session contract и `zellij`
метаданные. Он не гарантирует восстановление реальной agent session.

Из-за этого после потери pane возникают два пробела:

- повторный `run` не умеет понять, что живая agent session уже существует и ее
  нужно не создавать заново, а восстановить;
- повторный `run` идет по обычному launcher path и может запустить второй
  независимый agent process для той же issue.

Проблема особенно заметна для waiting/re-entry сценариев, где по
[../../../docs/adr/0013-agent-session-history-as-dialog-source.md](../../../docs/adr/0013-agent-session-history-as-dialog-source.md)
источником диалога считается именно история agent session.

## Who Is It For

- оператор, который повторно запускает `ai-teamlead run <issue>` после потери
  или удаления pane;
- разработчик, который работает с analysis/implementation issue через `zellij`
  и ожидает безопасный re-entry без дублирования agent session;
- владелец репозитория, который хочет предсказуемый launcher contract и
  понятные diagnostics для `created` vs `restored`.

## Outcome

Нужен runtime и launcher contract, в котором:

- `run` перед созданием новой agent session проверяет, существует ли уже
  resume-capable session для этой issue и stage;
- если связанная pane еще жива, `run` не запускает второй agent process и
  использует существующий live context;
- если agent session жива, но старая pane недоступна, `run` создает новую pane
  и в ней восстанавливает ту же agent session;
- если resume metadata отсутствует или session действительно умерла, текущий
  path создания новой session сохраняется;
- пользовательский вывод и launch diagnostics явно различают сценарии:
  `new session created`, `existing pane reused`, `existing session restored`;
- invariant `одна issue/stage -> одна живая agent session` не нарушается из-за
  потери `zellij`-pane.

## Scope

В текущую задачу входит:

- фиксация distinction между issue-level `session_uuid` и agent-level
  resume metadata;
- расширение runtime contract для хранения agent-specific resume token или
  эквивалентного resume handle;
- добавление pre-launch reconcile в `run`, который различает:
  - новую session;
  - reuse живой pane;
  - restore в новую pane;
- определение правил live-check для ранее записанных `zellij.session_id`,
  `zellij.tab_id`, `zellij.pane_id`;
- интеграция restore path с текущими `pane` / `tab` launcher semantics;
- обновление diagnostics, runtime metadata, docs и headless verification path;
- покрытие регрессий тестами или интеграционными headless-сценариями.

## Non-Goals

В текущую задачу не входит:

- универсальное восстановление любого внешнего multiplexer кроме `zellij`;
- восстановление agent session без поддерживаемого resume contract со стороны
  конкретного agent CLI;
- автоматическое лечение поврежденного runtime state, если и локальный binding,
  и agent resume metadata отсутствуют;
- поддержка нескольких независимых живых agent session для одной issue и stage;
- запуск опасных `zellij`-проверок в host session пользователя;
- redesign flow analysis/implementation или их статусной модели.

## Constraints And Assumptions

- invariant из
  [../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md](../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md)
  про durable binding `issue <-> session_uuid` сохраняется;
- `session_uuid` остается issue/stage binding идентификатором, но не обязан
  совпадать с native session id конкретного agent CLI;
- локально проверено на 2026-03-15:
  - `codex` CLI поддерживает `resume [SESSION_ID]`;
  - `claude` CLI поддерживает `--resume` и `--session-id <uuid>`;
- для `claude` детерминированный resume contract уже существует, а для `codex`
  в первой версии потребуется отдельный путь получения и сохранения
  resume-token после первого старта;
- текущий `zellij` CLI позволяет адресовать tab по `tab_id`, но не дает
  очевидного прямого focus-by-`pane_id` path, поэтому reuse живой pane должен
  проектироваться как минимум без дубля agent process и с best-effort фокусом
  на связанный tab;
- headless/Docker path остается единственно допустимым для автоматической
  проверки `zellij`-сценариев.

## User Story

Как оператор, который повторно входит в уже начатую задачу через
`ai-teamlead run <issue>`, я хочу, чтобы система восстановила уже существующую
agent session в текущем `zellij`-контексте или в новой pane, а не создавала
вторую независимую session и не теряла историю диалога.

## Use Cases

1. Оператор запускает `run`, связанная pane все еще жива, и система повторно
   использует уже существующий live context без запуска нового agent process.
2. Оператор запускает `run`, старая pane удалена, но agent session еще жива, и
   система создает новую pane, в которой выполняет `resume` той же session.
3. Оператор запускает `run`, и никакой resume-capable agent session для issue
   уже нет; система идет по обычному path создания новой session.
4. Issue использует `zellij.launch_target = pane`, и restore-path должен
   восстановить session в shared tab, не ломая существующий launcher contract.
5. Issue использует `zellij.launch_target = tab`, и restore-path должен
   учитывать issue-aware tab naming, если он уже задан в repo-local config.

## Dependencies

- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
  задает текущий launcher contract и прямо оставляет auto-restore вне scope
  первой версии;
- [../../../docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md](../../../docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md)
  задает текущий runtime layout, который нужно расширить без нарушения
  backward compatibility;
- [../../../docs/adr/0013-agent-session-history-as-dialog-source.md](../../../docs/adr/0013-agent-session-history-as-dialog-source.md)
  делает историю agent session важной частью user experience;
- [../../../docs/adr/0032-zellij-launch-target-pane-tab.md](../../../docs/adr/0032-zellij-launch-target-pane-tab.md)
  фиксирует различие `pane` / `tab`, которое restore path обязан уважать;
- [../../../docs/adr/0031-zellij-issue-aware-tab-name-template.md](../../../docs/adr/0031-zellij-issue-aware-tab-name-template.md)
  задает naming semantics для `tab`-ветки и не должен быть сломан восстановлением.
