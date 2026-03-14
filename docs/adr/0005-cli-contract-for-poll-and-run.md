# ADR-0005: CLI-контракт команд `poll` и `run`

Статус: accepted
Дата: 2026-03-13

## Контекст

Для первой версии `ai-teamlead` нужно определить минимальный CLI-контракт.

Требования:

- команда должна быть простой и терминальной
- ручной запуск должен соответствовать тем же правилам, что и `poll`
- CLI не должен вводить отдельную альтернативную модель состояний
- контракт должен быть достаточен для MVP без избыточного набора команд

Рассматривались варианты:

- только одна ручная команда
- отдельные команды `poll` и `run`
- более широкое CLI с `status`, `retry`, `doctor` и другими подкомандами

## Решение

В MVP фиксируются две ручные команды:

- `poll`
- `run`

Назначение:

- `poll` инициирует один цикл поиска и claim подходящей issue по тем же
  правилам, что и polling cycle
- `run` запускает analysis flow для явно указанной issue при соблюдении правил
  допустимых входных статусов

Базовый контракт:

- `poll` не принимает issue как аргумент
- `run` принимает идентификатор issue или URL issue
- `poll` и `run` могут принимать optional override
  `--zellij-session <SESSION>` для target launcher context
- обе команды работают в контексте текущего репозитория и его
  `./.ai-teamlead/settings.yml`
- обе команды используют ту же статусную модель GitHub Project

## Последствия

Плюсы:

- CLI остается минимальным
- ручное и автоматическое поведение не расходятся
- проще тестировать и отлаживать MVP

Минусы:

- пока нет отдельной команды диагностики
- часть операторских сценариев временно решается через существующие две команды

## Связанные документы

- [docs/issue-analysis-flow.md](/home/danil/code/teamlead/docs/issue-analysis-flow.md)
- [docs/features/0001-ai-teamlead-daemon/README.md](/home/danil/code/teamlead/docs/features/0001-ai-teamlead-daemon/README.md)
- [docs/adr/0021-zellij-session-target-resolution.md](/home/danil/code/teamlead/docs/adr/0021-zellij-session-target-resolution.md)

## Журнал изменений

### 2026-03-14

- добавлен optional CLI override `--zellij-session <SESSION>` для `poll` и
  `run`
