# Feature 0001: ai-teamlead CLI

Статус: draft
Владелец: владелец репозитория
Последнее обновление: 2026-03-13

## Контекст

Эта feature описывает первую версию `ai-teamlead` как CLI-утилиты, которая
работает в контексте одного репозитория, по команде `poll` ищет подходящие
GitHub issues и запускает `issue-analysis-flow`.

Документ оформлен как каталог, потому что feature затрагивает продуктовую
модель, execution model, конфигурацию, интеграции и критерии готовности.

## Что строим

- [01-what-we-build.md](/home/danil/code/teamlead/docs/features/0001-ai-teamlead-daemon/01-what-we-build.md)

## Как строим

- [02-how-we-build.md](/home/danil/code/teamlead/docs/features/0001-ai-teamlead-daemon/02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](/home/danil/code/teamlead/docs/features/0001-ai-teamlead-daemon/03-how-we-verify.md)

## План реализации

- [04-implementation-plan.md](/home/danil/code/teamlead/docs/features/0001-ai-teamlead-daemon/04-implementation-plan.md)

## Runtime-артефакты

- [05-runtime-artifacts.md](/home/danil/code/teamlead/docs/features/0001-ai-teamlead-daemon/05-runtime-artifacts.md)

## Связанные документы

- [README.md](/home/danil/code/teamlead/README.md)
- [docs/issue-analysis-flow.md](/home/danil/code/teamlead/docs/issue-analysis-flow.md)
- [docs/adr/0001-repo-local-ai-config.md](/home/danil/code/teamlead/docs/adr/0001-repo-local-ai-config.md)
- [docs/adr/0002-standalone-foreground-daemon.md](/home/danil/code/teamlead/docs/adr/0002-standalone-foreground-daemon.md)
- [docs/adr/0003-github-project-status-as-source-of-truth.md](/home/danil/code/teamlead/docs/adr/0003-github-project-status-as-source-of-truth.md)
- [docs/adr/0004-runtime-artifacts-in-git-dir.md](/home/danil/code/teamlead/docs/adr/0004-runtime-artifacts-in-git-dir.md)
- [docs/adr/0005-cli-contract-for-poll-and-run.md](/home/danil/code/teamlead/docs/adr/0005-cli-contract-for-poll-and-run.md)
- [docs/adr/0006-use-gh-cli-as-github-integration-layer.md](/home/danil/code/teamlead/docs/adr/0006-use-gh-cli-as-github-integration-layer.md)
- [docs/adr/0007-no-separate-health-interface-in-mvp.md](/home/danil/code/teamlead/docs/adr/0007-no-separate-health-interface-in-mvp.md)
- [docs/adr/0008-bind-issue-to-agent-session-uuid.md](/home/danil/code/teamlead/docs/adr/0008-bind-issue-to-agent-session-uuid.md)
- [docs/adr/0009-deterministic-backlog-ordering.md](/home/danil/code/teamlead/docs/adr/0009-deterministic-backlog-ordering.md)
- [docs/adr/0010-use-rust-for-mvp-implementation.md](/home/danil/code/teamlead/docs/adr/0010-use-rust-for-mvp-implementation.md)
- [docs/adr/0011-use-zellij-main-release-in-ci.md](/home/danil/code/teamlead/docs/adr/0011-use-zellij-main-release-in-ci.md)
- [docs/adr/0013-agent-session-history-as-dialog-source.md](/home/danil/code/teamlead/docs/adr/0013-agent-session-history-as-dialog-source.md)
- [docs/adr/0014-zellij-launch-context-naming.md](/home/danil/code/teamlead/docs/adr/0014-zellij-launch-context-naming.md)

## Открытые вопросы

- нужны ли дополнительные CLI-команды после MVP

## Журнал изменений

### 2026-03-13

- создан каталог feature 0001 для `ai-teamlead`
- feature сразу разнесена по трем главным осям документации
