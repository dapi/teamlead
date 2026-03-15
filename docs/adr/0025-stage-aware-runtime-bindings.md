# ADR-0025: stage-aware runtime bindings для issue session

Статус: accepted, частично superseded by ADR-0028
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
- первая версия post-merge reconciliation предполагала более богатую semantic
  роль runtime, чем в итоге допустила GitHub-first модель.

## Решение

Runtime model становится stage-aware.

Минимальный контракт:

- `sessions/<session_uuid>/session.json` получает поле `stage`;
- `issues/<issue_number>.json` хранит binding отдельно по stage;
- для каждого stage у одной issue допускается не более одного активного
  binding;
- analysis и implementation binding не перезаписывают друг друга;
- implementation session дополнительно может хранить `stage_branch`,
  `stage_worktree_root` и `stage_artifacts_dir`.

После принятия [ADR-0028](./0028-github-first-reconcile-and-runtime-cache-only.md)
это решение сохраняется, но в уточненном виде:

- stage-aware bindings остаются принятым runtime contract;
- runtime не является semantic source of truth по состоянию issue;
- поля вроде `tracked_pr_*` и `last_known_flow_status` не входят в обязательный
  semantic contract и могут существовать только как optional cache/diagnostic
  metadata.

Минимальная форма issue index:

```json
{
  "issue_number": 5,
  "bindings": {
    "analysis": "session-uuid-1",
    "implementation": "session-uuid-2"
  },
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
- решение о tracked PR metadata и `last_known_flow_status` вынесено на
  повторный пересмотр

### 2026-03-15

- ADR сохранен в статусе `accepted` для stage-aware binding model
- [ADR-0028](./0028-github-first-reconcile-and-runtime-cache-only.md)
  частично supersede-ит прежние допущения о semantic роли runtime
- хранение `tracked_pr_number`, `tracked_pr_url` и использование
  `last_known_flow_status` как обязательного semantic state больше не входят в
  этот ADR
