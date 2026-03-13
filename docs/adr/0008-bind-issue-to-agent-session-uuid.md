# ADR-0008: Durable-связка issue и агентской сессии через `session_uuid`

Статус: accepted
Дата: 2026-03-13

## Контекст

Для `issue-analysis-flow` нужно было сделать вопросы, план и human gate
достаточно durable, не превращая GitHub comments в основной канал общения.

Требования:

- вопросы и план должны жить не только в памяти текущего процесса
- issue должна быть однозначно связана с конкретной агентской сессией
- должно быть понятно, в какой `zellij` panel запущена эта сессия
- связь должна быть repo-local и не должна подменять GitHub Project как source
  of truth по статусу issue

## Решение

Каждая issue, взятая в анализ, получает ровно одну связанную агентскую сессию.

Для этой сессии заранее генерируется `session_uuid`, который:

- передается в запуск агентской сессии в `zellij`
- сохраняется как durable repo-local артефакт
- связывается с issue в отношении `1 <-> 1`

Durable-артефакты сессии должны сохранять как минимум:

- `session_uuid`
- номер issue
- `zellij.session_id`
- `zellij.tab_id`
- `zellij.pane_id`
- список заданных вопросов
- последний опубликованный пакет анализа и план
- журнал нормализованных действий оператора

Durable-артефакты сессии хранятся в `.git/ai-teamlead/`.

## Последствия

Плюсы:

- вопросы и план не теряются вместе с одним процессом daemon
- human gate можно привязать к проверяемым session-артефактам
- сохраняется агентская модель общения без GitHub comments как основного канала
- можно однозначно определить, в какой `zellij` panel живет агентская сессия

Минусы:

- появляется дополнительный repo-local durable слой
- нужно проектировать формат session-артефактов и событий оператора

## Связанные документы

- [docs/issue-analysis-flow.md](/home/danil/code/teamlead/docs/issue-analysis-flow.md)
- [docs/adr/0004-runtime-artifacts-in-git-dir.md](/home/danil/code/teamlead/docs/adr/0004-runtime-artifacts-in-git-dir.md)
- [docs/features/0001-ai-teamlead-daemon/README.md](/home/danil/code/teamlead/docs/features/0001-ai-teamlead-daemon/README.md)
