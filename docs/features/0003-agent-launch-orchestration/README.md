# Feature 0003: agent launch orchestration

Статус: draft
Владелец: владелец репозитория
Последнее обновление: 2026-03-13

## Контекст

Эта feature описывает orchestration flow, который запускает агентскую analysis
session по issue.

Важно:

- `issue-analysis-flow` это project-local markdown prompt для агента
- эта feature описывает не сам prompt, а orchestration вокруг его запуска

Документ оформлен как каталог, потому что feature затрагивает `poll`, `run`,
`zellij`, branch/worktree lifecycle, launcher contract и corner cases вокруг
session/tab.

## Что строим

- [01-what-we-build.md](./01-what-we-build.md)

## Как строим

- [02-how-we-build.md](./02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## План реализации

- [04-implementation-plan.md](./04-implementation-plan.md)

## Зависимости

- [Feature 0001](../0001-ai-teamlead-cli/README.md) — `ai-teamlead`
  предоставляет `poll` и `run`, которые вызывают orchestration launch path
- [Feature 0002](../0002-repo-init/README.md) — `init` создает
  `./.ai-teamlead/launch-agent.sh` и `settings.yml`, которые использует
  orchestration

## Связанные документы

- [../../issue-analysis-flow.md](../../issue-analysis-flow.md)
- [../../adr/0005-cli-contract-for-poll-and-run.md](../../adr/0005-cli-contract-for-poll-and-run.md)
- [../../adr/0008-bind-issue-to-agent-session-uuid.md](../../adr/0008-bind-issue-to-agent-session-uuid.md)
- [../../adr/0014-zellij-launch-context-naming.md](../../adr/0014-zellij-launch-context-naming.md)
- [../../adr/0022-zellij-layout-contract-for-new-sessions.md](../../adr/0022-zellij-layout-contract-for-new-sessions.md)

## Открытые вопросы

- как именно конкретный репозиторий назовет analysis branch и worktree root
- нужна ли в следующей версии отдельная machine-readable обратная связь от
  `launch-agent.sh` в core orchestration

## Журнал изменений

### 2026-03-13

- создан каталог feature 0003 для orchestration flow запуска агента

### 2026-03-14

- добавлена ссылка на ADR по `zellij.layout` и fallback-path новой session
- добавлено требование заказчика: analysis tab должна выглядеть как родной tab
  текущей session
 - зафиксирован project-local template `.ai-teamlead/zellij/analysis-tab.kdl`
   как versioned contract для analysis tab
