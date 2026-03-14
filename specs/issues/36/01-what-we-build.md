# Issue 36: Что строим

## Problem

Сейчас project-local launcher hardcode-ит запуск `codex` и не имеет явного
config contract для дополнительных пользовательских аргументов агента.

Из-за этого:

- владелец репозитория не может versioned-способом включить глобальные флаги
  запуска для конкретного агента;
- `templates/init/settings.yml` не показывает, как безопасно включать такие
  расширения через opt-in;
- правило из issue `#33` про extension-like закомментированные примеры пока не
  получает практического применения для agent launch path;
- shell boundary остается неявным: если когда-нибудь передавать args как raw
  string, появится риск скрытого shell-splitting и плохой валидации.

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
- при отсутствии пользовательского override runtime не добавляет никаких extra
  args;
- `templates/init/settings.yml` показывает оба примера только в
  закомментированном виде;
- launcher подставляет args только в ветку реально запускаемого агента;
- fallback без доступного агента продолжает работать без скрытой регрессии.

## Scope

В текущую задачу входит:

- расширение repo-local config contract для per-agent global args;
- явная семантика default-layer: отсутствие override означает пустой список;
- обновление `launch-agent.sh` и bootstrap template для передачи args в
  `codex` и `claude`;
- синхронизация `templates/init/settings.yml` с guardrail из issue `#33`;
- тесты на пустой default, `codex` path, `claude` path и degraded fallback;
- обновление документации вокруг launcher contract и bootstrap template.

## Non-Goals

В текущую задачу не входит:

- произвольный raw shell snippet вместо списка аргументов;
- per-issue или per-session override для agent args;
- хранение выбора агента в отдельном новом config contract;
- изменение prompt contract или структуры `issue-analysis-flow`;
- расширение args на другие инструменты помимо `claude` и `codex`;
- скрытый runtime default, который активируется только потому, что пример
  показан в шаблоне.

## Constraints And Assumptions

- формат должен быть безопасен для shell boundary, поэтому аргументы хранятся
  как список строк, а не как одна shell-строка;
- закомментированный пример в `templates/init/settings.yml` может отличаться от
  runtime default только как opt-in расширение и не должен менять поведение при
  отсутствии override;
- существующие конфиги без новых полей должны оставаться валидными;
- launcher не должен логировать значения пользовательских args как часть
  обычной диагностики, чтобы не плодить accidental secret leakage;
- degraded mode при отсутствии доступного агента должен оставаться допустимым
  исходом.

## User Story

Как владелец подключенного репозитория, я хочу явно включать global args для
`claude` и `codex` в `settings.yml`, чтобы настраивать launcher behavior через
versioned contract без скрытых runtime default и без небезопасной shell-строки.

## Use Cases

1. Пользователь разкомментирует пример для `codex` и получает запуск
   `codex` с `--full-auto`, при этом остальные launcher semantics не меняются.
2. Пользователь разкомментирует пример для `claude` и получает запуск
   `claude` с `--dangerously-skip-permissions`, если launcher идет по ветке
   `claude`.
3. Пользователь не включает ни одно поле и получает текущее поведение без
   extra args.
4. В окружении нет ни `codex`, ни `claude`; launcher по-прежнему оставляет
   shell в analysis worktree и не пытается интерпретировать пустые или
   отсутствующие args как команду.

## Dependencies

- issue `#33` как родительский guardrail для zero-config шаблона и
  extension-like закомментированных примеров
- [../../../docs/features/0002-repo-init/README.md](../../../docs/features/0002-repo-init/README.md)
- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
- существующий headless integration runner для `zellij`-затрагивающих сценариев
