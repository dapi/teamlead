# ADR-0021: Приоритет разрешения target session для `zellij`

Статус: accepted
Дата: 2026-03-14

## Контекст

До этого `ai-teamlead` выбирал target session для launcher только по
`zellij.session_name` из `settings.yml`.

Этого недостаточно для интерактивного сценария, когда оператор уже находится
внутри нужной `zellij` session и ожидает, что новый tab/pane откроется именно
там, а не в отдельной session по fallback-имени из конфига.

Дополнительно нужно было зафиксировать:

- одинаковое поведение для `poll` и `run`
- явный приоритет между CLI override, текущим runtime-контекстом и fallback
  конфигом
- запрет на использование одной existing `zellij` session для нескольких
  GitHub-репозиториев

## Решение

Target session для launcher определяется в таком порядке:

1. явный CLI override `--zellij-session <SESSION>`
2. `ZELLIJ_SESSION_NAME` из окружения текущего процесса
3. `zellij.session_name` из `./.ai-teamlead/settings.yml`

Это правило действует одинаково для:

- `poll`
- `run`

При этом:

- `zellij.session_name` остается versioned project-local fallback-полем
- bootstrap default для `zellij.session_name` остается `${REPO}`
- `zellij.tab_name` остается versioned именем tab внутри выбранной session

Для existing session вводится дополнительный runtime guard:

- перед открытием нового tab/pane `ai-teamlead` проверяет panes target session
- если в existing session обнаружены panes из другого GitHub repo, запуск
  завершается ошибкой
- использовать одну shared `zellij` session для нескольких репозиториев
  запрещено

## Последствия

Плюсы:

- интерактивный запуск по умолчанию попадает в текущую session оператора
- у оператора остается явный override для нестандартного target session
- `poll` и `run` используют одинаковый контракт выбора session
- сохраняется fallback для запуска вне `zellij`

Минусы:

- launcher теперь зависит не только от config, но и от runtime env
- для existing session появляется дополнительная проверка pane metadata
- shared multi-repo sessions теперь отбрасываются явно, а не неявно

## Связанные документы

- [ADR-0005](./0005-cli-contract-for-poll-and-run.md)
- [ADR-0014](./0014-zellij-launch-context-naming.md)
- [README.md](../../README.md)
- [Issue Analysis Flow](../issue-analysis-flow.md)
- [Feature 0003](../features/0003-agent-launch-orchestration/README.md)

## Журнал изменений

### 2026-03-14

- зафиксирован приоритет `args -> env -> settings` для target `zellij`
  session
- правило распространено и на `poll`, и на `run`
- добавлен запрет на shared multi-repo existing sessions
