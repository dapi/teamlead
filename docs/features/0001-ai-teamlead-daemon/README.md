# Feature 0001: ai-teamlead CLI

Статус: draft
Владелец: владелец репозитория
Последнее обновление: 2026-03-14

## Контекст

Эта feature описывает первую версию `ai-teamlead` как CLI-утилиты, которая
работает в контексте одного репозитория и разделяет CLI-ответственности между
`poll`, `run` и `loop`.

Документ оформлен как каталог, потому что feature затрагивает продуктовую
модель, execution model, конфигурацию, интеграции и критерии готовности.

## Что строим

- [01-what-we-build.md](./01-what-we-build.md)

## Как строим

- [02-how-we-build.md](./02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## План реализации

- [04-implementation-plan.md](./04-implementation-plan.md)

## Runtime-артефакты

- [05-runtime-artifacts.md](./05-runtime-artifacts.md)

## Связанные документы

- [README.md](../../../README.md)
- [docs/issue-analysis-flow.md](../../issue-analysis-flow.md)
- [docs/adr/0001-repo-local-ai-config.md](../../adr/0001-repo-local-ai-config.md)
- [docs/adr/0002-standalone-foreground-daemon.md](../../adr/0002-standalone-foreground-daemon.md)
- [docs/adr/0003-github-project-status-as-source-of-truth.md](../../adr/0003-github-project-status-as-source-of-truth.md)
- [docs/adr/0004-runtime-artifacts-in-git-dir.md](../../adr/0004-runtime-artifacts-in-git-dir.md)
- [docs/adr/0006-use-gh-cli-as-github-integration-layer.md](../../adr/0006-use-gh-cli-as-github-integration-layer.md)
- [docs/adr/0007-no-separate-health-interface-in-mvp.md](../../adr/0007-no-separate-health-interface-in-mvp.md)
- [docs/adr/0008-bind-issue-to-agent-session-uuid.md](../../adr/0008-bind-issue-to-agent-session-uuid.md)
- [docs/adr/0009-deterministic-backlog-ordering.md](../../adr/0009-deterministic-backlog-ordering.md)
- [docs/adr/0010-use-rust-for-mvp-implementation.md](../../adr/0010-use-rust-for-mvp-implementation.md)
- [docs/adr/0011-use-zellij-main-release-in-ci.md](../../adr/0011-use-zellij-main-release-in-ci.md)
- [docs/adr/0013-agent-session-history-as-dialog-source.md](../../adr/0013-agent-session-history-as-dialog-source.md)
- [docs/adr/0014-zellij-launch-context-naming.md](../../adr/0014-zellij-launch-context-naming.md)
- [docs/adr/0021-cli-contract-poll-run-loop.md](../../adr/0021-cli-contract-poll-run-loop.md)

## Открытые вопросы

- нужны ли дополнительные CLI-команды после MVP

## Журнал изменений

### 2026-03-13

- создан каталог feature 0001 для `ai-teamlead`
- feature сразу разнесена по трем главным осям документации

### 2026-03-14

- feature выровнена с SSOT по контракту `poll` / `run` / `loop`
