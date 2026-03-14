# Что строим

## Проблема

`ai-teamlead daemon` по документации должен быть самостоятельным foreground
процессом с собственным polling loop, но в текущем коде команда только выводит
`daemon ready` и завершается. Из-за этого daemon нельзя использовать как
реальный long-running entrypoint для регулярного анализа backlog issues.

## Для кого

- для владельца репозитория или оператора, который хочет запустить один
  repo-local daemon и не дергать `poll` вручную
- для будущего implementation flow, который рассчитывает на стабильный
  foreground runtime вокруг существующего polling cycle

## Ожидаемый результат

После изменения `ai-teamlead daemon`:

- выполняет последовательные polling cycles без ручного перезапуска
- использует интервал из `runtime.poll_interval_seconds`
- переживает пустые циклы и ошибки отдельного цикла
- оставляет понятную диагностику начала, исхода и завершения каждого цикла

## Scope

В scope этой issue входит:

- обернуть существующий polling path в бесконечный foreground loop
- переиспользовать текущую логику одного polling cycle вместо отдельного
  daemon-only path
- добавить sleep между циклами с учетом `runtime.poll_interval_seconds`
- сделать так, чтобы ошибка одного цикла не завершала весь daemon-процесс
- обновить unit, integration и smoke проверки под новый runtime-контракт

## Non-Goals

Вне scope этой issue:

- новый scheduler или переход на `systemd`/cron
- multi-worker или `max_parallel > 1`
- новый health endpoint или отдельный supervisor model
- изменение GitHub status model, runtime layout или launcher contract
- переработка `poll`/`run` semantics сверх необходимого для общего loop path

## Ограничения и допущения

- источник истины по состоянию issue остается в GitHub Project
- текущий repo-local конфиг не меняется; используется существующее поле
  `runtime.poll_interval_seconds`
- daemon должен оставаться foreground-процессом в контексте одного репозитория
- startup-ошибки уровня bootstrap остаются фатальными, а ошибки отдельного
  polling cycle должны изолироваться внутри loop
- диагностика должна быть достаточной для ручного чтения stdout/stderr без
  отдельного observability-сервиса

## User Story

Как оператор репозитория, я хочу запустить `ai-teamlead daemon` один раз и
получать повторяющиеся polling cycles по документированному интервалу, чтобы
новые backlog issues подбирались автоматически без ручного вызова `poll`.

## Use Cases

### Use Case 1. Пустой backlog

- оператор запускает `ai-teamlead daemon`
- daemon выполняет polling cycle
- подходящих issues нет
- процесс пишет диагностику пустого цикла, ждет интервал и запускает следующий
  cycle

### Use Case 2. Успешный claim

- daemon находит backlog issue
- использует тот же claim и launch path, что и `poll`
- после завершения цикла не завершается, а продолжает работать до следующего
  интервала

### Use Case 3. Ошибка одного цикла

- во время polling cycle происходит ошибка чтения snapshot, смены статуса или
  запуска launcher
- daemon пишет ошибку цикла
- процесс продолжает работу и пытается выполнить следующий cycle после sleep

## Зависимости

- существующая логика `run_poll` и связанные GitHub/zellij adapters
- конфиг `./.ai-teamlead/settings.yml` с валидным
  `runtime.poll_interval_seconds`
- текущие контракты feature 0001 и `issue-analysis-flow`, которые уже требуют
  foreground daemon loop

