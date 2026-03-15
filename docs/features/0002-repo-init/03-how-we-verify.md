# Feature 0002: Как проверяем

## Критерии корректности

Решение считается корректным, если:

- `init` работает только в git-репозитории с настроенным GitHub `origin`
- `init` создает `./.ai-teamlead/`, если каталог отсутствует
- `init` создает `./.ai-teamlead/settings.yml`
- `init` создает `./.ai-teamlead/README.md`
- `init` создает `./.ai-teamlead/init.sh`
- `init` создает `./.ai-teamlead/launch-agent.sh`
- `init` создает `./.ai-teamlead/zellij/analysis-tab.kdl`
- `init` создает `./.ai-teamlead/flows/issue-analysis-flow.md`
- `init` создает staged prompts в `./.ai-teamlead/flows/issue-analysis/`
- `init` создает `./.claude/README.md`
- `init` создает `./.codex/README.md`
- `init` создает comment-only `settings.yml`, который документирует required и
  defaulted поля без материализации runtime defaults в активный YAML
- `init` документирует `zellij.layout` со значением `compact` как opt-in
  example-only поле
- `init` bootstrap-ит `analysis-tab.kdl` с `compact-bar` и обязательными
  placeholders `${TAB_NAME}` и `${PANE_ENTRYPOINT}`
- если `./init.sh` отсутствует, `init` создает симлинк на
  `./.ai-teamlead/init.sh`
- bootstrapped `./.ai-teamlead/init.sh` копирует отсутствующие `.env*` из
  primary worktree, если primary worktree находится на default branch
- `init` не создает runtime state в `.git/.ai-teamlead/`
- повторный запуск не перезаписывает существующие versioned-файлы
- оператор получает понятный список созданных и пропущенных файлов

## Критерии готовности

Feature считается готовой к использованию, если:

- новый git-репозиторий с настроенным GitHub `origin` можно подготовить одной
  командой `ai-teamlead init`
- после `init` репозиторий содержит минимальный project-local контракт
- созданные файлы можно закоммитить без ручной генерации дополнительных данных
- обязательные unit и integration tests пройдены

## Инварианты

- `init` не изменяет файлы вне `./.ai-teamlead/`, кроме допустимого симлинка
  `./init.sh`
- `init` не пишет runtime-артефакты в рабочее дерево
- `init` не удаляет и не перезаписывает уже существующие project-local файлы
- `init` не требует обращения к GitHub API для базовой инициализации
- runtime defaults должны оставаться canonical в приложении, а не в активном
  YAML bootstrap-шаблона

## Сценарии проверки

Corner cases первой версии должны быть покрыты автоматическими тестами.

### Сценарий 1. Пустой репозиторий с настроенным `origin`

- есть git-репозиторий без `./.ai-teamlead/`
- настроен `origin`, указывающий на GitHub-репозиторий
- запускается `ai-teamlead init`
- создаются все обязательные project-local файлы
- `settings.yml` содержит только комментарии и documented defaults
- если `./init.sh` отсутствует, создается симлинк на `./.ai-teamlead/init.sh`

### Сценарий 1a. Zero-config шаблон

- freshly initialized `settings.yml` не содержит активных ключей YAML
- в шаблоне присутствует закомментированный placeholder `github.project_id`
- в шаблоне присутствуют закомментированные defaults для `runtime`, `zellij`,
  analysis/implementation statuses и `launch_agent.*`
- drift между template и canonical runtime defaults ловится unit-test guardrail

### Сценарий 2. Повторный запуск

- `./.ai-teamlead/` уже существует
- запускается `ai-teamlead init`
- существующие файлы не изменяются
- команда явно сообщает, что они были пропущены

### Сценарий 3. Частично инициализированный репозиторий

- существует только часть обязательных файлов
- запускается `ai-teamlead init`
- создаются только отсутствующие файлы
- существующие файлы не перезаписываются

### Сценарий 3a. В корне уже есть собственный `init.sh`

- в корне репозитория уже существует `./init.sh`
- запускается `ai-teamlead init`
- существующий `./init.sh` не заменяется симлинком

### Сценарий 4. Запуск вне git-репозитория

- команда запускается в каталоге без git
- `init` завершается с понятной ошибкой
- никаких файлов не создается

### Сценарий 5. Запуск без `origin`

- команда запускается внутри git-репозитория
- `origin` отсутствует или указывает не на GitHub
- `init` завершается с понятной ошибкой
- никаких project-local файлов не создается

### Сценарий 6. `init.sh` в feature worktree

- существует primary worktree на default branch
- в primary worktree есть `.env*`
- существует отдельный feature worktree
- в feature worktree запускается `./.ai-teamlead/init.sh`
- отсутствующие `.env*` копируются из primary worktree
- уже существующие `.env*` в feature worktree не перезаписываются

### Сценарий 7. Primary worktree не на default branch

- default branch проекта определим
- primary worktree сейчас checkout-нут на другой ветке
- в feature worktree запускается `./.ai-teamlead/init.sh`
- копирование `.env*` пропускается
- оператор получает понятное диагностическое сообщение

## Диагностика и наблюдаемость

Минимально необходимо видеть:

- в каком репозитории выполняется `init`
- какие файлы созданы
- какие файлы уже существовали и были пропущены
- почему команда завершилась ошибкой, если репозиторий не найден или запись не
  удалась

Для первой версии достаточно:

- stdout/stderr команды
- кода возврата процесса

## Связанные документы

- [README.md](../../../README.md)
- [docs/adr/0012-repo-init-command-and-project-contract-layer.md](../../adr/0012-repo-init-command-and-project-contract-layer.md)
- [docs/adr/0004-runtime-artifacts-in-git-dir.md](../../adr/0004-runtime-artifacts-in-git-dir.md)

## Открытые вопросы

- нужен ли отдельный smoke-сценарий для repo с уже кастомизированным
  `issue-analysis-flow.md`

## Журнал изменений

### 2026-03-13

- создан документ критериев проверки для `Feature 0002`

### 2026-03-14

- добавлен zero-config контракт для `settings.yml`
- добавлена проверка comment-only bootstrap template и canonical default-layer
