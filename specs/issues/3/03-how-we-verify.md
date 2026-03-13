# Issue 3: Как проверяем

Статус: draft
Последнее обновление: 2026-03-13

## Acceptance Criteria

Решение считается принятым, если одновременно выполнено следующее:

- был выполнен реальный dogfooding run на issue этого репозитория
- запуск шел через настоящий `ai-teamlead run <issue-url>`, а не через
  synthetic test harness
- в `specs/issues/3/` существует минимальный SDD-комплект из четырех файлов
- по итогам run получен один из допустимых результатов:
  `Waiting for Clarification`, `Waiting for Plan Review` или документированный
  blocking gap
- найденные UX/orchestration gaps сформулированы достаточно конкретно, чтобы их
  можно было вынести в follow-up issues без потери контекста

## Test Plan

Нужный набор проверок:

1. Подтвердить, что issue #3 существует в GitHub Project и находится в
   допустимом состоянии для `run`.
2. Выполнить запуск через `ai-teamlead run <issue-url>` или эквивалентный
   ручной путь текущего launcher contract.
3. Проверить, что launcher использует `analysis/issue-3` и
   `specs/issues/3`.
4. Проверить, что analysis flow реально дошел до оформления артефактов по трем
   осям.
5. Проверить, что итоговый исход зафиксирован как valid waiting outcome или как
   явный blocker.

## Verification Checklist

- issue URL в артефактах совпадает с `https://github.com/dapi/ai-teamlead/issues/3`
- task type зафиксирован как `chore`
- project type зафиксирован как `infra/platform`
- размер задачи зафиксирован как `medium`
- в `01-what-we-build.md` есть `Problem`, `Who Is It For`, `Scope`,
  `Non-Goals`, `Motivation`, `Operational Goal`, `Constraints`,
  `Dependencies`
- в `02-how-we-build.md` есть `Approach`, `Affected Areas`,
  `Interfaces And Data`, `Risks`, `External Interfaces`,
  `Alternatives Considered`, `Migration Or Rollout Notes`
- в `03-how-we-verify.md` есть `Acceptance Criteria`, `Test Plan`,
  `Verification Checklist`, `Operational Validation`, `Failure Scenarios`,
  `Observability`
- артефакты пригодны как для ручного чтения, так и как вход для будущего
  implementation flow

## Operational Validation

Операционно задача считается успешной, если:

- оператор понимает, какой именно run path был использован
- ясно, какой waiting-статус является финалом текущего анализа
- если run блокируется, причина блокировки локализована в конкретном месте:
  GitHub status, launcher, worktree, `gh`, `zellij`, `codex` или flow prompt
- follow-up работа отделена от текущего анализа и не смешана с артефактами
  как будто она уже выполнена

## Failure Scenarios

- issue не прикреплена к project item или не имеет ожидаемого статуса
- `run` переводит issue в `Analysis In Progress`, но launcher не может довести
  запуск до агента
- `launch-agent.sh` создает worktree, но агент не стартует из-за отсутствия
  `codex` или некорректного execution context
- анализ запускается, но не хватает критичного контекста, и тогда финалом
  должен стать `Waiting for Clarification`, а не неявный обрыв
- в ходе прогона обнаруживается системная проблема, требующая отдельной issue;
  в этом случае она должна быть зафиксирована как blocking gap, а не
  раствориться в общем тексте

## Observability

Для разбора этого dogfooding run должно быть видно:

- какой issue URL был передан в launcher
- какой `session_uuid` связан с issue
- какой analysis branch/worktree был использован
- какой GitHub status был у issue до запуска и после него
- были ли созданы versioned analysis-артефакты
- в какой точке возник blocker, если run не дошел до ожидаемого waiting-исхода
