# ADR-0025: stage-aware runtime bindings для issue session

Статус: accepted
Дата: 2026-03-14

## Контекст

Текущая runtime model проектировалась только для analysis stage:

- одна issue связана с одним `session_uuid`;
- `issues/<issue_number>.json` хранит один binding;
- `sessions/<session_uuid>/session.json` не различает stage.

Для implementation stage этого недостаточно:

- analysis binding нельзя терять после перехода к coding stage;
- implementation stage нужен собственный reusable binding;
- `run` должен понимать, какой binding искать при re-entry.
- post-merge reconciliation требует хранить identity tracked PR и workspace
  metadata после `ready-for-ci` / `ready-for-review`.

## Решение

Runtime model становится stage-aware.

Минимальный контракт:

- `sessions/<session_uuid>/session.json` получает поле `stage`;
- `issues/<issue_number>.json` хранит binding отдельно по stage;
- для каждого stage у одной issue допускается не более одного активного
  binding;
- analysis и implementation binding не перезаписывают друг друга;
- implementation session дополнительно может хранить `stage_branch`,
  `stage_worktree_root`, `stage_artifacts_dir`, `tracked_pr_number` и
  `tracked_pr_url`.

Минимальная форма issue index:

```json
{
  "issue_number": 5,
  "bindings": {
    "analysis": "session-uuid-1",
    "implementation": "session-uuid-2"
  },
  "last_known_flow_status": "Implementation In Progress",
  "updated_at": "2026-03-14T12:00:00Z"
}
```

## Последствия

Плюсы:

- сохраняется история analysis stage;
- implementation stage получает собственный re-entry context;
- `run` может искать binding по stage, а не по неявным эвристикам;
- схема расширяется и на будущие stage при необходимости.

Минусы:

- нужно мигрировать существующий runtime format;
- возрастает сложность чтения и записи issue index;
- тесты на runtime schema придется обновить.

## Альтернативы

### 1. Отдельные runtime директории по stage

Отклонено.

Это возможно, но в первой версии избыточно и сильнее разносит связанную
информацию по дереву `.git/.ai-teamlead/`.

### 2. Переиспользовать analysis binding для implementation stage

Отклонено.

Это ломает separation между stage и делает re-entry implementation flow
неоднозначным.

## Связанные документы

- [../issue-analysis-flow.md](../issue-analysis-flow.md)
- [../issue-implementation-flow.md](../issue-implementation-flow.md)
- [../features/0001-ai-teamlead-cli/05-runtime-artifacts.md](../features/0001-ai-teamlead-cli/05-runtime-artifacts.md)
- [./0008-bind-issue-to-agent-session-uuid.md](./0008-bind-issue-to-agent-session-uuid.md)

## Журнал изменений

### 2026-03-14

- runtime binding обобщен до stage-aware модели
- runtime schema расширена tracked PR metadata и workspace coordinates для
  post-merge lifecycle
