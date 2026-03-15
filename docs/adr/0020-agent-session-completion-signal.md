# ADR-0020: Сигнал завершения agent session через CLI-команду complete-stage

Статус: accepted
Дата: 2026-03-14
Связанный issue: #15

## Контекст

После запуска `ai-teamlead run <issue-url>` агент выполняет анализ в отдельном
worktree внутри `zellij`-сессии. На dogfooding-прогоне по issue #3 агент создал
полный SDD-комплект (`specs/issues/3/`), но:

- не закоммитил артефакты
- не запушил analysis branch
- не создал draft PR
- не перевёл статус issue в GitHub Project

Статус остался `Analysis In Progress`, хотя анализ фактически завершён.

Причина: flow prompt не содержит инструкций по VCS-операциям и смене статуса.
В SSOT `docs/issue-analysis-flow.md` это явно указано как "вне scope". Это
решение было осознанным на момент первого MVP, но dogfooding показал, что без
обратного канала flow выглядит зависшим.

## Рассмотренные варианты

### A: Всё в промпте

Промпт инструктирует агента самостоятельно выполнять `git commit`, `git push`,
`gh pr create` и менять статус issue.

Плюсы: минимум изменений в коде.
Минусы: агент может забыть шаги, выполнить их частично или в неправильном
порядке. VCS-операции рассыпаны по промпту. Ненадёжно и плохо расширяемо.

### B: Wrapper с post-processing

Заменить `exec codex` на обычный вызов. После завершения codex wrapper-скрипт
сам выполняет коммит, пуш, PR и смену статуса.

Плюсы: детерминированный, не зависит от агента.
Минусы: wrapper не знает outcome (plan-ready vs needs-clarification). Придётся
угадывать по наличию файлов, что ненадёжно.

### C: Result file + wrapper

Агент пишет machine-readable файл с outcome. Wrapper читает его после
завершения codex и выполняет VCS-операции.

Плюсы: разделение ответственности.
Минусы: два движущихся компонента (агент + wrapper), дополнительный контракт
на формат файла.

### D: CLI-команда из агента (выбрано)

Новая CLI-команда `ai-teamlead internal complete-stage` инкапсулирует всю
finalization-логику. Агент вызывает одну команду в конце работы.

### E: Гибрид — промпт + wrapper fallback

Агент вызывает `complete-stage`, wrapper проверяет после завершения codex. Если
агент не вызвал команду, wrapper выполняет best-effort fallback.

Плюсы: belt-and-suspenders.
Минусы: over-engineering для MVP. Fallback с эвристиками добавляет сложность
без пропорциональной надёжности.

## Решение

Выбран вариант D: CLI-команда `ai-teamlead internal complete-stage`.

### Обоснование

1. **Агент знает outcome** — только агент понимает, нужны ли clarifications или
   план готов. Wrapper (вариант B) вынужден угадывать по файлам.

2. **VCS-операции инкапсулированы** — одна команда делает commit + push +
   draft PR + status change. Агент не выполняет git-команды самостоятельно.

3. **Идиоматично для проекта** — расширяет существующий паттерн
   `ai-teamlead internal bind-zellij-pane`, который уже вызывается из
   `launch-agent.sh`.

4. **Env vars уже готовы** — `AI_TEAMLEAD_BIN`, `AI_TEAMLEAD_SESSION_UUID`,
   `AI_TEAMLEAD_REPO_ROOT` уже экспортируются launcher-ом.

5. **Минимальная зависимость от агента** — одна строка вызова. Если агент не
   вызовет, статус остаётся `Analysis In Progress`, артефакты сохранены в
   worktree, оператор разберётся вручную.

## Спецификация

### CLI-контракт

```
ai-teamlead internal complete-stage <session_uuid> --outcome <outcome>
```

Допустимые значения `outcome`:

- `plan-ready` — анализ завершён, SDD-комплект собран
- `needs-clarification` — для продолжения нужны ответы пользователя
- `blocked` — анализ заблокирован технической проблемой

### Что делает команда

Для `plan-ready`:

1. Определяет worktree и analysis branch из session context
2. `git add` артефактов в `specs/issues/{N}/`
3. `git commit` с сообщением по нотации (см. ниже)
4. `git push origin <analysis-branch>`
5. `gh pr create --draft` (или находит существующий draft PR)
6. Меняет статус issue в GitHub Project → `Waiting for Plan Review`
7. Обновляет `session.json`: `status → completed`
8. Выводит диагностику в stdout

Для `needs-clarification`:

1. Коммит и пуш артефактов (если есть)
2. Меняет статус → `Waiting for Clarification`
3. Обновляет `session.json`: `status → completed`

Для `blocked`:

1. Коммит и пуш артефактов (если есть)
2. Меняет статус → `Analysis Blocked`
3. Обновляет `session.json`: `status → completed`

### Нотация именования

Номер GitHub issue (`#{N}`) обязателен в первой строке commit message, в
названии ветки и в заголовке PR.

Ветка (уже задаётся launcher-ом):

```
analysis/issue-{N}
```

Commit message:

```
analysis(#{N}): <краткое описание результата>

<опциональное тело>
```

Примеры:

```
analysis(#3): SDD-комплект для repo init user story
analysis(#15): вопросы по контракту обратного канала
analysis(#12): заблокировано — нет доступа к staging API
```

PR title:

```
analysis(#{N}): <краткое описание>
```

PR body должен содержать:

- ссылку на GitHub issue (`Closes #N` или `Ref #N`)
- список созданных артефактов
- outcome (plan-ready / needs-clarification / blocked)

Агент формирует текст commit message и PR title самостоятельно, соблюдая
нотацию выше.

### Определение repo root из worktree

Команда `complete-stage` вызывается из worktree, но runtime state
(`.git/.ai-teamlead/sessions/`) находится в primary repo.

Решение: `launch-agent.sh` уже экспортирует `AI_TEAMLEAD_REPO_ROOT` (строки
7, 140), указывающий на primary repo root. Команда `complete-stage` читает эту
переменную для доступа к session state.

Fallback: если env var не задан, команда может вычислить primary repo root
через `git worktree list --porcelain` (первая запись всегда primary).

### Изменения в flow prompt

В `issue-analysis-flow.md` добавляется секция `Завершение анализа` с
инструкцией вызвать `complete-stage` с нужным outcome. Явно указывается, что
агент НЕ должен выполнять `git commit`, `git push` или `gh pr create`
самостоятельно — всё инкапсулировано в команде.

### Изменения в SSOT

В `docs/issue-analysis-flow.md` секция "Вне scope" обновляется: убирается
пункт "orchestration создания коммита или PR", добавляется описание
`complete-stage` как контракта завершения стадии.

### Обработка ошибок

- Если `complete-stage` не может закоммитить (нет изменений) — не фатально,
  продолжает со сменой статуса
- Если не может запушить — выводит ошибку, не меняет статус
- Если не может создать PR — выводит предупреждение, продолжает со сменой
  статуса
- Если не может сменить статус в GitHub Project — выводит ошибку, сессия
  остаётся `active`
- При любой ошибке — диагностика в stderr, ненулевой exit code

### Диагностика при отсутствии вызова

Если агент не вызвал `complete-stage`:

- Статус остаётся `Analysis In Progress`
- Оператор видит это в GitHub Project
- Артефакты сохранены локально в worktree (не потеряны)
- Оператор может зайти в worktree, проверить и вызвать `complete-stage`
  вручную

## Последствия

- Появляется новая CLI-подкоманда `internal complete-stage` в Rust-коде
- Flow prompt получает секцию с инструкцией по завершению
- SSOT обновляется для отражения нового контракта
- `launch-agent.sh` не меняется
- Runtime state layout не меняется (используется существующий `session.json`)
- Паттерн `internal`-подкоманд закрепляется как стандартный способ
  взаимодействия agent session → core

## Журнал изменений

### 2026-03-14

- создан ADR
