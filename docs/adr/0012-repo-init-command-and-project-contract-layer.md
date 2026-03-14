# ADR-0012: `ai-teamlead init` и versioned project contract layer

Статус: accepted
Дата: 2026-03-13

## Контекст

`issue-analysis-flow` должен настраиваться владельцем конкретного репозитория.
Это означает, что в целевом репозитории должен существовать versioned-слой, в
котором хранятся:

- repo-local конфиг
- project-local flow-документы
- в будущем другие project-local шаблоны и контракты

Ранее эта идея начала появляться в коде как `bootstrap`, но как отдельная фича
и отдельный CLI-контракт она еще не была описана.

## Решение

Для подключения `ai-teamlead` к репозиторию вводится отдельная команда:

- `ai-teamlead init`

Команда `init` отвечает только за инициализацию versioned project contract
layer в рабочем дереве репозитория.

`init` в первой версии требует, чтобы:

- команда запускалась внутри git-репозитория
- в репозитории уже был настроен `origin`
- `origin` указывал на GitHub-репозиторий, который считается target context для
  текущего проекта

Versioned project contract layer хранится в:

- `./.ai-teamlead/`

В первой версии `init` должна создавать versioned-файлы contract layer:

- `./.ai-teamlead/settings.yml`
- `./.ai-teamlead/README.md`
- `./.ai-teamlead/init.sh`
- `./.ai-teamlead/launch-agent.sh`
- `./.ai-teamlead/flows/issue-analysis-flow.md`
- `./.ai-teamlead/flows/issue-analysis/README.md`
- `./.ai-teamlead/flows/issue-analysis/01-what-we-build.md`
- `./.ai-teamlead/flows/issue-analysis/02-how-we-build.md`
- `./.ai-teamlead/flows/issue-analysis/03-how-we-verify.md`
- `./.claude/README.md`
- `./.codex/README.md`

Если в корне репозитория отсутствует `./init.sh`, `init` дополнительно создает
симлинк:

- `./init.sh -> ./.ai-teamlead/init.sh`

Ephemeral/runtime state не создается и не обслуживается командой `init`.
Runtime-артефакты по-прежнему хранятся отдельно в:

- `.git/.ai-teamlead/`

Команда `init` должна быть идемпотентной:

- существующие versioned-файлы не перезаписываются
- команда сообщает, какие файлы созданы, а какие пропущены

## Последствия

Плюсы:

- repo-local contract становится явной и versioned-частью проекта
- flow можно ревьюить и изменять средствами самого репозитория
- подключение `ai-teamlead` к новому репозиторию получает отдельную точку входа
- versioned project contract четко отделяется от runtime state

Минусы:

- появляется дополнительная пользовательская команда
- нужно отдельно проектировать и тестировать `init`
- нужно синхронизировать эту feature с `Feature 0001`

## Альтернативы

### 1. Не делать отдельную команду

Отклонено.

Это размывает контракт подключения репозитория и мешает явно отделить
инициализацию project-local файлов от runtime-модели.

### 2. Оставить имя `bootstrap`

Отклонено.

`init` короче, понятнее и лучше соответствует назначению команды.

### 3. Хранить project-local flow вне репозитория

Отклонено.

Это противоречит требованию repo-local и versioned customization layer.

## Связанные документы

- [README.md](/home/danil/code/teamlead/README.md)
- [docs/features/0002-repo-init/README.md](/home/danil/code/teamlead/docs/features/0002-repo-init/README.md)
- [docs/adr/0001-repo-local-ai-config.md](/home/danil/code/teamlead/docs/adr/0001-repo-local-ai-config.md)
- [docs/adr/0004-runtime-artifacts-in-git-dir.md](/home/danil/code/teamlead/docs/adr/0004-runtime-artifacts-in-git-dir.md)

## Журнал изменений

### 2026-03-13

- зафиксирована команда `ai-teamlead init`
- зафиксирован versioned project contract layer в `./.ai-teamlead/`
- зафиксировано разделение `./.ai-teamlead/` и `.git/.ai-teamlead/`
