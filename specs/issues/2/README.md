# Issue 2: довести issue-analysis flow до реального создания SDD-артефактов

Статус: draft
Issue: https://github.com/dapi/ai-teamlead/issues/2
Тип задачи: `feature`
Тип проекта: `infra/platform`
Размер: `medium`
Последнее обновление: 2026-03-14

## Резюме

`run` и `launch-agent.sh` уже умеют claim-ить issue, поднимать analysis
worktree и запускать агента со staged prompts, но этого недостаточно, если
итог анализа не закреплен как предсказуемый versioned output contract.

Смысл этой задачи: довести flow до состояния, в котором реальный агент
формирует минимальный SDD-комплект в `specs/issues/<issue>/`, делает это
компактно для маленьких задач, соблюдает rule-based выбор секций для
`feature`/`bug`/`chore` и подтверждается хотя бы одним живым smoke-сценарием.

## Артефакты

### Что строим

- [01-what-we-build.md](./01-what-we-build.md)

### Как строим

- [02-how-we-build.md](./02-how-we-build.md)

### Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## Связанный контекст

- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../.ai-teamlead/flows/issue-analysis-flow.md](../../../.ai-teamlead/flows/issue-analysis-flow.md)
- [../../../.ai-teamlead/flows/issue-analysis/README.md](../../../.ai-teamlead/flows/issue-analysis/README.md)
- [../../../docs/adr/0017-minimal-sdd-artifact-set-for-issue-analysis.md](../../../docs/adr/0017-minimal-sdd-artifact-set-for-issue-analysis.md)
- [../../../docs/adr/0019-conditional-sections-by-task-type-project-type-and-size.md](../../../docs/adr/0019-conditional-sections-by-task-type-project-type-and-size.md)
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
