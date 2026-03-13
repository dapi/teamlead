# ADR-0003: Статусы GitHub Project как источник истины

Статус: accepted
Дата: 2026-03-13

## Контекст

Для `issue-analysis-flow` нужно было определить, где хранится источник истины
по состоянию issue.

Рассматривались варианты:

- постоянный локальный state
- локальная база или state file
- labels/comments в GitHub issue
- статусы в default GitHub Project

Требования:

- состояние должно быть видно снаружи
- оно не должно зависеть от локальной машины
- оно должно быть удобно для ручного контроля
- flow должен быть переносим между репозиториями

## Решение

Источником истины по состоянию issue считается поле статуса в настроенном
default GitHub Project.

Для `issue-analysis-flow` используются статусные состояния:

- `Backlog`
- `Analysis In Progress`
- `Waiting for Clarification`
- `Waiting for Plan Review`
- `Ready for Implementation`
- `Analysis Blocked`

Постоянный локальный runtime state не используется как источник истины.

## Последствия

Плюсы:

- состояние видно прямо в GitHub
- статусы доступны человеку без локального доступа к машине
- проще избежать рассинхронизации между кодом и внешним состоянием
- flow лучше переносится между репозиториями

Минусы:

- требуется надежная интеграция с GitHub Project API
- любые ошибки смены статуса нужно обрабатывать явно
- модель flow зависит от корректно настроенного проекта

## Связанные документы

- [README.md](/home/danil/code/teamlead/README.md)
- [docs/issue-analysis-flow.md](/home/danil/code/teamlead/docs/issue-analysis-flow.md)
