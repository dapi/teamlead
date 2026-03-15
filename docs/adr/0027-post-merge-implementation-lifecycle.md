# ADR-0027: post-merge lifecycle остается частью `issue-implementation-flow`

Статус: на пересмотре, см. ADR-0028
Дата: 2026-03-14

## Контекст

После merge implementation PR первая версия `issue-implementation-flow`
останавливалась на `Waiting for Code Review`.

Это оставляло незафиксированными несколько ключевых вопросов:

- какой статус завершает issue после merge implementation PR;
- должен ли post-merge path быть отдельным flow или terminal веткой текущего
  implementation lifecycle;
- как связать merge PR, закрытие issue и GitHub Project status;
- какие runtime/worktree/branch cleanup действия допустимы после merge.

## Решение

Минимальный post-merge lifecycle остается частью
`issue-implementation-flow`.

Принятые правила:

- terminal project status после merge tracked implementation PR называется
  `Done`;
- `run <issue>` при статусе `Waiting for Code Review` может выполнить
  post-merge reconciliation без нового coding launch, если implementation PR
  уже merged;
- `internal complete-stage --stage implementation --outcome merged`
  используется как канонический terminal finalization path;
- merged finalization переводит project item в `Done`, закрывает GitHub issue и
  выполняет best-effort cleanup implementation worktree/local branch;
- cleanup warning не откатывает terminal business result и не возвращает issue
  в активный статус.

## Последствия

Плюсы:

- implementation lifecycle получает детерминированное завершение;
- merge tracked PR больше не оставляет issue и project status в подвешенном
  состоянии;
- post-merge reconciliation не требует отдельного operator-facing flow для MVP.

Минусы:

- первая версия решения делала runtime богаче и пыталась хранить tracked PR
  metadata;
- `run` и `complete-stage` получают дополнительную branch post-merge логики;
- для legacy issues нужен явный GitHub-first reconcile path.

## Альтернативы

### 1. Отдельный `issue-post-merge-flow`

Отклонено для MVP.

Это добавляет новый stage и новый prompt entrypoint без достаточной пользы для
базового post-merge contract.

### 2. Закрывать issue по любому merged PR, связанному с issue

Отклонено.

Это создает риск закрытия issue по неверному PR и противоречит требованию
явной канонической идентификации implementation PR.

## Связанные документы

- [../issue-implementation-flow.md](../issue-implementation-flow.md)
- [../features/0004-issue-implementation-flow/README.md](../features/0004-issue-implementation-flow/README.md)
- [./0025-stage-aware-runtime-bindings.md](./0025-stage-aware-runtime-bindings.md)
- [./0026-stage-aware-complete-stage.md](./0026-stage-aware-complete-stage.md)

## Журнал изменений

### 2026-03-14

- принят минимальный post-merge lifecycle в составе `issue-implementation-flow`

### 2026-03-15

- статус ADR переведен в `на пересмотре`
- механизм `tracked PR metadata in runtime` вынесен на повторный пересмотр в
  [ADR-0028](./0028-github-first-reconcile-and-runtime-cache-only.md)
