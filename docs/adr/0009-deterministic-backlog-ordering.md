# ADR-0009: Детерминированный порядок выбора issue из `Backlog`

Статус: accepted
Дата: 2026-03-13

## Контекст

Для команды `poll` и daemon loop нужно было определить, в каком порядке
выбирать issue, если в `Backlog` находится несколько подходящих задач.

Требования:

- порядок должен быть детерминированным
- правило должно быть простым для реализации через `gh`
- поведение должно быть предсказуемым в smoke-проверках и при ручной отладке

## Решение

В MVP выбор следующей issue из `Backlog` выполняется по возрастанию номера
issue.

То есть при прочих равных первой выбирается верхняя issue в порядке GitHub
Project.

## Последствия

Плюсы:

- правило простое и прозрачное
- не зависит от отдельного project ordering API
- легко проверить и воспроизвести

Минусы:

- порядок не обязательно совпадает с визуальным порядком карточек в GitHub
  Project
- позже может потребоваться отдельное решение, если понадобится project-native
  ordering

## Связанные документы

- [docs/issue-analysis-flow.md](/home/danil/code/teamlead/docs/issue-analysis-flow.md)
- [docs/features/0001-ai-teamlead-daemon/03-how-we-verify.md](/home/danil/code/teamlead/docs/features/0001-ai-teamlead-daemon/03-how-we-verify.md)
