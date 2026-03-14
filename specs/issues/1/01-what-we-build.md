# Что строим

## Проблема

Исторически в issue использовался термин `ai-teamlead daemon`, но канонический
CLI-контракт теперь называет этот режим `loop`.

Проблема остается той же: foreground-команда для непрерывной обработки backlog
должна выполнять повторяющиеся polling cycles, а не завершаться после bootstrap.

## Для кого

- для владельца репозитория или оператора, который хочет запустить один
  repo-local `loop` и не дергать `poll` вручную
- для будущего implementation flow, который рассчитывает на стабильный
  foreground runtime вокруг существующего polling cycle

## Ожидаемый результат

После изменения `ai-teamlead loop`:

- выполняет последовательные polling cycles без ручного перезапуска
- использует интервал из `runtime.poll_interval_seconds`
- переживает пустые циклы и ошибки отдельного цикла
- оставляет понятную диагностику начала, исхода и завершения каждого цикла

## Scope

В scope этой issue входит:

- обернуть существующий polling path в бесконечный foreground loop
- переиспользовать текущую логику одного polling cycle вместо отдельного
  loop-only path
- добавить sleep между циклами с учетом `runtime.poll_interval_seconds`
- сделать так, чтобы ошибка одного цикла не завершала весь foreground-процесс
- обновить unit, integration и smoke проверки под новый runtime-контракт

## Non-Goals

Вне scope этой issue:

- новый scheduler или переход на `systemd`/cron
- multi-worker или `max_parallel > 1`
- новый health endpoint или отдельный supervisor model
- изменение GitHub status model, runtime layout или launcher contract
- переработка `poll`/`run` semantics сверх необходимого для общего `loop` path

## Ограничения и допущения

- источник истины по состоянию issue остается в GitHub Project
- текущий repo-local конфиг не меняется; используется существующее поле
  `runtime.poll_interval_seconds`
- `loop` должен оставаться foreground-процессом в контексте одного репозитория
- startup-ошибки уровня bootstrap остаются фатальными, а ошибки отдельного
  polling cycle должны изолироваться внутри loop
- диагностика должна быть достаточной для ручного чтения stdout/stderr без
  отдельного observability-сервиса

## User Story

Как оператор репозитория, я хочу запустить `ai-teamlead loop` один раз и
получать повторяющиеся polling cycles по документированному интервалу, чтобы
новые backlog issues подбирались автоматически без ручного вызова `poll`.

## Use Cases

### Use Case 1. Пустой backlog

- оператор запускает `ai-teamlead loop`
- `loop` выполняет polling cycle
- подходящих issues нет
- процесс пишет диагностику пустого цикла, ждет интервал и запускает следующий
  cycle

### Use Case 2. Успешный claim

- `loop` находит backlog issue
- использует тот же claim и launch path, что и `poll`
- после завершения цикла не завершается, а продолжает работать до следующего
  интервала

### Use Case 3. Ошибка одного цикла

- во время polling cycle происходит ошибка чтения snapshot, смены статуса или
  запуска launcher
- `loop` пишет ошибку цикла
- процесс продолжает работу и пытается выполнить следующий cycle после sleep

## Зависимости

- существующая логика `run_poll` и связанные GitHub/zellij adapters
- конфиг `./.ai-teamlead/settings.yml` с валидным
  `runtime.poll_interval_seconds`
- текущие контракты feature 0001 и `issue-analysis-flow`, которые уже требуют
  foreground `loop` поверх `poll`
