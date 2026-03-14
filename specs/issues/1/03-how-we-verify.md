# Как проверяем

## Acceptance Criteria

- `ai-teamlead loop` после старта не завершается сам по себе после первого
  polling cycle
- каждый cycle `loop` использует тот же polling path, что и ручная команда
  `poll`
- при пустом backlog `loop` пишет диагностируемый результат цикла и запускает
  следующий cycle после `runtime.poll_interval_seconds`
- при ошибке одного cycle `loop` пишет ошибку и остается пригодным для
  следующего cycle
- при успешном cycle `loop` сохраняет существующее поведение claim + launcher и
  после этого продолжает loop
- в диагностике видны как минимум начало цикла, его исход и ожидание до
  следующего запуска

## Ready Criteria

- код `loop` соответствует существующему SSOT и feature 0001
- обновлены или добавлены unit/integration/smoke проверки под loop behavior
- `poll` остается one-shot командой без регрессии текущих integration tests
- не требуется изменение `./.ai-teamlead/settings.yml` или runtime layout

## Invariants

- один `loop` обслуживает один репозиторий
- source of truth по статусу issue остается в GitHub Project
- `poll` и `loop` используют единый polling contract
- ошибка одного polling cycle не должна завершать `loop`
- sleep между циклами берется из `runtime.poll_interval_seconds`

## Happy Path

### Happy Path 1. Последовательные пустые циклы

- `loop` стартует с валидным конфигом
- первый cycle не находит backlog issues
- процесс логирует пустой результат, ждет интервал и запускает второй cycle

### Happy Path 2. Успешный claim с продолжением loop

- первый cycle находит backlog issue и успешно запускает analysis session
- `loop` логирует успешный outcome
- после sleep запускается следующий cycle без перезапуска процесса

## Edge Cases

- backlog пуст в нескольких циклах подряд
- первый cycle завершается ошибкой, второй проходит успешно
- успешный cycle и пустой cycle чередуются без деградации процесса
- `poll_interval_seconds` минимален (`1`), и `loop` все равно не превращается в
  busy loop внутри одного cycle

## Failure Scenarios

- ошибка чтения project snapshot не завершает `loop` навсегда
- ошибка status update не завершает `loop` навсегда
- ошибка launcher после claim отражается в диагностике конкретного cycle и не
  ломает последующие итерации
- фатальная ошибка bootstrap до входа в loop по-прежнему завершает команду
  сразу, чтобы не скрывать некорректную конфигурацию или broken repo context

## Observability

Минимально нужно видеть в stdout/stderr:

- старт `loop` с repo и `project_id`
- номер или порядковый идентификатор текущего cycle
- время начала cycle
- outcome: пусто, claim issue, ошибка
- интервал ожидания до следующего cycle

Если для тестов или ручной диагностики потребуется более стабильный контракт,
допустимо нормализовать сообщения цикла в несколько фиксированных строк.

## Test Plan

Unit tests:

- тест для loop-control, который подтверждает: после `NoEligibleIssue` `loop`
  планирует следующий cycle
- тест для loop-control, который подтверждает: после `CycleFailed` `loop`
  планирует следующий cycle
- если выделен отдельный outcome/helper, тест на сохранение one-shot semantics
  у команды `poll`

Integration tests:

- тест, который поднимает `loop` в временном репозитории со stub `gh` и
  подтверждает минимум два последовательных cycle по логам или побочным
  артефактам
- тест, который имитирует пустой первый cycle и backlog issue на следующем, чтобы
  подтвердить recovery без ручного рестарта
- тест, который имитирует ошибку одного cycle и убеждается, что процесс не
  завершился до следующей попытки

Smoke tests:

- ручной запуск `ai-teamlead loop` в реальном репозитории и наблюдение как
  минимум двух cycle подряд
- проверка, что пустой cycle не завершает процесс
- проверка, что искусственно вызванная ошибка одного cycle не мешает следующему
  polling pass после интервала

## Verification Checklist

- `loop` действительно входит в loop, а не печатает только readiness banner
- `poll` по-прежнему выполняет ровно один cycle
- sleep между cycle соответствует `runtime.poll_interval_seconds`
- пустой cycle не ломает следующий
- ошибочный cycle не ломает следующий
- диагностика читаема и позволяет понять, что происходило в каждом cycle
