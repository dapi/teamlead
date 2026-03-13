# ADR-0002: Standalone foreground daemon для MVP

Статус: accepted
Дата: 2026-03-13

## Контекст

Изначально в качестве базовой модели запуска рассматривался
`systemd --user timer`.

В процессе проектирования MVP стало ясно, что на первом этапе важнее получить
простой и наблюдаемый runtime-контур:

- один процесс
- foreground execution
- собственный polling loop
- минимальная оркестрационная сложность

## Решение

На первом этапе `ai-teamlead` реализуется как standalone daemon, который
работает в foreground и сам выполняет polling loop.

`systemd --user timer` не используется как базовая модель запуска первого MVP и
рассматривается как возможный следующий этап интеграции.

MVP runtime-модель:

- single-process loop
- `max_parallel: 1`
- один экземпляр daemon на один репозиторий

## Последствия

Плюсы:

- проще реализовать и отлаживать
- легче наблюдать lifecycle процесса
- меньше внешних зависимостей в первом этапе
- лучше подходит для dogfooding и ручной разработки

Минусы:

- нужен собственный polling loop
- часть функций supervisor/process manager пока не делегируется systemd
- при дальнейшем росте возможностей может понадобиться дополнительный режим
  запуска

## Связанные документы

- [README.md](/home/danil/code/teamlead/README.md)
- [docs/issue-analysis-flow.md](/home/danil/code/teamlead/docs/issue-analysis-flow.md)
