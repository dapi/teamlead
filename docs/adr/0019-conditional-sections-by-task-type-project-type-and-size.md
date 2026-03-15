# ADR-0019: условные секции по типу задачи, типу проекта и размеру

Статус: accepted
Дата: 2026-03-13

## Контекст

Versioned SDD-комплект для issue-analysis не может быть полностью одинаковым
для всех задач.

Секции внутри документов зависят как минимум от:

- типа задачи (`feature`, `bug`, `chore`)
- типа проекта (product/UI, library/API, infra/platform)
- размера задачи (`small`, `medium`, `large`)

Если сделать один жесткий шаблон для всех случаев, маленькие задачи будут
перегружены формальным шумом, а большие будут недоописаны.

## Решение

Для документов `issue-analysis` вводится rule-based модель секций:

- `core` — обязательны всегда
- `conditional` — обязательны только для релевантных типов задач или проектов
- `scaling` — добавляются для `medium` и `large` задач

Это правило применяется к:

- `README.md`
- `01-what-we-build.md`
- `02-how-we-build.md`
- `03-how-we-verify.md`

## Базовая схема

`README.md`

- всегда содержит только компактный индекс issue-спеки

`01-what-we-build.md`

- `core`: `Problem`, `Who Is It For`, `Scope`, `Non-Goals`
- `conditional`:
  - `feature`: `User Story`, `Use Cases`
  - `bug`: `Observed Behavior`, `Expected Behavior`, `Impact`
  - `chore`: `Motivation`, `Operational Goal`
- `scaling`:
  - `medium|large`: `Constraints`, `Dependencies`

`02-how-we-build.md`

- `core`: `Approach`, `Affected Areas`, `Interfaces And Data`, `Risks`
- `conditional`:
  - внешний integration surface: `External Interfaces`
  - значимая архитектурная смена: `Architecture Notes`
  - решение уровня проекта: `ADR Impact`
- `scaling`:
  - `medium|large`: `Alternatives Considered`, `Migration Or Rollout Notes`

`03-how-we-verify.md`

- `core`: `Acceptance Criteria`, `Test Plan`, `Verification Checklist`
- `conditional`:
  - `bug`: `Regression Checks`
  - `feature`: `Happy Path`, `Edge Cases`
  - `chore`: `Operational Validation`
- `scaling`:
  - `medium|large`: `Failure Scenarios`, `Observability`

## Последствия

Плюсы:

- маленькие issue не перегружаются
- большие issue получают больше структуры
- выбор секций становится rule-based, а не случайным

Минусы:

- prompt-layer становится чуть сложнее
- агент должен сначала оценить task type, project type и size

## Связанные документы

- [docs/issue-analysis-flow.md](../issue-analysis-flow.md)
- [docs/adr/0017-minimal-sdd-artifact-set-for-issue-analysis.md](./0017-minimal-sdd-artifact-set-for-issue-analysis.md)

## Журнал изменений

### 2026-03-13

- зафиксирована rule-based модель выбора секций внутри analysis artifacts
