# ADR-0026: stage-aware `internal complete-stage`

Статус: accepted, частично superseded by ADR-0028
Дата: 2026-03-14

## Контекст

`internal complete-stage` уже используется как analysis finalization contract.

Implementation stage тоже требует одного детерминированного completion path для:

- commit и push;
- draft PR и ready-for-review transitions;
- переводов issue между implementation statuses;
- сохранения stage-specific runtime state.

Создавать отдельную unrelated команду для каждого stage нежелательно:

- тогда finalization semantics расползается по CLI;
- prompt-слой снова начнет знать слишком много о git/gh деталях;
- stage handling перестанет быть единообразным.

## Решение

`internal complete-stage` расширяется до stage-aware контракта.

Канонический вид:

```text
ai-teamlead internal complete-stage <session_uuid> --stage <stage> --outcome <outcome> --message <message>
```

Правила:

- `--stage analysis` для обратной совместимости может быть значением по
  умолчанию;
- `--stage implementation` включает implementation-specific outcome handling;
- prompt не выполняет `git commit`, `git push`, `gh pr create`,
  `gh pr ready`, `gh pr checks` самостоятельно.

После принятия [ADR-0028](./0028-github-first-reconcile-and-runtime-cache-only.md)
граница решения уточнена:

- `complete-stage` остается каноническим stage-aware finalization contract;
- но он не должен полагаться на runtime как на обязательный semantic source of
  truth;
- implementation-specific reconcile перед terminal decisions должен
  использовать GitHub-first observed state.

Implementation outcomes:

- `ready-for-ci`
- `ready-for-review`
- `merged`
- `needs-rework`
- `blocked`

## Последствия

Плюсы:

- один finalization contract для разных stage;
- stage-aware status transitions и VCS operations остаются инкапсулированными в
  CLI;
- analysis flow сохраняет совместимость;
- проще тестировать и диагностировать completion behavior.

Минусы:

- CLI parsing и service layer становятся сложнее;
- нужно продумать backward compatibility для analysis prompts;
- реализация должна различать stage-specific outcome vocabulary.
- часть implementation-specific reconciliation logic не должна зависеть от
  runtime как от источника истины.

## Альтернативы

### 1. Отдельная команда `complete-implementation-stage`

Отклонено.

Это дублирует lifecycle semantics и усложняет prompt contracts.

### 2. Оставить implementation finalization в prompt

Отклонено.

Это уже показало себя ненадежным даже для analysis stage.

## Связанные документы

- [../issue-analysis-flow.md](../issue-analysis-flow.md)
- [../issue-implementation-flow.md](../issue-implementation-flow.md)
- [../features/0004-issue-implementation-flow/README.md](../features/0004-issue-implementation-flow/README.md)
- [./0020-agent-session-completion-signal.md](./0020-agent-session-completion-signal.md)

## Журнал изменений

### 2026-03-14

- `complete-stage` расширен до stage-aware контракта для implementation flow
- добавлен implementation outcome `merged` для terminal post-merge finalization

### 2026-03-15

- ADR сохранен в статусе `accepted` для stage-aware finalization contract
- [ADR-0028](./0028-github-first-reconcile-and-runtime-cache-only.md)
  частично supersede-ит только ту часть решения, где implementation reconcile
  мог зависеть от runtime как от источника истины
