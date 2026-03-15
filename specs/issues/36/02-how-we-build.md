# Issue 36: Как строим

## Approach

Решение делается как расширение существующего launcher contract без смены
общего flow:

1. Добавить в `LaunchAgentConfig` новый defaulted-блок с per-agent args.
2. Хранить args как `Vec<String>` для каждого агента, чтобы избежать
   shell-splitting по пробелам.
3. Оставить Rust source of truth для parsing/validation/defaults, а в shell
   передавать уже безопасно quoted bash arrays через
   `internal render-launch-agent-context`.
4. Разделить запуск агента в `launch-agent.sh` на отдельные ветки
   `start_codex`, `start_claude`, `start_degraded_shell`, чтобы каждая ветка
   применяла только свои args.
5. Сохранить текущее поведение по умолчанию: без override дополнительные args
   берутся из application defaults, а не из пустого списка.

## Affected Areas

- `src/config.rs`
  новая модель config-поля, defaults и валидация;
- `src/app.rs`
  рендер launch context и shell-safe передача per-agent arrays в launcher;
- `templates/init/settings.yml`
  активные defaults и opt-in примеры для `claude` и `codex`;
- `templates/init/launch-agent.sh`
  branch-specific запуск агента с подстановкой args;
- `./.ai-teamlead/launch-agent.sh`
  dogfooding-копия bootstrap launcher-а в текущем репозитории;
- `tests/integration/*`
  сценарии `run`/`poll`/fallback и общие test helpers/stubs.

## Interfaces And Data

Целевой config contract:

```yaml
launch_agent:
  analysis_branch_template: "analysis/issue-${ISSUE_NUMBER}"
  worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  analysis_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
  global_args:
    claude:
      - "--permission-mode"
      - "auto"
    codex:
      - "--ask-for-approval"
      - "never"
      - "--sandbox"
      - "workspace-write"
```

Семантика:

- `launch_agent.global_args` является optional/defaulted блоком;
- `launch_agent.global_args.claude` и `launch_agent.global_args.codex` являются
  отдельными optional/defaulted полями;
- отсутствие блока или конкретного поля эквивалентно application defaults;
- каждый элемент списка должен быть непустой строкой после `trim()`.

Канонические defaults:

- `codex`: `["--ask-for-approval", "never", "--sandbox", "workspace-write"]`
- `claude`: `["--permission-mode", "auto"]`

Более агрессивные значения считаются opt-in overrides, а не default-layer.

Почему не raw string:

- список строк прозрачно валидируется;
- одна строка не требует shell-парсинга;
- аргумент вида `--flag=value with spaces` не ломается на два токена.

Предпочтительный bootstrap-фрагмент в шаблоне:

```yaml
launch_agent:
  analysis_branch_template: "analysis/issue-${ISSUE_NUMBER}"
  worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  analysis_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
  global_args:
    claude:
      - "--permission-mode"
      - "auto"
    codex:
      - "--ask-for-approval"
      - "never"
      - "--sandbox"
      - "workspace-write"
  # opt-in example:
  # global_args:
  #   claude:
  #     - "--dangerously-skip-permissions"
```

Граница Rust -> shell:

- Rust читает YAML, применяет defaults и валидирует данные;
- `internal render-launch-agent-context` дополнительно рендерит shell-quoted
  bash arrays, например `CLAUDE_GLOBAL_ARGS=(...)` и `CODEX_GLOBAL_ARGS=(...)`;
- `launch-agent.sh` больше не интерпретирует YAML сам и не делает свой
  собственный shell-splitting.

Поведение launcher-а:

1. `codex` доступен:
   `codex --cd "$WORKTREE_ROOT" --no-alt-screen "${CODEX_GLOBAL_ARGS[@]}" "$prompt"`
2. `codex` недоступен, но `claude` доступен:
   launcher запускает `claude` и подставляет только `CLAUDE_GLOBAL_ARGS`
3. ни один агент не доступен:
   launcher сохраняет текущий degraded shell fallback

Ветка `claude` считается не новой продуктовой функцией, а выравниванием
template launcher-а с уже зафиксированным doc contract "`codex` или `claude`".

## Configuration And Runtime Assumptions

- новые поля являются `defaulted-by-application`, а не
  `required-without-default`;
- `templates/init/settings.yml` должен показывать реальные defaults как активную
  часть generated конфига;
- opt-in dangerous overrides должны оставаться только примерами или явными
  пользовательскими изменениями;
- текущие templates branch/worktree/artifacts не меняются;
- `render-launch-agent-context` остается единственной точкой, которая знает,
  как превратить config в shell-safe launcher context;
- runtime-логи должны различать выбранную ветку запуска (`codex`, `claude`,
  `degraded`), но не обязаны печатать полные значения args.

## Risks

- если args будут рендериться как одна shell-строка, появится регрессия по
  quoting и shell injection;
- если defaults будут зафиксированы только в шаблоне, а не в Rust default-layer,
  появится расхождение между новым bootstrap и существующими конфигами;
- переход от `codex-only` launcher-а к `codex`/`claude` ветвлению может
  незаметно поменять degraded path, если не покрыть сценарий отсутствия обоих
  бинарников;
- логирование сырых аргументов может засветить чувствительные значения;
- частично заполненный YAML-блок `global_args` может вести себя неочевидно,
  если defaults и validation разойдутся;
- dogfooding launcher в `./.ai-teamlead/launch-agent.sh` и bootstrap template
  `templates/init/launch-agent.sh` могут разъехаться без отдельной проверки.

## External Interfaces

Внешние интерфейсы:

- `codex` CLI
- `claude` CLI
- shell entrypoint `./.ai-teamlead/launch-agent.sh`

Практическое требование для реализации:

- порядок флагов должен соответствовать реальному CLI каждого инструмента;
- tests не должны зависеть от реального сетевого вызова агентских CLI, а
  должны использовать stubs и фиксировать argv;
- zellij-related integration path должен оставаться headless и запускаться
  через Docker runner, а не в host session пользователя.

## ADR Impact

Новый ADR не требуется.

Причина:

- issue расширяет уже принятый family contract из ADR-0015 и ADR-0016;
- execution model, ownership границ и launcher responsibility не меняются;
- решение является локальным расширением существующего config contract и может
  быть зафиксировано обновлением feature-спек и связанных docs.

Нужно синхронизировать:

- feature 0002 для bootstrap `settings.yml`
- feature 0003 для launcher contract и деградированного fallback path
- при необходимости docs вокруг guardrail из issue `#33`

## Alternatives Considered

### 1. Хранить args одной shell-строкой

Отклонено.

Это ухудшает валидацию, заставляет launcher делать shell-parsing и делает
границу безопасного quoting неявной.

### 2. Добавить `claude_args` и `codex_args` как два плоских top-level поля

Не выбрано.

Такой вариант рабочий, но хуже группирует данные по launcher contract. Вложенный
блок `launch_agent.global_args.*` лучше показывает, что речь идет именно о
launcher-level настройке.

### 3. Передавать args в launcher через отдельный JSON-файл runtime state

Отклонено.

Это добавляет лишний runtime-артефакт, хотя нужные данные уже есть в config и
могут быть безопасно отрендерены через существующий internal command path.

## Migration Or Rollout Notes

- существующие `settings.yml` без `global_args` должны продолжить загружаться
  без миграции;
- rollout считается обратно совместимым по схеме, но меняет runtime behavior:
  старые конфиги без override начнут получать canonical defaults;
- это нужно явно зафиксировать как осознанное изменение контракта относительно
  прежней версии анализа, где default предполагался пустым;
- тесты на `claude` path должны использовать отдельный stub и запуск без
  доступного `codex`, чтобы избежать ложноположительного прохождения;
- dogfooding-копия `./.ai-teamlead/launch-agent.sh` должна обновляться
  одновременно с bootstrap template.
