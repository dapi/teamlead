# Stage 3: Как проверяем

Переход к этапу допустим только после того, как собраны оси `Что строим` и
`Как строим`.

Цель этапа:

- определить, как доказать корректность решения
- заполнить `03-how-we-verify.md`

Обязательный минимум:

- acceptance criteria
- test plan
- verification checklist

Используй rule-based выбор секций:

- `core`:
  - `Acceptance Criteria`
  - `Test Plan`
  - `Verification Checklist`

`conditional`:

- для `bug`:
  - `Regression Checks`
- для `feature`:
  - `Happy Path`
  - `Edge Cases`
- для `chore`:
  - `Operational Validation`

`scaling`:

- для `medium` и `large`:
  - `Failure Scenarios`
  - `Observability`

Акценты по типу проекта:

- для `product/UI` усиливай пользовательские сценарии, acceptance criteria и
  edge cases
- для `library/API` усиливай контрактные, совместимые и регрессионные проверки
- для `infra/platform` усиливай эксплуатационные проверки, отказные сценарии и
  наблюдаемость

На выходе документ должен:

- содержать все обязательные `core`-секции
- содержать релевантные `conditional` и `scaling`-секции
- быть пригодным как для ручной проверки, так и для будущей автоматизации

Документ должен быть пригоден как для ручной проверки человеком, так и как
основа для будущей автоматической реализации агентом.
