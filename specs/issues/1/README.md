# Issue 1: foreground `loop` поверх `poll`

Статус: draft

## Резюме

Issue исторически описывала foreground `daemon` loop, но канонический
CLI-контракт проекта теперь использует термин `loop`.

Текущий смысл этого task-артефакта: описать минимальные изменения, при которых
публичная команда `loop` переиспользует существующий `poll`-path, непрерывно
работает в foreground, соблюдает `runtime.poll_interval_seconds` и не
становится непригодной после пустого или ошибочного цикла.

GitHub issue: https://github.com/dapi/ai-teamlead/issues/1

## Артефакты

- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

## Связанный контекст

- [../../../docs/features/0001-ai-teamlead-daemon/README.md](../../../docs/features/0001-ai-teamlead-daemon/README.md)
- [../../../docs/features/0001-ai-teamlead-daemon/04-implementation-plan.md](../../../docs/features/0001-ai-teamlead-daemon/04-implementation-plan.md)
- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/adr/0021-cli-contract-poll-run-loop.md](../../../docs/adr/0021-cli-contract-poll-run-loop.md)
