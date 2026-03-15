# Дизайн: complete-stage — обратный канал из agent session в core

Дата: 2026-03-14
ADR: 0020
Issue: #15
Статус: выполнен

## Проблема

Agent session завершает анализ, но:

1. Артефакты остаются незакоммиченными в worktree
2. Analysis branch не запушен
3. Draft PR не создан
4. Статус issue в GitHub Project не обновлён

## Решение

Новая CLI-команда `ai-teamlead internal complete-stage`, которую агент
вызывает в конце работы. Команда инкапсулирует все finalization-операции.

## Компоненты изменений

### 1. Rust: новая подкоманда `internal complete-stage`

Файл: `src/cli.rs` — добавить вариант в enum `InternalCommand`.

Файл: `src/app.rs` или отдельный `src/complete_stage.rs` — логика:

```
fn run_complete_stage(session_uuid, outcome, repo_root) -> Result<()>
    1. load session manifest из {repo_root}/.git/.ai-teamlead/sessions/{uuid}/
    2. определить worktree_root, branch, issue_number из session + env vars
    3. if есть изменения в specs/issues/{N}/:
         git add specs/issues/{N}/
         git commit -m "analysis(#{N}): {agent_provided_message}"
         git push origin {branch}
    4. if outcome == plan-ready:
         gh pr create --draft или найти существующий
    5. update GitHub Project status по outcome
    6. update session.json: status → completed
    7. update issues/{N}.json: last_known_flow_status
```

### 2. Промпт: секция завершения

Файл: `.ai-teamlead/flows/issue-analysis-flow.md` — добавить в конец:

```markdown
## Завершение анализа

После завершения работы вызови ОДНУ из команд:

Если SDD-комплект собран и полон:
  $AI_TEAMLEAD_BIN internal complete-stage $AI_TEAMLEAD_SESSION_UUID \
    --outcome plan-ready \
    --message "краткое описание результата"

Если нужны ответы пользователя:
  $AI_TEAMLEAD_BIN internal complete-stage $AI_TEAMLEAD_SESSION_UUID \
    --outcome needs-clarification \
    --message "краткое описание вопросов"

Если заблокирован:
  $AI_TEAMLEAD_BIN internal complete-stage $AI_TEAMLEAD_SESSION_UUID \
    --outcome blocked \
    --message "причина блокировки"

Команда сама выполнит коммит, пуш и создание draft PR.
НЕ выполняй git commit, git push, gh pr create самостоятельно.

Нотация commit message: analysis(#N): <описание>
Нотация PR title: analysis(#N): <описание>
В PR body укажи Ref #N и список артефактов.
```

### 3. SSOT: обновление docs/issue-analysis-flow.md

- Убрать "orchestration создания коммита или PR" из секции "Вне scope"
- Добавить секцию "Контракт завершения стадии" с описанием `complete-stage`
- Обновить журнал изменений

## Определение repo root из worktree

Приоритет:

1. Env var `AI_TEAMLEAD_REPO_ROOT` (уже экспортируется launcher-ом)
2. Fallback: `git worktree list --porcelain` → первая запись = primary repo

## Передача commit message

Агент передаёт `--message` с кратким описанием. Команда `complete-stage`
форматирует финальный commit message:

```
analysis(#{issue_number}): {agent_message}
```

Аналогично для PR title. Агент отвечает за содержательную часть, команда —
за нотацию.

## Обработка ошибок

| Ситуация | Поведение |
|-|-|
| Нет изменений для коммита | Пропустить git add/commit, продолжить |
| Push failed | Ошибка, не менять статус, exit 1 |
| PR create failed | Предупреждение, продолжить со сменой статуса |
| Status update failed | Ошибка, session остаётся active, exit 1 |
| Невалидный outcome | Ошибка, exit 1 |
| Session not found | Ошибка, exit 1 |
| Повторный вызов (session already completed) | Предупреждение, exit 0 |

## Что НЕ меняется

- `launch-agent.sh` — без изменений
- Runtime state layout — без изменений
- Существующие CLI-команды — без изменений
- GitHub Project статусы — без изменений
- Staged prompts (01/02/03) — без изменений

## План реализации

1. Rust: добавить `CompleteStage` в `InternalCommand` enum
2. Rust: реализовать логику complete-stage (git, gh, GitHub API)
3. Обновить flow prompt
4. Обновить SSOT
5. Integration test: stub-агент вызывает complete-stage
