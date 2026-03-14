# Feature 0005: Как проверяем

## Критерии корректности

Решение считается корректным, если:

- один локальный entrypoint поднимает sandbox и выполняет сценарий end-to-end
- sandbox не использует host `zellij` session, tab или pane пользователя
- `ai-teamlead` внутри sandbox запускается по обычному CLI path
- `launch-agent.sh` и project-local flow prompt используются без test-only
  bypass
- `stub` и `live` режимы разделяют один и тот же orchestration path
- default live path использует `codex`
- все GitHub-взаимодействия идут через `gh` stub, а не через реальный GitHub
- agent credentials и config files попадают в sandbox только через явный
  allowlist
- live-режим поддерживает не только API credentials, но и host-level
  account/session auth для `claude` / `codex` по подписке, если его
  поддерживает сам агент
- bridge использует те же host-level credentials и параметры подключения, с
  которыми запущен сам test suite
- forwarded secrets не сохраняются в artifact bundle и логах
- в artifact bundle сохраняется invocation log `gh` stub
- по завершении прогона пользователь получает итоговый verdict и путь к
  артефактам

## Критерии готовности

Feature считается готовой, если:

- есть хотя бы один стабильный `stub`-сценарий для CI
- есть хотя бы один локальный `live`-сценарий для реального агента `codex`
- `codex` зафиксирован как default live path
- `claude` поддержан как дополнительный live-profile первой версии
- есть как минимум один сценарий, который проверяет invocation log `gh` stub
- результаты можно воспроизвести повторным запуском на той же машине
- ошибки preflight, sandbox startup и assertion failure различимы по статусу и
  диагностике

## Инварианты

- host-окружение пользователя не является execution surface для `zellij`-tests
- versioned scenario manifest является источником истины для test intent
- sandbox побочные эффекты не должны писать в рабочее дерево пользователя;
  host repo используется только как read-only input mount
- live-режим использует реальные локальные agent settings, но только через
  explicit bridge
- explicit bridge использует именно host-level настройки, credentials и
  account/session auth запущенного test suite
- GitHub API не должен быть доступен из тестового прогона напрямую
- тестовая платформа не подменяет `launch-agent.sh` отдельным искусственным
  runner-скриптом
- любой verdict должен сопровождаться artifact bundle

## Сценарии проверки

### Сценарий 1. `stub` happy path

- запущен `ai-teamlead test agent-flow --scenario run-happy-path --agent stub --mode stub`
- sandbox создается успешно
- `ai-teamlead run` проходит обычный launcher path
- `gh` stub получает ожидаемые вызовы и пишет invocation log
- `stub` завершает анализ как `plan-ready`
- assertions подтверждают, что созданы analysis artifacts и итоговый статус
  корректен

### Сценарий 2. `stub` clarification path

- сценарий моделирует `needs-clarification`
- runner проверяет смену статуса и наличие диагностического сообщения

### Сценарий 3. `live codex`

- пользователь запускает локальный live-сценарий с `codex`
- sandbox получает только разрешенные env vars и mounts для `codex`
- агент стартует внутри sandbox и использует project-local flow
- по завершении доступны логи и runtime artifacts

### Сценарий 4. `live claude` как дополнительный профиль

- пользователь запускает локальный live-сценарий с `claude`
- sandbox получает allowlisted настройки для `claude`
- по умолчанию для `claude` используется профиль Claude Code с моделью класса
  Sonnet
- сценарий остается валидным, если доступ к модели обеспечивается через
  подписочный account login Claude Code, а не через отдельный API key
- orchestration path совпадает с `codex`-сценарием до точки выбора agent
  profile

### Сценарий 5. Нет credentials

- выбран `live`-режим
- отсутствуют и API credentials, и требуемый allowlisted account/session auth
- runner завершает прогон в статусе `preflight failed`
- пользователь видит, какого именно bridge entry не хватило

### Сценарий 6. Попытка real GitHub доступа

- sandbox или процесс внутри него пытается обратиться к реальному GitHub
- runner фиксирует нарушение sandbox policy
- прогон завершается с явной ошибкой, а не с неявным fallback на реальный `gh`

### Сценарий 7. Текущий проект с локальными изменениями

- в текущем working tree есть локальные изменения flow или launcher
- sandbox видит эти изменения через read-only volume mount текущего проекта
- runner создает отдельный writable workspace внутри контейнера
- host repo остается неизменным после завершения теста

### Сценарий 8. Падение assertions

- orchestration path завершается, но expected assertions не выполняются
- verdict = `failed`
- artifact bundle сохраняется для ручного разбора

### Сценарий 9. Сохранение sandbox по запросу

- пользователь запускает сценарий с `--keep-sandbox`
- после завершения runner не удаляет container/filesystem
- в artifact metadata явно указан путь к сохраненному sandbox

## Диагностика и наблюдаемость

Минимально необходимо видеть:

- `run_id`, имя сценария, выбранные `agent` и `mode`
- effective image/tag sandbox-а
- был ли использован `stub` или `live`
- список forwarded env var names без значений
- список mounted config paths
- путь к `gh` invocation log
- путь к workspace snapshot и artifact bundle
- ключевые переходы состояний:
  `snapshot_prepared`, `sandbox_ready`, `runtime_started`, `agent_running`,
  `asserting`, итоговый verdict
- stdout/stderr entrypoint-а и launcher-а
- причину verdict `preflight failed`, `failed` или `errored`

## Связанные документы

- [README.md](./README.md)
- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [../../adr/0011-use-zellij-main-release-in-ci.md](../../adr/0011-use-zellij-main-release-in-ci.md)
