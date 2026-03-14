# ADR-0021: CLI-контракт `poll`, `run`, `loop`

Статус: accepted
Дата: 2026-03-14

## Контекст

В проекте сформировался конфликт между несколькими слоями документации и
реализации:

- SSOT уже фиксирует `poll` как one-shot selection cycle
- часть stable-документов по-прежнему описывает только `poll` и `run`
- в task-specific документах еще встречается старый термин `daemon`
- в коде есть loop-поведение, но оно не оформлено как единый публичный
  CLI-контракт

Из-за этого нет одного канонического ответа на вопросы:

- какая команда отвечает только за selection cycle
- где находится единая issue-level orchestration logic
- какая команда отвечает за повторный foreground запуск без внешнего scheduler
- как соотносятся `poll`, `run`, `loop` и legacy-термин `daemon`

## Решение

Для MVP и ближайшего развития CLI фиксируются три публичные команды:

- `poll`
- `run`
- `loop`

### `poll`

`poll` является one-shot командой одного цикла просмотра project snapshot.

Правила:

- команда выбирает только подходящие issues из `Backlog`
- команда не принимает issue как аргумент
- команда не реализует отдельную issue-level orchestration logic
- если issue найдена, команда передает ее в общий issue-level `run`-path
- если issue не найдена, команда завершает цикл без ошибки

### `run`

`run` является каноническим issue-level entrypoint.

Правила:

- команда принимает issue number или URL issue
- команда используется и для явного ручного запуска, и как внутренний path
  после выбора issue командой `poll`
- команда отвечает за проверку допустимости входа, перевод статуса, работу с
  `session_uuid`, launcher orchestration и re-entry behavior
- именно здесь должно приниматься единое решение о создании нового launcher path
  или восстановлении существующего контекста

### `loop`

`loop` является бесконечным foreground loop поверх `poll`.

Правила:

- каждая итерация `loop` переиспользует `poll`
- пауза между итерациями берется из `runtime.poll_interval_seconds`
- пустой цикл и ошибка одного цикла не завершают весь процесс
- bootstrap/config/runtime ошибки до входа в loop считаются фатальными

### Терминология

- канонический термин для one-shot команды: `poll`
- канонический термин для бесконечного foreground режима: `loop`
- `daemon` не используется как официальный CLI-контракт
- legacy-упоминания `daemon` должны быть либо удалены из контракта, либо
  обозначены как исторические task-артефакты

## Последствия

Плюсы:

- появляется четкое разделение ответственности между selection, issue-level
  lifecycle и foreground loop
- `poll`, `run` и `loop` могут переиспользовать один и тот же issue-level path
- упрощается развитие re-entry, session recovery и foreground automation

Минусы:

- нужно синхронно выровнять документацию, код и тесты
- task-specific материалы со старым термином `daemon` требуют явной миграции

## Связанные документы

- [../issue-analysis-flow.md](../issue-analysis-flow.md)
- [../features/0001-ai-teamlead-daemon/README.md](../features/0001-ai-teamlead-daemon/README.md)
- [./0002-standalone-foreground-daemon.md](./0002-standalone-foreground-daemon.md)
- [./0005-cli-contract-for-poll-and-run.md](./0005-cli-contract-for-poll-and-run.md)
