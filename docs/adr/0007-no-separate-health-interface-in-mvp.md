# ADR-0007: Не вводить отдельный health/status интерфейс в MVP

Статус: accepted
Дата: 2026-03-13

## Контекст

Для первой версии `ai-teamlead` нужно было решить, требуется ли
отдельный health/status интерфейс для наблюдения за утилитой.

Рассматривались варианты:

- отдельная команда `status`
- отдельный health endpoint или status file
- минимальная наблюдаемость через foreground logs и runtime-артефакты

Требования:

- не усложнять MVP лишним control plane
- сохранить достаточную диагностируемость
- сделать поведение понятным для terminal-first workflow

## Решение

В MVP отдельный health/status интерфейс не вводится.

Для наблюдаемости достаточно:

- stdout/stderr логов foreground-процесса
- явных сообщений об ошибках у ручных команд `poll` и `run`
- repo-local диагностических артефактов в `.git/.ai-teamlead/`

Отдельная команда `status`, отдельный endpoint или отдельный постоянный status
file в первую версию не входят.

## Последствия

Плюсы:

- меньше сложности в MVP
- меньше дополнительного state и API surface
- проще быстрее перейти к рабочей утилите

Минусы:

- наблюдаемость пока менее формализована
- часть диагностики требует просмотра логов и runtime-артефактов
- в будущем может понадобиться отдельный operator-facing status interface

## Связанные документы

- [docs/features/0001-ai-teamlead-daemon/README.md](/home/danil/code/teamlead/docs/features/0001-ai-teamlead-daemon/README.md)
- [docs/features/0001-ai-teamlead-daemon/03-how-we-verify.md](/home/danil/code/teamlead/docs/features/0001-ai-teamlead-daemon/03-how-we-verify.md)
