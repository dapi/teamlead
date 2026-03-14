# Issue 36: Как проверяем

## Acceptance Criteria

1. В config contract есть отдельные defaulted поля для global args `claude` и
   `codex`.
2. При отсутствии пользовательского override launcher не добавляет extra args
   ни в одну agent branch.
3. После явного включения `codex` args launcher запускает `codex` с указанными
   аргументами.
4. После явного включения `claude` args launcher запускает `claude` с
   указанными аргументами, если выбран именно этот branch.
5. `templates/init/settings.yml` показывает оба примера только в
   закомментированном виде и не превращает их в runtime default.
6. Degraded mode без доступного агента не ломается после добавления нового
   contract.
7. Launcher не делает raw shell-splitting по пользовательской строке args.

## Ready Criteria

- analysis docs, feature docs и шаблон `settings.yml` не расходятся по
  семантике `global_args`;
- определена одна каноническая точка defaults и validation для новых полей;
- есть тест, который проверяет пустой default, и тесты на оба agent path;
- zellij-related integration coverage остается совместимой с headless runner и
  не требует host `zellij` пользователя.

## Invariants

- source of truth для default semantics остается в Rust config layer;
- закомментированный пример в шаблоне не равен активному runtime default;
- `launch-agent.sh` получает уже валидированный и shell-safe context, а не
  парсит YAML напрямую;
- per-agent args применяются только к соответствующему агенту;
- отсутствие `codex` и `claude` по-прежнему ведет к degraded shell path;
- launcher diagnostics не обязаны печатать сырые значения пользовательских
  args.

## Happy Path

1. Пользователь включает только `codex` args.
2. Launcher идет по ветке `codex`.
3. Stub или реальный CLI получает флаги перед prompt без изменения остальных
   аргументов.

4. Пользователь включает только `claude` args.
5. `codex` недоступен, launcher выбирает ветку `claude`.
6. `claude` получает только свои args, а не `codex` args.

## Edge Cases

- задан только один из двух списков;
- блок `global_args` отсутствует полностью;
- в списке есть пустая или whitespace-only строка;
- значение содержит пробелы внутри одного аргумента;
- в окружении доступны оба агента;
- в окружении не доступен ни один агент.

## Test Plan

Unit tests:

- `Config` загружается без `launch_agent.global_args` и подставляет пустые
  списки;
- `Config` корректно загружает только `claude` или только `codex` override;
- validation отклоняет пустые элементы списка;
- `render-launch-agent-context` рендерит shell-safe arrays без потери границ
  аргументов;
- render path для пустого default создает пустые arrays, а не фиктивные
  placeholder-значения.

Integration tests:

- `run`/launcher path без override не добавляет extra args к `codex`;
- `run`/launcher path с `codex` override передает `--full-auto` в stub `codex`;
- `run`/launcher path с `claude` override и без доступного `codex` передает
  `--dangerously-skip-permissions` в stub `claude`;
- degraded fallback path по-прежнему открывает shell, если оба агента
  отсутствуют;
- bootstrap `init` продолжает создавать шаблон с закомментированными примерами.

Operational validation:

- zellij-touching integration tests запускать только через
  `tests/integration/docker-test-runner.sh` или эквивалентный headless path;
- при ручной проверке launcher log должен различать выбранный agent branch, не
  раскрывая целиком пользовательские args.

## Verification Checklist

- `cargo test` проходит для `config`/`app`/launcher-related unit tests;
- integration tests проходят в headless окружении;
- `templates/init/settings.yml` содержит только закомментированные примеры
  `global_args`;
- `templates/init/launch-agent.sh` и `./.ai-teamlead/launch-agent.sh`
  синхронизированы по новому contract;
- argv stubs подтверждают отсутствие extra args по default;
- argv stubs подтверждают корректную подстановку args для `codex` и `claude`;
- fallback без агентов не регрессировал.

## Failure Scenarios

- пользователь указал пустой элемент списка: config validation должна падать
  явно;
- launcher подставил args не в тот agent branch: это считается функциональной
  регрессией;
- отсутствует синхронизация между template launcher и dogfooding launcher:
  integration test должен это ловить;
- комментарии в `settings.yml` случайно стали активным YAML: guardrail из issue
  `#33` нарушен.

## Observability

- launcher log должен показывать, какой branch выбран:
  `codex`, `claude` или `degraded`;
- для отладки достаточно знать количество extra args и их источник
  (`settings` vs empty default), а не полные значения;
- test stubs должны сохранять argv поэлементно, чтобы можно было проверить
  границы аргументов, а не только substring-поиск по строке.
