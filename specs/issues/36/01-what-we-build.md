# Issue 36: Что строим

## Problem

Сейчас project-local launcher hardcode-ит запуск `codex` и не имеет явного
config contract для дополнительных пользовательских аргументов агента.

Из-за этого:

- владелец репозитория не может versioned-способом включить глобальные флаги
  запуска для конкретного агента;
- `templates/init/settings.yml` не показывает, какие agent args являются
  runtime defaults, а какие должны включаться только через opt-in;
- правило из issue `#33` про extension-like закомментированные примеры пока не
  получает практического применения для agent launch path;
- shell boundary остается неявным: если когда-нибудь передавать args как raw
  string, появится риск скрытого shell-splitting и плохой валидации.
- у пользователей нет repo-level default для "достаточно автономного" запуска
  обоих поддерживаемых агентов.

Отдельно есть несоответствие между документацией и текущим bootstrap launcher:
документы уже говорят о запуске `codex` или `claude`, но template launcher
реально использует только ветку `codex` и затем падает в degraded shell mode.

## Who Is It For

- владелец репозитория, который настраивает `./.ai-teamlead/settings.yml`
- оператор, которому нужен предсказуемый launcher behavior без скрытых
  runtime default
- разработчик, который поддерживает config contract, launcher script и
  integration tests

## Outcome

Нужен явный config contract, в котором:

- в `settings.yml` есть отдельное поле для `claude` и отдельное поле для
  `codex` внутри launcher-настроек;
- значение каждого поля задается как список CLI-аргументов, а не как raw shell
  string;
- runtime по умолчанию использует достаточно автономные, но не максимально
  опасные значения:
  - для `codex`: `--ask-for-approval never --sandbox workspace-write`
  - для `claude`: `--permission-mode auto`
- `templates/init/settings.yml` показывает эти defaults явно, а более
  агрессивные режимы оставляет opt-in примерами;
- launcher подставляет args только в ветку реально запускаемого агента;
- fallback без доступного агента продолжает работать без скрытой регрессии.

## Scope

В текущую задачу входит:

- расширение repo-local config contract для per-agent global args;
- явная семантика default-layer: отсутствие override означает application
  defaults для разумной автономии;
- обновление `launch-agent.sh` и bootstrap template для передачи args в
  `codex` и `claude`;
- синхронизация `templates/init/settings.yml` с новым правилом: автономные
  defaults активны по умолчанию, а dangerous-режимы остаются opt-in;
- тесты на canonical defaults, `codex` path, `claude` path и degraded fallback;
- обновление документации вокруг launcher contract и bootstrap template.

## Non-Goals

В текущую задачу не входит:

- произвольный raw shell snippet вместо списка аргументов;
- per-issue или per-session override для agent args;
- хранение выбора агента в отдельном новом config contract;
- изменение prompt contract или структуры `issue-analysis-flow`;
- расширение args на другие инструменты помимо `claude` и `codex`;
- включение максимально опасных bypass-режимов по умолчанию.

## Constraints And Assumptions

- формат должен быть безопасен для shell boundary, поэтому аргументы хранятся
  как список строк, а не как одна shell-строка;
- runtime default должен быть один и совпадать между Rust default-layer,
  `templates/init/settings.yml` и документацией;
- `--ask-for-approval never --sandbox workspace-write` для `codex` и
  `--permission-mode auto` для `claude` рассматриваются как "достаточно
  автономные", но не как опасный bypass safety boundary;
- dangerous-режимы вроде `--dangerously-skip-permissions` для `claude` должны
  оставаться явным opt-in;
- существующие конфиги без новых полей должны оставаться валидными;
- launcher не должен логировать значения пользовательских args как часть
  обычной диагностики, чтобы не плодить accidental secret leakage;
- degraded mode при отсутствии доступного агента должен оставаться допустимым
  исходом.

## User Story

Как владелец подключенного репозитория, я хочу явно включать global args для
`claude` и `codex` в `settings.yml`, сохраняя при этом осмысленные автономные
defaults, чтобы launcher был достаточно самостоятельным из коробки, но более
опасные режимы я мог включать только явным opt-in через versioned contract.

## Use Cases

1. Пользователь ничего не меняет в конфиге и получает запуск `codex` с runtime
   default `--ask-for-approval never --sandbox workspace-write`, при этом
   остальные launcher semantics не меняются.
2. Пользователь ничего не меняет в конфиге и получает запуск `claude` с
   runtime default `--permission-mode auto`, если launcher идет по ветке
   `claude`.
3. Пользователь явным opt-in включает для `claude`
   `--dangerously-skip-permissions`, если ему нужен более агрессивный режим.
4. В окружении нет ни `codex`, ни `claude`; launcher по-прежнему оставляет
   shell в analysis worktree и не пытается интерпретировать пустые или
   отсутствующие args как команду.

## Dependencies

- issue `#33` как родительский guardrail для единого default-layer и
  контролируемых opt-in расширений
- [../../../docs/features/0002-repo-init/README.md](../../../docs/features/0002-repo-init/README.md)
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
- существующий headless integration runner для `zellij`-затрагивающих сценариев
