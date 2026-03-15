# Issue 25: восстановление существующей агентской сессии в новой `zellij`-панели

Статус: draft
Тип задачи: `feature`
Тип проекта: `infra/platform`
Размер: `medium`
Последнее обновление: 2026-03-15

## Контекст

Issue: `feat(run): восстановление существующей агентской сессии в новой zellij-панели`

- GitHub: https://github.com/dapi/ai-teamlead/issues/25
- Analysis branch: `analysis/issue-25`
- Session UUID: `16d2559f-df44-49df-a014-75e400e9124e`

Сейчас `run` уже умеет переиспользовать существующий `session_uuid` issue, но
этого недостаточно для реального resume user flow:

- runtime хранит binding `issue <-> session_uuid` и `zellij` ids;
- повторный `run` всегда запускает новый launcher path;
- `launch-agent.sh` не использует `session_uuid` как resume-token для реального
  агента;
- если старая pane исчезла, а агентская session все еще жива, система не умеет
  корректно различить сценарии `reuse live pane`, `restore in new pane` и
  `start brand new session`.

Цель анализа: зафиксировать минимальный, но проверяемый контракт, в котором
`run` сначала пытается восстановить уже существующую agent session, а вторую
независимую agent session для той же issue не создает.

## Артефакты

## Что строим

- [01-what-we-build.md](./01-what-we-build.md)

## Как строим

- [02-how-we-build.md](./02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## План имплементации

- [04-implementation-plan.md](./04-implementation-plan.md)

## Связанный контекст

- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/implementation-plan.md](../../../docs/implementation-plan.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
- [../../../docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md](../../../docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md)
- [../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md](../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md)
- [../../../docs/adr/0013-agent-session-history-as-dialog-source.md](../../../docs/adr/0013-agent-session-history-as-dialog-source.md)
- [../../../docs/adr/0023-zellij-session-target-resolution.md](../../../docs/adr/0023-zellij-session-target-resolution.md)
- [../../../docs/adr/0031-zellij-issue-aware-tab-name-template.md](../../../docs/adr/0031-zellij-issue-aware-tab-name-template.md)
- [../../../docs/adr/0032-zellij-launch-target-pane-tab.md](../../../docs/adr/0032-zellij-launch-target-pane-tab.md)

## Вывод анализа

Информации в issue и в текущем runtime-коде достаточно, чтобы готовить план
реализации без дополнительных вопросов пользователю.

Предлагаемый контракт:

- `run` должен различать три исхода:
  - создается новая agent session;
  - переиспользуется уже живая pane без запуска второго agent process;
  - создается новая pane и в ней восстанавливается существующая agent session;
- одного `session_uuid` недостаточно, поэтому runtime должен хранить
  agent-specific resume metadata отдельно от issue binding;
- `zellij`-introspection должна определять, жива ли ранее связанная pane, tab и
  session;
- diagnostics и тесты обязаны явно различать сценарии `created`, `reused`,
  `restored`;
- для реализации нужен отдельный ADR, потому что задача меняет runtime contract
  и launcher semantics.

Блокирующих вопросов по текущему issue не выявлено.
