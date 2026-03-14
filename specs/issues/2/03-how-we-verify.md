# Issue 2: Как проверяем

Статус: draft
Последнее обновление: 2026-03-14

## Acceptance Criteria

- после реального `run` на issue в analysis worktree появляется versioned
  каталог `specs/issues/<issue>/`
- в каталоге существует минимальный SDD-комплект:
  `README.md`, `01-what-we-build.md`, `02-how-we-build.md`,
  `03-how-we-verify.md`
- комплект пригоден для human review и как вход в следующий implementation
  stage без ручной пересборки структуры
- для small issue не создаются лишние документы сверх минимального набора, если
  задача этого не требует
- для `feature` в продуктовой оси присутствуют `User Story` и `Use Cases`
- для `bug` и `chore` используются соответствующие conditional sections
- поведение flow и выходной контракт задокументированы и подтверждены smoke
  сценарием

## Ready Criteria

- системный SSOT и project-local flow не расходятся по минимальному комплекту
  артефактов и правилу выбора секций
- init templates выровнены с актуальным flow-контрактом
- есть как минимум одна проверка, которая подтверждает создание полного
  минимального комплекта в реальном launch path
- есть проверка или явный сценарий, который подтверждает compactness для small
  issue
- есть проверка или явный сценарий, который подтверждает task-type-specific
  секции для `feature`, `bug` и `chore`

## Invariants

- analysis output живет в `specs/issues/${ISSUE_NUMBER}/`
- минимальный набор из четырех файлов обязателен для каждой issue
- на каждую ось `Что строим`, `Как строим`, `Как проверяем` приходится минимум
  один документ
- `README.md` остается компактным индексом issue-спеки, а не свалкой всех
  деталей
- выбор секций делается по rule-based модели:
  task type + project type + task size
- launcher подготавливает execution context, но не подменяет работу агента по
  написанию SDD
- headless ограничения для `zellij`-related проверок соблюдаются

## Happy Path

### Happy Path 1. Feature issue проходит полный анализ

- `run` поднимает analysis worktree
- агент получает staged prompts и путь к каталогу артефактов
- в каталоге issue появляются четыре обязательных файла
- в `01-what-we-build.md` присутствуют `User Story` и `Use Cases`
- анализ завершается допустимым waiting-исходом

### Happy Path 2. Small issue остается компактной

- анализируется маленькая issue
- создается только минимальный SDD-комплект
- внутри файлов присутствуют только core и реально релевантные conditional
  секции

## Edge Cases

- issue не имеет labels, и task type выводится из текста
- small issue ошибочно тяготеет к medium-структуре и начинает разрастаться
- `bug` и `chore` используют одни и те же общие prompts, но должны получать
  разные conditional sections
- реальный агент формально создает все файлы, но пропускает одну из
  обязательных секций внутри документа

## Failure Scenarios

- launcher не создает каталог артефактов до старта агента
- агент завершает анализ без полного SDD-комплекта
- flow создает только один `README.md`, а остальные документы отсутствуют
- staged prompts и SSOT расходятся по названиям секций или обязательному
  минимуму
- `templates/init` отстают от актуального repo-local flow и создают устаревший
  contract layer в новых репозиториях

## Observability

Для диагностики должно быть видно:

- какой issue URL анализировался
- какой `session_uuid` связан с запуском
- какой `analysis branch` и `worktree` использовались
- какой `analysis_artifacts_dir` был передан агенту
- какие файлы появились в `specs/issues/<issue>/`
- завершился ли run с `plan-ready`, `needs-clarification` или `blocked`
- в какой точке произошел сбой, если комплект не собрался

## Test Plan

Документарные проверки:

- проверить, что `docs/issue-analysis-flow.md`,
  `./.ai-teamlead/flows/issue-analysis-flow.md` и staged prompts одинаково
  описывают минимальный комплект и логику выбора секций
- проверить, что init templates повторяют тот же contract layer

Integration tests:

- усилить stub-agent fixture так, чтобы он создавал не один `README.md`, а весь
  минимальный SDD-комплект
- добавить проверку, что `run` и/или `poll` доводят агента до каталога
  `specs/issues/<issue>/` и этот каталог содержит все четыре обязательных файла
- добавить targeted fixture coverage для как минимум одного `feature`, одного
  `bug` и одного `chore` результата
- если semantic content сложно валидировать автоматически, проверять хотя бы
  стабильные заголовки обязательных секций

Smoke tests:

- выполнить живой `ai-teamlead run <issue-url>` на реальной issue этого
  репозитория
- проверить, что в analysis worktree появился полный SDD-комплект
- проверить, что комплект читаем, структурирован по трем осям и не требует
  ручной пересборки
- отдельно зафиксировать, что small issue не была перегружена лишними
  документами

## Verification Checklist

- в `specs/issues/<issue>/` есть `README.md`
- в `specs/issues/<issue>/` есть `01-what-we-build.md`
- в `specs/issues/<issue>/` есть `02-how-we-build.md`
- в `specs/issues/<issue>/` есть `03-how-we-verify.md`
- `README.md` содержит резюме issue и ссылки на артефакты
- `01-what-we-build.md` содержит обязательные core-секции
- `02-how-we-build.md` содержит обязательные core-секции
- `03-how-we-verify.md` содержит `Acceptance Criteria`, `Ready Criteria`,
  `Invariants`, `Test Plan`, `Verification Checklist`
- для `feature` есть `User Story` и `Use Cases`
- для `bug` есть bug-specific секции
- для `chore` есть `Motivation`, `Operational Goal` и `Operational Validation`
- структура артефактов пригодна для human review и дальнейшей автоматизации
