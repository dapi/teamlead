# Feature 0005: План имплементации

Статус: draft
Последнее обновление: 2026-03-14

## Назначение

Этот документ задает порядок реализации платформы integration-тестирования
agent flow для feature `0005`.

В отличие от `02-how-we-build.md`, этот документ описывает не целевую
архитектуру, а практический порядок шагов, который нужен, чтобы прийти к
рабочему MVP с внятными verification-сценариями и диагностикой.

## Scope

В текущий план входят:

- CLI entrypoint `ai-teamlead test agent-flow`
- Docker-based headless sandbox для `zellij`-related проверок
- read-only mount текущего проекта и allowlisted host agent files
- writable sandbox workspace внутри контейнера
- `gh` stub, invocation log и artifact bundle
- хотя бы один `stub`-сценарий для CI
- хотя бы один `live codex`-сценарий
- support path для `claude` как дополнительного live-profile

## Вне scope

В текущий план не входят:

- автоматический запуск live-сценариев на каждый commit в CI
- browser E2E или UI automation
- оценка semantic quality generated текста модели
- альтернативные sandbox backend кроме Docker

## Связанные документы

- Issue: GitHub issue `#38`
- Feature / issue spec:
  - [README.md](./README.md)
  - [01-what-we-build.md](./01-what-we-build.md)
  - [02-how-we-build.md](./02-how-we-build.md)
  - [03-how-we-verify.md](./03-how-we-verify.md)
- SSOT:
  - [../../issue-analysis-flow.md](../../issue-analysis-flow.md)
- ADR:
  - [../../adr/0011-use-zellij-main-release-in-ci.md](../../adr/0011-use-zellij-main-release-in-ci.md)
- Verification:
  - [03-how-we-verify.md](./03-how-we-verify.md)
- Code quality:
  - [../../code-quality.md](../../code-quality.md)
- Зависимые планы или фичи:
  - [../0001-ai-teamlead-cli/README.md](../0001-ai-teamlead-cli/README.md)
  - [../0002-repo-init/README.md](../0002-repo-init/README.md)
  - [../0003-agent-launch-orchestration/README.md](../0003-agent-launch-orchestration/README.md)
  - [../0003-agent-launch-orchestration/04-implementation-plan.md](../0003-agent-launch-orchestration/04-implementation-plan.md)

## Зависимости и предпосылки

- базовый CLI `ai-teamlead` уже умеет запускать `run` и `poll`
- launcher contract из feature `0003` считается источником истины для запуска
  агента
- `zellij`-related проверки остаются только внутри headless Docker sandbox
- host repo используется как read-only input layer; writable execution идет
  только во внутреннем sandbox workspace
- весь GitHub layer во время теста реализуется через `gh` stub

## Порядок работ

### Этап 1. Test CLI и scenario manifest

Цель:

- реализовать entrypoint `ai-teamlead test agent-flow`
- добавить загрузку и валидацию scenario manifest
- зафиксировать mapping между CLI flags и scenario contract

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- CLI принимает `--scenario`, `--agent`, `--mode`
- `--agent` и `--mode` валидируются против manifest
- ошибки конфигурации и валидации диагностируются до sandbox startup

Проверка:

- unit/integration tests на разбор manifest и CLI validation
- ручная проверка негативных кейсов для `preflight failed`

### Этап 2. Docker sandbox и workspace materialization

Цель:

- реализовать Docker-based headless runner
- смонтировать текущий проект read-only
- materialize-ить writable sandbox workspace внутри контейнера

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../../adr/0011-use-zellij-main-release-in-ci.md](../../adr/0011-use-zellij-main-release-in-ci.md)

Результат этапа:

- sandbox не использует host `zellij`
- host repo не получает runtime-побочных эффектов
- sandbox workspace содержит актуальный working tree и достаточный git context

Проверка:

- integration test на startup sandbox
- сценарий 7 из [03-how-we-verify.md](./03-how-we-verify.md)

### Этап 3. Agent bridge и preflight

Цель:

- реализовать allowlist bridge для `codex` и `claude`
- пробрасывать только нужные env vars и host config/auth files
- диагностировать отсутствие нужного auth/config path как `preflight failed`

Основание:

- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- `codex` работает как default live profile
- `claude` работает как дополнительный live-profile
- подписочный account/session auth и API credentials поддерживаются через один
  bridge contract

Проверка:

- integration tests на preflight
- сценарии 3, 4 и 5 из [03-how-we-verify.md](./03-how-we-verify.md)

### Этап 4. `gh` stub и artifact bundle

Цель:

- реализовать sandbox-local `gh` stub
- добавить versioned fixtures и invocation log
- собрать artifact bundle вне зависимости от verdict

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- все вызовы GitHub идут только через `gh` stub
- invocation log попадает в artifact bundle
- прямой real GitHub access приводит к явной ошибке среды

Проверка:

- integration tests на `gh` stub
- сценарий 6 и проверка invocation log из [03-how-we-verify.md](./03-how-we-verify.md)

### Этап 5. `stub`-agent и deterministic CI scenario

Цель:

- реализовать controlled `stub`-agent поверх того же launcher path
- обеспечить deterministic happy-path и clarification-path для CI

Основание:

- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- `stub` стартует через тот же orchestration path, что live agents
- CI получает стабильный сценарий без real LLM dependency
- проверки статуса и analysis artifacts воспроизводимы

Проверка:

- сценарии 1 и 2 из [03-how-we-verify.md](./03-how-we-verify.md)

### Этап 6. Live agents и smoke verification

Цель:

- довести до рабочего состояния `live codex`
- подтвердить support path для `live claude`
- пройти end-to-end smoke verification и сохранить диагностику

Основание:

- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

Результат этапа:

- локальный live-сценарий с `codex` проходит end-to-end
- `claude` как дополнительный profile запускается через тот же sandbox path
- `keep-sandbox`, artifact export и verdict lifecycle работают согласованно

Проверка:

- сценарии 3, 4, 8 и 9 из [03-how-we-verify.md](./03-how-we-verify.md)
- ручной smoke run на машине разработчика

## Критерий завершения

- существует рабочий entrypoint `ai-teamlead test agent-flow`
- Docker sandbox поднимается headless и не взаимодействует с host `zellij`
- host repo используется только как read-only input mount
- writable execution path и runtime side effects ограничены sandbox workspace
- `gh` stub и artifact bundle работают в `stub` и `live` режимах одинаково
- есть хотя бы один стабильный `stub`-сценарий для CI
- есть хотя бы один локальный `live codex`-сценарий
- `claude` поддержан как дополнительный live-profile

## Открытые вопросы и риски

- нужно ли ограничивать budget, timeout и максимальное число LLM вызовов на
  сценарий уже в первой версии
- какие именно host paths кроме `~/.codex` и `~/.claude` понадобятся для
  устойчивого live-run на разных машинах
- насколько объемным окажется минимальный Docker image с нужными CLI и pinned
  `zellij`

## Журнал изменений

### 2026-03-14

- создан начальный план имплементации для Feature 0005
