# Feature 0006: public repo security

Статус: draft
Владелец: владелец репозитория
Последнее обновление: 2026-03-14

## Контекст

`ai-teamlead` работает локально на машине пользователя, читает GitHub issue,
контент репозитория и runtime-артефакты, а затем запускает агентский workflow с
доступом к shell, файловой системе и внешним интеграциям.

Для публичных репозиториев это создает отдельный security-класс задач:
недоверенный контент может пытаться управлять поведением агента, заставлять его
выполнять нежелательные действия или выводить чувствительные данные.

Эта feature задает contract layer для безопасного использования
`ai-teamlead` в публичных репозиториях и определяет минимальный safe mode,
который должен применяться к hostile-by-default входам.

## Что строим

- [01-what-we-build.md](./01-what-we-build.md)

## Как строим

- [02-how-we-build.md](./02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## План реализации

- [04-implementation-plan.md](./04-implementation-plan.md)

## Связанные документы

- [README.md](../../../README.md)
- [docs/untrusted-input-security.md](../../untrusted-input-security.md)
- [docs/issue-analysis-flow.md](../../issue-analysis-flow.md)
- [docs/issue-implementation-flow.md](../../issue-implementation-flow.md)
- [docs/code-quality.md](../../code-quality.md)
- [docs/features/0001-ai-teamlead-cli/README.md](../0001-ai-teamlead-cli/README.md)
- [docs/features/0003-agent-launch-orchestration/README.md](../0003-agent-launch-orchestration/README.md)
- [docs/adr/0006-use-gh-cli-as-github-integration-layer.md](../../adr/0006-use-gh-cli-as-github-integration-layer.md)
- [docs/adr/0027-untrusted-github-content-as-hostile-input.md](../../adr/0027-untrusted-github-content-as-hostile-input.md)
- [docs/adr/0028-public-repo-safe-mode-and-permission-gates.md](../../adr/0028-public-repo-safe-mode-and-permission-gates.md)

## Открытые вопросы

- как именно runtime должен определять visibility репозитория и когда fallback в
  safe mode должен включаться по умолчанию;
- нужен ли отдельный флаг принудительного `public-safe` режима даже для private
  репозиториев;
- какие ограничения нужно enforce-ить в CLI, а какие можно оставить на уровне
  agent launcher и operator guidance.

## Журнал изменений

### 2026-03-14

- создана feature 0006 для безопасности использования `ai-teamlead` в публичных
  репозиториях
