# Feature 0001: Что строим

## Проблема

Нужен инструмент, который может автономно жить внутри конкретного репозитория,
периодически проверять GitHub Project, забирать подходящую issue в анализ и
запускать agent-driven flow без ручного микроменеджмента каждого шага.

При этом инструмент должен:

- быть пригодным для личного использования
- оставаться переносимым между разными репозиториями
- не опираться на скрытый глобальный state

## Пользователь

Основной пользователь первой версии:

- автор проекта
- продвинутый Linux-пользователь
- разработчик, который работает из терминала и готов к repo-local настройке

Вторичная аудитория:

- другие хардкорные Linux-разработчики с похожим workflow

## Результат

Полезным результатом первой версии считается CLI-утилита, которая:

- запускается в foreground
- работает в контексте одного репозитория
- читает `./.ai-teamlead/settings.yml` из репозитория
- умеет выполнить один selection cycle через `poll`
- умеет выполнить issue-level запуск через `run`
- умеет выполнять непрерывный foreground loop через `loop`
- переводит issue в `Analysis In Progress` внутри общего issue-level `run`-path
- запускает `issue-analysis-flow` в настроенной `zellij` session и tab

## Scope

В первую версию входит:

- CLI-утилита с командами `init`, `poll`, `run`, `loop`
- repo-local конфиг `./.ai-teamlead/settings.yml`
- использование versioned project-local contract из `./.ai-teamlead/`
- one-shot selection cycle через `poll`
- общий issue-level orchestration path через `run`
- foreground loop поверх `poll`
- выбор одной issue в рамках `max_parallel: 1`
- запуск `issue-analysis-flow`
- ручные команды `poll`, `run`, `loop`

## Вне scope

- автоматическая реализация issue
- создание ветки, worktree, коммитов и PR самим `ai-teamlead` (эта ответственность
  делегирована project-local `launch-agent.sh`, см.
  [Feature 0003](../0003-agent-launch-orchestration/README.md))
- web UI
- глобальный shared control plane для нескольких репозиториев
- постоянная локальная база состояния issue

## Ограничения и предпосылки

- источник истины по состоянию issue находится в GitHub Project
- вопросы пользователю задаются в агентской сессии
- flow работает по status model, описанной в `issue-analysis-flow`
- на первом этапе целевой режим это `max_parallel: 1`
