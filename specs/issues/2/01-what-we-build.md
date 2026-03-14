# Issue 2: Что строим

Статус: draft
Последнее обновление: 2026-03-14

## Problem

В репозитории уже есть orchestration-слой, который:

- выбирает issue через `poll` или `run`
- подготавливает analysis branch и worktree
- запускает агента с project-local `issue-analysis-flow`

Но без явного и реально исполняемого output contract анализ остается
недостаточно надежным:

- агент может оставить результат только в чате или в неполном наборе файлов
- маленькие задачи могут обрастать лишним формальным шумом
- задачи разных типов могут оформляться разными случайными секциями
- следующий implementation stage не получает стабильный вход в виде
  versioned SDD-комплекта

Нужно сделать так, чтобы реальный запуск `issue-analysis-flow` устойчиво
производил минимальный набор артефактов в `specs/issues/<issue>/` и делал это
по rule-based правилам, а не по случайной форме конкретного ответа агента.

## Who Is It For

Основной пользователь:

- владелец репозитория или оператор, который запускает `ai-teamlead run` и
  ожидает предсказуемый analysis output

Дополнительно результат нужен:

- будущему implementation flow, который должен получать versioned входной
  SDD-комплект
- самому проекту `ai-teamlead`, который должен dogfood-ить flow на себе без
  ручной сборки документов постфактум
- владельцам внешних репозиториев, где `ai-teamlead` должен работать по тому
  же контракту

## Outcome

После выполнения задачи реальный analysis run:

- создает в `specs/issues/<issue>/` минимум четыре файла:
  `README.md`, `01-what-we-build.md`, `02-how-we-build.md`,
  `03-how-we-verify.md`
- оформляет SDD-комплект так, чтобы он был пригоден для human review и для
  следующего implementation stage
- для small issue остается компактным и не создает лишних документов сверх
  минимума
- для `feature`, `bug` и `chore` использует предсказуемые rule-based секции
- имеет задокументированный и воспроизводимый smoke-сценарий

## Scope

В scope этой issue входит:

- зафиксировать минимальный обязательный набор analysis-артефактов и их
  минимальное содержание
- выровнять project-local entrypoint prompt и staged prompts под этот контракт
- обеспечить, что реальный агент получает путь к каталогу артефактов и создает
  комплект в versioned виде внутри analysis worktree
- проверить compactness для small issue
- проверить rule-based выбор секций для `feature`, `bug` и `chore`
- зафиксировать verification path, включающий живой smoke-run
- при необходимости усилить project-local contract и init templates, чтобы
  новые репозитории получали тот же flow по умолчанию

## Non-Goals

Вне scope этой issue:

- автоматическая реализация задачи после анализа
- построение большого фиксированного пакета документов на каждую issue
- перенос content-logic выбора секций в hardcoded Rust-правила вместо
  markdown-contract layer
- полная автоматическая semantic-валидация любого свободного LLM-ответа
- изменение source of truth по статусам issue или введение локальной базы
  состояния

## Constraints And Assumptions

- source of truth по состоянию issue остается в GitHub Project, а не в локальных
  файлах
- project-local flow должен оставаться versioned и храниться в репозитории
- `launch-agent.sh` подготавливает worktree и каталог артефактов до старта
  агента, но не берет на себя написание самих SDD-документов
- минимум из четырех файлов обязателен даже для маленькой issue
- дополнительные документы допустимы только по реальной необходимости
- выбор секций зависит минимум от трех факторов:
  типа задачи, размера задачи и типа проекта
- первый релиз решения должен опираться на живой dogfooding, а не только на
  synthetic fixtures

## User Story

Как владелец репозитория, я хочу запустить `ai-teamlead run` на реальной issue
и получить в analysis worktree компактный, versioned и структурированный
SDD-комплект, чтобы можно было быстро отдать его на review и использовать как
вход в следующий implementation stage без ручной доработки.

## Use Cases

### Use Case 1. Реальная feature issue

- оператор запускает `ai-teamlead run <issue-url>`
- launcher подготавливает `analysis/issue-N` и `specs/issues/N`
- агент проходит staged prompts по трем осям
- в `01-what-we-build.md` появляются `User Story` и `Use Cases`
- по завершении в каталоге issue существует полный минимальный SDD-комплект

### Use Case 2. Маленькая bug issue

- анализируется маленькая bug issue
- агент все равно создает четыре обязательных файла
- внутри документов остаются только core и bug-specific секции
- лишние дополнительные документы не создаются

### Use Case 3. Chore issue для infra/platform

- задача классифицируется как `chore`
- продуктовая ось усиливается секциями `Motivation` и `Operational Goal`
- verification-ось включает `Operational Validation`
- комплект остается пригодным как для человека, так и для следующего агента

## Dependencies

- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
  как системный SSOT анализа issue
- [../../../.ai-teamlead/flows/issue-analysis-flow.md](../../../.ai-teamlead/flows/issue-analysis-flow.md)
  как project-local entrypoint prompt
- staged prompts в
  [../../../.ai-teamlead/flows/issue-analysis/](../../../.ai-teamlead/flows/issue-analysis/)
- [../../../docs/adr/0017-minimal-sdd-artifact-set-for-issue-analysis.md](../../../docs/adr/0017-minimal-sdd-artifact-set-for-issue-analysis.md)
  для минимального набора артефактов
- [../../../docs/adr/0019-conditional-sections-by-task-type-project-type-and-size.md](../../../docs/adr/0019-conditional-sections-by-task-type-project-type-and-size.md)
  для rule-based выбора секций
- `launch-agent.sh`, `settings.yml` и init templates, которые должны передавать
  агенту корректный execution context
- headless integration/smoke окружение, безопасное для `zellij`-related
  проверок
