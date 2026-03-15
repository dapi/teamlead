# Issue 36: Как проверяем

## Acceptance Criteria

1. В config contract есть отдельные defaulted поля для global args `claude` и
   `codex`.
2. При отсутствии пользовательского override launcher использует canonical
   defaults:
   - `codex`: `--full-auto`
   - `claude`: `--permission-mode auto`
3. После явного пользовательского override `codex` launcher запускает `codex`
   с указанными аргументами вместо default-значения.
4. После явного пользовательского override `claude` launcher запускает
   `claude` с указанными аргументами вместо default-значения.
5. `templates/init/settings.yml` показывает реальные runtime defaults как
   активный generated config и оставляет более агрессивные режимы opt-in.
6. Degraded mode без доступного агента не ломается после добавления нового
   contract.
7. Launcher не делает raw shell-splitting по пользовательской строке args.

## Ready Criteria

- analysis docs, feature docs и шаблон `settings.yml` не расходятся по
  семантике `global_args`;
- определена одна каноническая точка defaults и validation для новых полей;
- есть тесты на canonical defaults, пользовательские overrides и оба agent
  path;
- zellij-related integration coverage остается совместимой с headless runner и
  не требует host `zellij` пользователя.

## Invariants

- source of truth для default semantics остается в Rust config layer;
- шаблон `settings.yml` не расходится с активным runtime default;
- `launch-agent.sh` получает уже валидированный и shell-safe context, а не
  парсит YAML напрямую;
- per-agent args применяются только к соответствующему агенту;
- отсутствие `codex` и `claude` по-прежнему ведет к degraded shell path;
- launcher diagnostics не обязаны печатать сырые значения пользовательских
  args.

## Happy Path

1. Пользователь включает только `codex` args.
2. Launcher идет по ветке `codex`.
3. Stub или реальный CLI по default получает `--full-auto` перед prompt без
   изменения остальных аргументов.

4. Пользователь не задает override для `claude`.
5. `codex` недоступен, launcher выбирает ветку `claude`.
6. `claude` получает `--permission-mode auto` и не получает `codex` args.

## Edge Cases

- задан только один из двух списков;
- блок `global_args` отсутствует полностью;
- в списке есть пустая или whitespace-only строка;
- значение содержит пробелы внутри одного аргумента;
- старый конфиг без `global_args` начинает использовать default-layer;
- в окружении доступны оба агента;
- в окружении не доступен ни один агент.

## Test Plan

Unit tests:

- `Config` загружается без `launch_agent.global_args` и подставляет canonical
  defaults;
- `Config` корректно загружает только `claude` или только `codex` override;
- validation отклоняет пустые элементы списка;
- `render-launch-agent-context` рендерит shell-safe arrays без потери границ
  аргументов;
- render path для default-layer создает корректные arrays для `codex` и
  `claude`.

Integration tests:

- `run`/launcher path без override передает `--full-auto` в stub `codex`;
- `run`/launcher path с `codex` override передает пользовательские args и не
  дублирует default `--full-auto`, если пользователь его заменил;
- `run`/launcher path с `claude` override и без доступного `codex` передает
  пользовательские args в stub `claude`;
- `run`/launcher path без override и без доступного `codex` передает
  `--permission-mode auto` в stub `claude`;
- degraded fallback path по-прежнему открывает shell, если оба агента
  отсутствуют;
- bootstrap `init` продолжает создавать шаблон с активными defaults и
  opt-in dangerous примерами.

Operational validation:

- zellij-touching integration tests запускать только через
  `tests/integration/docker-test-runner.sh` или эквивалентный headless path;
- при ручной проверке launcher log должен различать выбранный agent branch, не
  раскрывая целиком пользовательские args.

## Verification Checklist

- `cargo test` проходит для `config`/`app`/launcher-related unit tests;
- integration tests проходят в headless окружении;
- `templates/init/settings.yml` содержит активные defaults `global_args` и
  отдельно opt-in примеры;
- `templates/init/launch-agent.sh` и `./.ai-teamlead/launch-agent.sh`
  синхронизированы по новому contract;
- argv stubs подтверждают canonical defaults по default;
- argv stubs подтверждают корректную подстановку args для `codex` и `claude`;
- fallback без агентов не регрессировал.

## Failure Scenarios

- пользователь указал пустой элемент списка: config validation должна падать
  явно;
- launcher подставил args не в тот agent branch: это считается функциональной
  регрессией;
- old config path не получил canonical defaults: это считается регрессией
  default-layer;
- отсутствует синхронизация между template launcher и dogfooding launcher:
  integration test должен это ловить;
- шаблон `settings.yml` расходится с Rust defaults: guardrail единого
  default-layer нарушен.

## Observability

- launcher log должен показывать, какой branch выбран:
  `codex`, `claude` или `degraded`;
- для отладки достаточно знать количество extra args и их источник
  (`settings override` vs `application default`), а не полные значения;
- test stubs должны сохранять argv поэлементно, чтобы можно было проверить
  границы аргументов, а не только substring-поиск по строке.
