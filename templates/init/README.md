# .ai-teamlead

Этот каталог содержит versioned project-specific настройки и документы для
`ai-teamlead`.

Правила:

- содержимое каталога живет в рабочем дереве репозитория и может коммититься
- владелец репозитория может менять эти файлы под нужды проекта
- runtime-state сюда не пишется
- ephemeral state хранится отдельно в `.git/.ai-teamlead/`

Текущие файлы:

- `settings.yml` — repo-local конфиг `ai-teamlead`
- `init.sh` — project-local bootstrap script для worktree
- `launch-agent.sh` — project-local launcher script для issue-analysis session
- `zellij/analysis-tab.kdl` — versioned template для analysis tab
- `flows/issue-analysis-flow.md` — repo-local flow анализа issue

Базовый системный контракт flow остается в документации `ai-teamlead`, а файлы
в этом каталоге задают project-specific адаптацию.

Если в корне репозитория отсутствует `./init.sh`, команда `ai-teamlead init`
создает симлинк:

- `./init.sh -> ./.ai-teamlead/init.sh`
