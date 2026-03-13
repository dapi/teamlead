# Issue 3: Как строим

Статус: draft
Последнее обновление: 2026-03-13

## Approach

Решение строится как реальный dogfooding run поверх уже существующего
application path, без отдельного "демо-режима" для этой issue.

Базовый подход:

- использовать текущую команду `ai-teamlead run <issue-url>` как единственный
  точный путь запуска
- позволить `run` перевести issue в `Analysis In Progress`, если это допустимо
  по текущему статусу
- передать управление в project-local `./.ai-teamlead/launch-agent.sh`
- подготовить analysis branch/worktree и каталог артефактов
- выполнить агентский анализ по staged prompts и сохранить результат в
  `specs/issues/3/`
- по итогам run зафиксировать один из двух исходов:
  завершение в допустимом waiting-статусе или обнаружение blocking gap

Это означает, что сама задача проверяет не новую продуктовую возможность, а
согласованность уже принятых контрактов между CLI, GitHub Project, runtime
артефактами, launcher script и агентским prompt flow.

## Affected Areas

- `src/app.rs`
  ручной путь `run`, проверка допустимости статуса и запуск zellij launcher
- `src/github.rs`
  чтение project snapshot и обновление project status
- `src/runtime.rs`
  issue/session binding и runtime manifest для повторных запусков
- `src/zellij.rs`
  запуск pane и передача управления launcher script
- `./.ai-teamlead/launch-agent.sh`
  bind pane, render launch context, подготовка worktree, вызов `codex`
- `./.ai-teamlead/settings.yml`
  project id, имена статусов и шаблоны analysis workspace
- `./.ai-teamlead/flows/issue-analysis-flow.md`
  entrypoint prompt и staged routing анализа
- `specs/issues/3/`
  versioned SDD-результат текущего dogfooding run

## Interfaces And Data

Ключевые данные и состояния:

- `issue_number = 3`
- `session_uuid = 60b8102f-9fb5-411f-bc42-8cb1d4b4b39a`
- analysis branch: `analysis/issue-3`
- analysis artifacts dir: `specs/issues/3`
- GitHub Project statuses:
  `Backlog`, `Analysis In Progress`, `Waiting for Clarification`,
  `Waiting for Plan Review`, `Ready for Implementation`,
  `Analysis Blocked`

Ключевые переходы для этого run:

- допустимый старт из `Backlog`
- перевод в `Analysis In Progress` при claim/relaunch
- дальнейший переход в `Waiting for Clarification` при нехватке критичной
  информации
- или переход в `Waiting for Plan Review`, если SDD-комплект собран достаточно
  хорошо
- или фиксация `Analysis Blocked`, если реальный run упирается в технический
  блокер вне scope текущего прогона

Выходные артефакты текущей issue:

- `README.md`
- `01-what-we-build.md`
- `02-how-we-build.md`
- `03-how-we-verify.md`

## Risks

- issue может не находиться в ожидаемом project status, и тогда `run` будет
  отклонен еще до старта анализа
- в реальном окружении могут проявиться зависимости, не видимые в integration
  tests: доступ к `gh`, `zellij`, default branch, worktree paths, `codex`
- операторский опыт может оказаться размытым, если ошибка возникает между
  сменой GitHub status и фактическим стартом агента
- dogfooding run может выявить системный gap, который нельзя закрыть в рамках
  одной analysis issue без отдельного product/architecture решения

## External Interfaces

- GitHub CLI (`gh`)
  используется для чтения issue/project snapshot и обновления статуса в GitHub
  Project
- Git
  используется для определения repo context, default branch и создания worktree
- Zellij
  используется как session/tab/pane runtime для анализа issue
- Codex
  используется как агент, которому launcher передает flow prompt и execution
  context

## Alternatives Considered

### Не выполнять реальный run и ограничиться review существующих тестов

Отклонено, потому что цель issue прямо состоит в первом живом dogfooding run, а
не в повторной теоретической проверке уже существующих контрактов.

### Добавить отдельный synthetic workflow специально для dogfooding

Отклонено, потому что это скрыло бы реальные orchestration проблемы текущего
`run` path и уменьшило бы ценность находок.

### Считать успехом только `Waiting for Plan Review`

Отклонено, потому что для первого живого прогона полезным результатом также
считается четкий список blocking gaps, если именно они мешают дойти до конца
без спекуляций.

## Migration Or Rollout Notes

Специальная миграция не нужна: задача использует уже существующий flow.

Rollout-эффект этой issue заключается в другом:

- после первого живого прогона команда уточняет, можно ли использовать текущий
  workflow без дополнительных guardrails
- выявленные blocking gaps должны оформляться отдельными issues, а не
  неявными заметками в чате
- если по итогам прогона понадобится изменить flow-контракт, сначала должен
  обновиться SSOT/ADR, и только потом код
