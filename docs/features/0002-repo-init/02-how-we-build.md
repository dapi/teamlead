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
- `./.ai-teamlead/flows/issue-analysis-flow.md`

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
- `issue-analysis-flow` bootstrap-ится как repo-local markdown-документ
- `init` использует уже настроенный GitHub `origin` как обязательный repo
  context
- существующие файлы не перезаписываются

`init` не должен:

- создавать lock-файлы или session-артефакты
- читать GitHub Project
- обращаться к GitHub API по сети
- запускать daemon
- запускать `zellij`

## Конфигурация

`init` не требует существующего `./.ai-teamlead/settings.yml` на входе.

Он создает стартовый шаблон `settings.yml`, в котором:

- присутствуют обязательные поля MVP
- `github.project_id` заполняется placeholder-значением, требующим ручной
  донастройки

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
  flows/
    issue-analysis-flow.md
```

Ephemeral runtime layer при этом остается отдельным:

```text
.git/.ai-teamlead/
```

`init` управляет только первой структурой.
