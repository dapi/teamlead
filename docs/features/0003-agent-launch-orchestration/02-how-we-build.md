# Feature 0003: Как строим

## Архитектура

Orchestration flow делится на две части:

### 1. `ai-teamlead`

Отвечает за:

- выбор issue через `poll` или `run`
- смену статуса issue в GitHub Project
- генерацию `session_uuid`
- создание или поиск нужного `zellij` session/tab
- открытие новой pane
- запуск versioned `./.ai-teamlead/launch-agent.sh`

### 2. `./.ai-teamlead/launch-agent.sh`

Отвечает за:

- привязку текущей pane к `session_uuid`
- создание или переиспользование branch/worktree
- запуск `init.sh`, если это нужно проекту
- запуск реального агента (`codex` или `claude`)
- project-local правила по analysis branch/worktree lifecycle

## Контракт launcher

`launch-agent.sh`:

- хранится в `./.ai-teamlead/launch-agent.sh`
- является versioned project-local script
- запускается из корня репозитория как `cwd`
- принимает аргументы в таком порядке:
  - `<session_uuid>`
  - `<issue_url>`
- должен сам подготовить analysis worktree до запуска реального агента
- должен использовать naming/path templates из `./.ai-teamlead/settings.yml`

`issue-analysis-flow`:

- хранится как entrypoint в `./.ai-teamlead/flows/issue-analysis-flow.md`
- внутри маршрутизирует агента на staged prompts в
  `./.ai-teamlead/flows/issue-analysis/`
- передается реальному агенту уже из `launch-agent.sh`

## Zellij context

В `settings.yml` фиксируются versioned fallback-поля launcher context:

- `zellij.session_name`
- `zellij.tab_name`
- `zellij.launch_target` со значениями `pane | tab`
- optional `zellij.tab_name_template` для issue-aware имени tab на tab-launch
  path
- `zellij.layout` как optional layout name для branch создания новой session

Bootstrap default:

- `session_name = ${REPO}`
- `tab_name = issue-analysis`
- `launch_target = tab`

Runtime правила:

- target session определяется в порядке:
  `--zellij-session` -> `ZELLIJ_SESSION_NAME` -> `zellij.session_name`
- effective launch target для `run` определяется в порядке:
  `--launch-target` -> `zellij.launch_target` -> runtime default `tab`
- `poll` и `loop` не имеют отдельного public override и используют только
  config/default target
- если effective target session уже существует, используется она
- если effective target session отсутствует, `ai-teamlead` создает ее:
  - через `zellij.layout`, если поле задано
  - через default session-create path без bare generated analysis layout, если
    поле отсутствует
- если launch target = `pane`:
  - `ai-teamlead` ищет shared tab по stable `zellij.tab_name`
  - при единственном совпадении открывает новую pane внутри этого tab
  - при отсутствии shared tab сначала создает его через versioned
    `analysis-tab.kdl`
  - при нескольких совпадениях завершает запуск явной ошибкой
- если launch target = `tab`:
  - `ai-teamlead` создает отдельный analysis tab через versioned layout
  - если `zellij.tab_name_template` задан, runtime рендерит effective tab name
    из `${ISSUE_NUMBER}` до генерации `launch-layout.kdl`
  - если `zellij.tab_name_template` не задан, runtime использует stable
    `zellij.tab_name`
- analysis tab не должна выглядеть как bare technical tab, если project-local
  contract ожидает bar/plugins и другой tab-level UX
- для каждого запуска issue-analysis открывается новая pane
- existing session с panes из другого GitHub repo считается недопустимой
  и отклоняется до запуска

После старта pane:

- `launch-agent.sh` вызывает
  `ai-teamlead internal bind-zellij-pane <session_uuid>`
- эта команда читает `ZELLIJ_PANE_ID`
- `pane_id` дописывается в runtime state

## Lifecycle `launch-agent.sh`

Минимальный lifecycle первой версии:

1. Принять `<session_uuid>` и `<issue_url>`.
2. Вызвать `ai-teamlead internal bind-zellij-pane <session_uuid>`.
3. Определить или создать analysis branch/worktree для issue.
4. Перейти в корень analysis worktree.
5. Запустить `./init.sh`, если он существует в worktree.
6. Убедиться, что существует каталог versioned analysis-артефактов.
7. Запустить реального агента с:
   - `./.ai-teamlead/flows/issue-analysis-flow.md`
   - URL issue

Bootstrap default для первой версии:

- если в окружении доступен `codex`, launcher стартует его интерактивно
- если `codex` отсутствует, launcher оставляет пользователя в shell внутри
  подготовленного analysis worktree
- это считается допустимым degraded mode, а не отдельным типом analysis flow

Инварианты:

- branch/worktree должны быть готовы до запуска `codex` или `claude`
- каталог versioned analysis-артефактов должен существовать до старта агента
- `issue-analysis-flow` не отвечает за эти git-операции
- naming и пути берутся из `settings.yml`, а их применение остается
  ответственностью `launch-agent.sh`
- при повторном `run` в waiting-статусах используется существующий
  `session_uuid`, но все равно создается новая pane для нового launcher path

## Corner cases

### 1. Session с нужным именем уже существует

Поведение:

- использовать существующую session
- не создавать вторую session с тем же semantic назначением
- перед запуском проверить, что existing session не смешивает несколько repo

### 2. Session была раньше, но сейчас отсутствует

Поведение:

- создать новую session с тем же `session_name`
- считать это нормальным recreate, а не ошибкой

### 3. Session resurrect-нулась после внешнего восстановления

Поведение:

- если она доступна по тому же `session_name`, считать ее валидной existing
  session
- не пытаться автоматически различать “живая” это session или “resurrected”
  в первой версии

### 4. Tab с нужным именем уже существует

Поведение:

- в `pane`-режиме использовать этот tab как shared launch context
- в `tab`-режиме не переиспользовать его молча, а создавать новый analysis tab

### 7. Existing session содержит panes другого repo

Поведение:

- launcher анализирует `pane_cwd` и repo context existing session
- если обнаружен другой GitHub repo, запуск завершается ошибкой
- shared multi-repo session не используется как launch context

### 5. Tab с нужным именем отсутствует

Поведение:

- в `pane`-режиме создать shared tab с `tab_name = issue-analysis`
- в `tab`-режиме создать отдельный analysis tab через versioned layout

### 6. Есть несколько tab с одинаковым именем

Поведение:

- это считается нештатным launcher state
- первая версия не должна пытаться “угадывать”
- запуск должен завершаться диагностической ошибкой

## Технические решения

- `poll` и `run` используют один и тот же launch path
- `launch-agent.sh` не генерируется runtime-ом, а bootstrap-ится в проект
- branch/worktree orchestration живет в `launch-agent.sh`, а не в `ai-teamlead`
- `ai-teamlead` не передает `repo_root` аргументом; правильный repo context
  задается через `cwd`
- runtime может генерировать только технический shim `pane-entrypoint.sh` и
  `launch-layout.kdl`, но не несет в них branch/worktree логику
- если `launch-layout.kdl` используется для analysis tab, его source of truth
  должен быть versioned contract/template `.ai-teamlead/zellij/analysis-tab.kdl`,
  а не hardcoded bare layout

## Направление эволюции

В MVP `ai-teamlead` сам содержит `zellij`-specific реализацию поиска или
создания session/tab и открытия pane.

При дальнейшем расширении под `tmux` и другие мультиплексоры эта логика должна
быть вынесена в project-local shell boundary, например:

- `./.ai-teamlead/find-or-create-launch-context.sh`

Ожидаемая ответственность такого script-layer:

- найти или создать launcher context по stable names
- вернуть machine-readable runtime identifiers
- скрыть multiplexer-specific команды от основного Rust-приложения

То есть основной `ai-teamlead` должен эволюционировать в сторону абстракции
launch context provider, а не в сторону вшитого знания о каждом мультиплексоре.

## Конфигурация

Минимально значимые поля для orchestration:

```yaml
zellij:
  session_name: "${REPO}"
  tab_name: "issue-analysis"
  launch_target: "tab"
  # tab_name_template: "#${ISSUE_NUMBER}"
  # layout: "my-custom-layout"

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

Во время запуска `session_name` рендерится тем же template path, что и
`launch_agent.*`, но для него разрешен только `${REPO}`.
Во время tab-launch path `tab_name_template`, если он задан, поддерживает
только `${ISSUE_NUMBER}` и не меняет semantics stable `zellij.tab_name`.
В `pane`-ветке `tab_name_template` игнорируется, потому что shared tab должен
оставаться стабильным semantic context.

Поддерживаемые placeholder-переменные в первой версии:

- `${HOME}`
- `${REPO}`
- `${ISSUE_NUMBER}`
- `${BRANCH}` для worktree root и artifacts dir

Неизвестные placeholder-переменные текущая реализация не валидирует и оставляет
в строке как литералы для `launch_agent.*`.

Для `zellij.session_name` это правило строже:

- literal-значения без placeholder допустимы
- `${REPO}` рендерится из canonical GitHub repo slug
- любые оставшиеся `${...}` считаются ошибкой конфигурации
- полученное значение используется как fallback, если нет CLI override и
  `ZELLIJ_SESSION_NAME`

Для `launch_agent.global_args.*` действуют дополнительные правила:

- значения задаются как список строк, а не как одна shell-строка;
- отсутствие пользовательского override означает application defaults;
- canonical defaults:
  - `codex`: `["--ask-for-approval", "never", "--sandbox", "workspace-write"]`
  - `claude`: `["--permission-mode", "auto"]`
- более агрессивные значения, например
  `["--dangerously-skip-permissions"]` для `claude` или
  `["--dangerously-bypass-approvals-and-sandbox"]` для `codex`, считаются
  opt-in override и не входят в default-layer

## Ограничение минимального generated layout

Минимальный runtime-generated layout удобен как атомарный transport для
`tab + pane + command`, но он не гарантирует продуктовое требование
"analysis tab выглядит как родной tab session".

Поэтому для analysis tab нужен отдельный versioned source of truth для
tab-level UX. Попытка восстановить такой UX из live-state уже открытой session
считается хрупкой и не входит в контракт первой версии.

В текущем контракте таким источником служит project-local template:

- `./.ai-teamlead/zellij/analysis-tab.kdl`
- runtime `launch-layout.kdl` является только отрендеренной копией этого
  template для конкретного `session_uuid`
- обязательные placeholders template:
  `${TAB_NAME}` и `${PANE_ENTRYPOINT}`
