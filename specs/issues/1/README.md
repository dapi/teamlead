# Issue 1: foreground daemon loop для `poll`

Статус: draft

## Резюме

Issue закрывает текущий разрыв между документированным контрактом `ai-teamlead daemon`
и фактическим поведением кода: сейчас команда `daemon` только инициализирует
execution context и печатает диагностику, но не выполняет повторяющиеся polling
cycles.

Целевой результат анализа: описать минимальные изменения, при которых `daemon`
переиспользует существующий `poll`-path, непрерывно работает в foreground,
соблюдает `runtime.poll_interval_seconds` и не становится непригодным после
пустого или ошибочного цикла.

GitHub issue: https://github.com/dapi/ai-teamlead/issues/1

## Артефакты

- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

## Связанный контекст

- [../../../docs/features/0001-ai-teamlead-daemon/README.md](../../../docs/features/0001-ai-teamlead-daemon/README.md)
- [../../../docs/features/0001-ai-teamlead-daemon/04-implementation-plan.md](../../../docs/features/0001-ai-teamlead-daemon/04-implementation-plan.md)
- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)

