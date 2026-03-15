# Feature 0006: Как строим

## Архитектура

Security model делится на четыре слоя:

### 1. Классификация входов

Все входы делятся на:

- `trusted control plane`:
  локальные инструкции агента, CLI-контракт, принятые ADR, явно заданные
  operator approvals;
- `repo-scoped semi-trusted`:
  versioned project-local assets, созданные владельцем подключенного
  репозитория и прошедшие явный review;
- `untrusted content plane`:
  issue body, comments, labels, titles, branch names, shell output, test output,
  generated artifacts, linked docs и внешний веб-контент;
- `sensitive local plane`:
  локальная файловая система вне целевого репозитория, секреты окружения,
  токены, SSH-ключи, глобальные конфиги и любые operator-specific данные.

### 2. Safe mode policy

Для public repos вводится отдельный operating mode:

- untrusted content читается только как данные для анализа, а не как источник
  команд управления runtime;
- автоматический intake по умолчанию ограничивается issue, созданными самим
  оператором запуска или заранее разрешенным owner/allowlist автором;
- переход от чтения контента к опасным действиям требует явного human gate;
- открытие внешних ссылок, network-запросы, запись вне workspace и исполнение
  команд с повышенными привилегиями не делаются автоматически;
- если visibility репозитория не удалось определить, используется поведение
  уровня `public-safe` по умолчанию.

Ограничение `owner-authored issues only` рассматривается как intake-layer, а не
как полный trust upgrade:

- оно снижает риск hostile issue intake;
- оно не делает весь thread доверенным;
- comments внутри owner-authored issue все равно остаются `untrusted content`,
  если для них не существует отдельного trusted mechanism.

### 3. Permission gates

Ключевые разрешения должны контролироваться отдельными воротами:

- `filesystem gate` для чтения и записи вне repo/worktree;
- `network gate` для любых обращений к внешним хостам, кроме явно разрешенных
  integration paths;
- `execution gate` для команд, которые меняют систему, публикуют данные или
  выполняются вне sandbox;
- `publication gate` для публикации результатов в GitHub, если вывод может
  содержать чувствительные локальные данные.

### 4. Documentation and runtime alignment

Политика не должна жить только в prompt-тексте. Она должна одновременно
фиксироваться в:

- [docs/untrusted-input-security.md](../../untrusted-input-security.md) как SSOT;
- [docs/adr/0029-untrusted-github-content-as-hostile-input.md](../../adr/0029-untrusted-github-content-as-hostile-input.md);
- [docs/adr/0030-public-repo-safe-mode-and-permission-gates.md](../../adr/0030-public-repo-safe-mode-and-permission-gates.md);
- runtime-слое `ai-teamlead`, который enforce-ит ключевые gates, а не только
  документирует их.

## Данные и состояния

Для security layer вводятся следующие понятия:

- `repo_visibility`: `public`, `private`, `unknown`;
- `operating_mode`: `standard`, `public-safe`;
- `intake_policy`: `owner-only`, `allowlist`, `open-intake`;
- `input_trust_class`: `trusted-control`, `semi-trusted-repo`, `untrusted`,
  `sensitive-local`;
- `approval_state`: `not-required`, `required`, `granted`, `denied`.

Минимальные правила:

- `repo_visibility = public` всегда приводит к `operating_mode = public-safe`;
- `repo_visibility = unknown` не может автоматически приводить к ослаблению
  ограничений;
- `intake_policy = owner-only` фильтрует, какие issue можно автоматически брать
  в работу, но не меняет trust-класс comments;
- `input_trust_class = untrusted` не может повышать собственные привилегии;
- `approval_state = granted` должен относиться к конкретному действию, а не ко
  всему последующему сеансу без ограничений.

## Интерфейсы

Контракт должен затронуть следующие интерфейсы:

- `run` и `poll` как issue-level entrypoints;
- launcher и agent runtime, которые читают repo-local assets;
- GitHub integration layer через `gh`;
- sandbox / escalation layer, через который проходят опасные действия;
- intake filtering layer, который проверяет автора issue до старта workflow;
- project-local prompts и инструкции, которые должны явно различать trusted и
  untrusted content.

## Технические решения

- публичный GitHub-контент считается hostile input по умолчанию;
- repo-local docs не могут сами по себе расширять filesystem или network scope;
- shell output, test logs и generated artifacts тоже относятся к untrusted input,
  если они возникли как следствие обработки недоверенной задачи;
- режим `public-safe` должен быть fail-closed, а не fail-open;
- owner-authored intake policy допустима как дополнительный default gate для
  public repos;
- comments не становятся trusted только потому, что issue создал владелец;
- в документации нужно отдельно различать `operator intent` и `content
  suggestion`, чтобы issue-текст не маскировался под действие пользователя.

## Конфигурация

Точное выражение security policy в `settings.yml` пока не зафиксировано, но
feature предполагает как минимум такие настраиваемые слои:

- включение или принудительное переопределение `public-safe` режима;
- intake policy для автоматического отбора issue;
- allowlist авторов, которым разрешен auto-intake;
- allowlist для допустимых network destinations;
- policy для filesystem scope;
- список действий, которые всегда требуют явного human approval.

До принятия отдельного config ADR отсутствие этих полей не должно приводить к
неявному ослаблению ограничений.

## Ограничения реализации

- первая версия может документировать часть policy раньше, чем весь enforcement
  появится в коде;
- policy не должна зависеть только от добросовестности prompt-following модели;
- часть mitigation потребует изменений сразу в CLI, launcher и project-local
  assets;
- безопасность public repos не должна подменяться рекомендацией
  "просто не открывать suspicious issue".
