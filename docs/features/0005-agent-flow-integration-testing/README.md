# Feature 0005: платформа integration-тестирования agent flow

Статус: accepted
Владелец: владелец репозитория
Последнее обновление: 2026-03-14

## Контекст

Проект уже фиксирует requirement на integration tests для `poll`, `run`,
launcher-контракта и headless `zellij`, но пока не имеет отдельной платформы,
которая локально поднимает полностью изолированный sandbox и прогоняет agent
flow end-to-end.

Новая feature описывает именно такую платформу:

- с единым локальным entrypoint
- с disposable sandbox, не использующим host `zellij`
- с поддержкой реального `codex` по умолчанию и дополнительной поддержкой
  `claude`
- с использованием тех же host-level настроек, credentials и параметров
  подключения к LLM, с которыми запущен сам test suite, через явный allowlist
- с поддержкой host-level account/session auth для `claude` / `codex` по
  подписке, а не только через отдельные LLM API credentials
- с обязательным `gh` stub внутри sandbox вместо доступа к реальному GitHub

## Что строим

- [01-what-we-build.md](./01-what-we-build.md)

## Как строим

- [02-how-we-build.md](./02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## План реализации

- [04-implementation-plan.md](./04-implementation-plan.md)

## Зависимости

- [Feature 0001](../0001-ai-teamlead-daemon/README.md) — основной CLI и runtime
  orchestration
- [Feature 0002](../0002-repo-init/README.md) — repo-local assets и
  `settings.yml`
- [Feature 0003](../0003-agent-launch-orchestration/README.md) — launcher path,
  `zellij` и запуск агента

## Связанные документы

- [../../code-quality.md](../../code-quality.md)
- [../../issue-analysis-flow.md](../../issue-analysis-flow.md)
- [../../adr/0011-use-zellij-main-release-in-ci.md](../../adr/0011-use-zellij-main-release-in-ci.md)

## Открытые вопросы

- нужно ли отдельно ограничивать budget, timeout и максимальное число LLM
  вызовов на сценарий

## Журнал изменений

### 2026-03-14

- создан draft feature-документ для платформы integration-тестирования agent
  flow
- спецификация утверждена заказчиком; можно переходить к реализации по
  `04-implementation-plan.md`
