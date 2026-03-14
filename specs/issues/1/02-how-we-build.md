# Как строим

## Подход

Изменение должно сводиться к выделению одного переиспользуемого polling cycle и
оборачиванию его в loop для команды `daemon`.

Предпочтительный путь:

- вынести текущую логику `run_poll` в общий helper, который выполняет ровно один
  polling cycle и возвращает структурированный outcome
- оставить CLI-команду `poll` thin wrapper над этим helper, чтобы ее контракт не
  изменился
- переписать `run_daemon`, чтобы после успешного bootstrap она входила в
  бесконечный цикл:
  - печать `cycle started`
  - запуск одного polling cycle
  - печать `cycle finished` с outcome или ошибкой
  - sleep на `runtime.poll_interval_seconds`
- не пробрасывать ошибки одного cycle наружу из daemon loop; вместо этого
  логировать их и продолжать следующую итерацию

Это сохраняет один источник бизнес-логики для `poll` и `daemon` и не создает
второй расходящийся execution path.

## Затронутые области

- `src/app.rs`
  daemon entrypoint, выделение общего polling cycle, обработка loop и
  диагностики
- при необходимости отдельный модуль application/service уровня
  для `poll` outcome и loop-control, если логика в `app.rs` начнет разрастаться
- integration tests для `daemon`
- возможно unit tests для loop-control, если будет добавлен явный outcome/helper

## Интерфейсы и данные

Входы:

- `RepoContext`
- `Config`
- `RuntimeLayout`
- `runtime.poll_interval_seconds`
- текущие GitHub snapshot/status update interfaces
- текущий `ZellijLauncher`

Новые внешние интерфейсы не требуются. Формат `settings.yml`, runtime layout и
CLI shape (`daemon`, `poll`, `run`) остаются прежними.

Полезно ввести внутренний outcome одного цикла, например:

- `NoEligibleIssue`
- `Claimed { issue_number, session_uuid }`
- `CycleFailed { message }`

Даже если outcome останется локальным типом внутри `app.rs`, он упростит единый
лог для `poll` и `daemon` и позволит тестировать loop без парсинга случайных
строк.

## Конфигурация и runtime-допущения

- `runtime.poll_interval_seconds >= 1` уже валидируется конфигом и может
  использоваться без дополнительной миграции схемы
- sleep выполняется между завершением одного cycle и стартом следующего
- daemon должен продолжать работать, пока процесс не остановлен внешним
  сигналом или не произошла фатальная startup-ошибка до входа в loop
- поведение `poll` как one-shot команды должно сохраниться без sleep и без
  бесконечного цикла

## Architecture Notes

Это не новый архитектурный слой, а доведение существующей execution model до
документированного состояния. Главный архитектурный выбор здесь не в добавлении
новых возможностей, а в том, чтобы:

- не дублировать polling logic между `poll` и `daemon`
- четко разделить bootstrap-ошибки и cycle-ошибки
- сделать диагностику loop явной, а не выводить только стартовые параметры

## Risks

- если sleep будет пропускаться после ошибки, daemon превратится в busy loop
- если `daemon` получит отдельный polling path, `poll` и `daemon` начнут
  расходиться по статусным переходам и launcher contract
- если ошибки launcher не будут корректно локализованы в одном cycle, процесс
  будет завершаться вопреки критерию готовности issue
- бесконечный loop усложнит integration tests, если не продумать способ
  ограничить время выполнения тестового процесса

## External Interfaces

Внешние интеграции остаются теми же:

- `gh` CLI для snapshot и status updates
- `zellij` launcher для старта issue-analysis session
- stdout/stderr как операторская диагностика foreground daemon

Изменение касается только частоты и устойчивости их вызова во времени.

## ADR Impact

Новый ADR не требуется.

Issue реализует уже принятые решения, зафиксированные как минимум в:

- [../../../docs/adr/0002-standalone-foreground-daemon.md](../../../docs/adr/0002-standalone-foreground-daemon.md)
- [../../../docs/adr/0005-cli-contract-for-poll-and-run.md](../../../docs/adr/0005-cli-contract-for-poll-and-run.md)
- [../../../docs/adr/0007-no-separate-health-interface-in-mvp.md](../../../docs/adr/0007-no-separate-health-interface-in-mvp.md)

## Alternatives Considered

### Оставить отдельную реализацию `poll` и добавить новый daemon-only cycle

Отклонено, потому что это создаст два почти одинаковых path с высоким риском
дрейфа контрактов и тестов.

### Завершать daemon при любой ошибке цикла

Отклонено, потому что это противоречит явному требованию issue: ошибка одного
цикла не должна делать процесс непригодным для дальнейшей работы.

### Делегировать периодичность внешнему scheduler

Отклонено для этой issue, потому что MVP уже фиксирует standalone foreground
daemon с собственным polling loop.

## Migration Or Rollout Notes

- миграция конфигурации не нужна
- обратная совместимость `poll` и `run` должна сохраниться
- rollout сводится к обновлению бинаря и прогону smoke-сценариев для реального
  foreground daemon

