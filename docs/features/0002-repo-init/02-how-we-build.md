# Feature 0002: Как строим

## Архитектура

`init` реализуется как отдельная ручная CLI-команда:

- `ai-teamlead init`

Команда:

1. определяет текущий git-репозиторий
2. читает `origin` и определяет GitHub repo context текущего проекта
3. вычисляет пути versioned project contract layer
4. создает недостающие каталоги
5. создает стартовые project-local файлы из встроенных шаблонов
6. печатает оператору, что было создано, а что уже существовало

## Данные и состояния

Ключевые входные данные:

- текущий `repo_root`
- `origin` текущего git-репозитория
- набор встроенных шаблонов `init`

Ключевые результирующие артефакты:

- `./.ai-teamlead/settings.yml`
- `./.ai-teamlead/README.md`
- `./.ai-teamlead/init.sh`
- `./.ai-teamlead/launch-agent.sh`
- `./.ai-teamlead/zellij/analysis-tab.kdl`
- `./.ai-teamlead/flows/issue-analysis-flow.md`
- `./.ai-teamlead/flows/issue-analysis/README.md`
- `./.ai-teamlead/flows/issue-analysis/01-what-we-build.md`
- `./.ai-teamlead/flows/issue-analysis/02-how-we-build.md`
- `./.ai-teamlead/flows/issue-analysis/03-how-we-verify.md`
- `./.claude/README.md`
- `./.codex/README.md`
- `./init.sh -> ./.ai-teamlead/init.sh`, если `./init.sh` отсутствовал

Состояния команды:

- репозиторий найден
- project contract layer отсутствует полностью
- project contract layer существует частично
- все обязательные файлы уже существуют
- инициализация завершена

## Интерфейсы

Внешние интерфейсы:

- локальный git-репозиторий как источник `repo_root`
- `origin` как источник GitHub repo context
- файловая система рабочего дерева
- stdout/stderr для отчета о результате

Внутренние интерфейсы:

- repo context discovery
- вычисление project-local путей
- запись встроенных шаблонов

## Технические решения

Уже принятые решения:

- команда называется `init`, а не `bootstrap`
- versioned project contract layer живет в `./.ai-teamlead/`
- runtime state живет отдельно в `.git/.ai-teamlead/`
- `settings.yml` хранится в `./.ai-teamlead/settings.yml`
- типовой `init.sh` bootstrap-ится в `./.ai-teamlead/init.sh`
- типовой `launch-agent.sh` bootstrap-ится в `./.ai-teamlead/launch-agent.sh`
- `issue-analysis-flow` bootstrap-ится как repo-local markdown-документ
- типовой `init.sh` копирует в новый worktree отсутствующие `.env*` из primary
  worktree, если primary worktree сейчас находится на default branch
- `init` использует уже настроенный GitHub `origin` как обязательный repo
  context
- существующие файлы не перезаписываются
- root-level `./init.sh` создается только как симлинк и только если его еще нет

`init` не должен:

- создавать lock-файлы или session-артефакты
- читать GitHub Project
- обращаться к GitHub API по сети
- запускать `poll` или `run`
- запускать `zellij`

## Конфигурация

`init` не требует существующего `./.ai-teamlead/settings.yml` на входе.

Он создает стартовый шаблон `settings.yml`, в котором:

- `github.project_id` показывается как закомментированный placeholder,
  требующий ручной донастройки;
- все поля с canonical runtime default показываются как закомментированные
  documented defaults, а не как обязательный активный YAML;
- `zellij.session_name` документируется как template `${REPO}`;
- `zellij.tab_name` документируется со значением `issue-analysis`;
- `zellij.tab_name_template` документируется как application default
  `#${ISSUE_NUMBER}` для `tab`-режима и показывается в шаблоне как
  закомментированный documented default;
- `zellij.layout` документируется со значением `compact` как opt-in пример, а
  не как runtime default;
- `./.ai-teamlead/zellij/analysis-tab.kdl` bootstrap-ится как versioned template
  для analysis tab с placeholders `${TAB_NAME}` и `${PANE_ENTRYPOINT}` и
  встроенным `compact-bar` как bootstrap default для tab-level UX;
- `launch_agent.analysis_branch_template` документируется как
  `analysis/issue-${ISSUE_NUMBER}`;
- `launch_agent.worktree_root_template` документируется как
  `${HOME}/worktrees/${REPO}/${BRANCH}`;
- `launch_agent.analysis_artifacts_dir_template` документируется как
  `specs/issues/${ISSUE_NUMBER}`;
- `launch_agent.global_args.codex` и `launch_agent.global_args.claude`
  документируются как canonical runtime defaults;
- более агрессивные launcher args, например
  `--dangerously-skip-permissions` для `claude`, показываются только как
  opt-in пример и не входят в runtime default-layer.

Runtime loading при этом строится как `defaults + active YAML overrides`:

- `github.project_id` остается `required-without-default`;
- отсутствующие defaulted-поля подставляются из canonical Rust default-layer;
- `launch_agent.global_args.*` при отсутствии active override берутся из runtime
  defaults приложения;
- `zellij.tab_name_template` входит в canonical runtime default-layer:
  отсутствие active override все равно дает issue-aware tab naming в
  `tab`-режиме;
- `zellij.layout` остается допустимым `example-only extension`: шаблон
  показывает, как включить custom layout, но отсутствие active override не
  меняет launcher path;
- comment-only `settings.yml` допустим как bootstrap состояние, но `poll`/`run`
  все равно требуют, чтобы оператор задал `github.project_id`.

Операторские действия после `init`:

1. раскомментировать и заменить `github.project_id` placeholder на реальный
   GitHub Project id
2. при необходимости раскомментировать и скорректировать literal или template
   `zellij.session_name`
3. при необходимости раскомментировать и скорректировать
   `zellij.tab_name_template`, `zellij.layout` и
   `./.ai-teamlead/zellij/analysis-tab.kdl`
4. при необходимости раскомментировать и скорректировать `launch_agent.*`
   templates
5. при необходимости заменить canonical agent defaults своими override-args
6. только после этого запускать `poll` или `run`

Если placeholder не заменен или id невалиден, текущая реализация не проходит
этап загрузки project snapshot и завершает `poll`/`run` ошибкой.

## Ограничения реализации

- первая версия использует встроенные шаблоны, а не внешний template registry
- первая версия не спрашивает пользователя о значениях через интерактивный
  мастер
- первая версия не пытается мигрировать уже существующие project-local файлы
- первая версия считает успешной инициализацию частично существующего набора,
  если недостающие файлы были добавлены без перезаписи

## Contract layer

Versioned project contract layer в первой версии выглядит так:

```text
.ai-teamlead/
  README.md
  settings.yml
  init.sh
  launch-agent.sh
  zellij/
    analysis-tab.kdl
  flows/
    issue-analysis-flow.md
    issue-analysis/
      README.md
      01-what-we-build.md
      02-how-we-build.md
      03-how-we-verify.md
```

Дополнительно bootstrap-ятся project-local каталоги:

```text
.claude/
  README.md

.codex/
  README.md
```

Ephemeral runtime layer при этом остается отдельным:

```text
.git/.ai-teamlead/
```

При этом `init` может дополнительно создать корневой симлинк:

```text
./init.sh -> ./.ai-teamlead/init.sh
```

## Поведение `init.sh`

Bootstrapped `./.ai-teamlead/init.sh` выполняется уже внутри конкретного
worktree и в первой версии делает только безопасный локальный bootstrap:

- при наличии `mise.toml` выполняет `mise trust` и `mise install`
- при наличии `.gitmodules` выполняет `git submodule update --init --recursive`
- копирует отсутствующие `.env*` из primary worktree
- при наличии `.envrc` выполняет `direnv allow`

Default branch определяется в таком порядке:

1. `refs/remotes/origin/HEAD`
2. `gh repo view --json defaultBranchRef`
3. `git remote show -n origin`
4. локальные ветки `main` / `master`
5. текущая ветка

Путь к primary worktree определяется через `git rev-parse --git-common-dir`, а
не через перебор `git worktree list`.
