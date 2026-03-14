# ADR-0024: stage-aware dispatch внутри `run`

Статус: accepted
Дата: 2026-03-14

## Контекст

До этого `run` в документации и коде был analysis-only issue-level entrypoint.

Это вступило в конфликт с ожидаемым пользовательским контрактом:

- оператор не хочет выбирать отдельную top-level команду для разных stage;
- `run <issue>` должен сам понимать, на какой стадии находится issue;
- analysis и implementation flow должны оставаться разными SSOT и разными
  execution path.

## Решение

`run` остается единственным публичным issue-level entrypoint.

При запуске `run`:

- читается текущий status issue в GitHub Project;
- по status выбирается stage:
  - analysis lifecycle;
  - implementation lifecycle;
- после этого `run` вызывает stage-specific orchestration path.

Stage dispatch не переносится в prompt и не оформляется как отдельная top-level
CLI-команда.

## Последствия

Плюсы:

- пользовательский контракт остается простым;
- `poll` продолжает переиспользовать тот же `run` path;
- analysis и implementation flow остаются раздельными каноническими
  документами;
- логика выбора stage концентрируется в одном месте.

Минусы:

- `run` становится сложнее и требует явного mapping статусов по stage;
- documentation и runtime model нужно синхронно обновить;
- stage-specific ошибки теперь нужно диагностировать внутри одного entrypoint.

## Альтернативы

### 1. Отдельная команда `implement`

Отклонено.

Это расходится с ожидаемым пользовательским контрактом и дублирует issue-level
entrypoint.

### 2. Один multi-stage prompt вместо dispatch в `run`

Отклонено.

Это смешивает orchestration и prompt layer и делает status-based routing
неявной.

## Связанные документы

- [../issue-analysis-flow.md](../issue-analysis-flow.md)
- [../issue-implementation-flow.md](../issue-implementation-flow.md)
- [../features/0001-ai-teamlead-cli/README.md](../features/0001-ai-teamlead-cli/README.md)
- [../features/0004-issue-implementation-flow/README.md](../features/0004-issue-implementation-flow/README.md)

## Журнал изменений

### 2026-03-14

- зафиксирован единый `run <issue>` как stage-aware dispatcher
