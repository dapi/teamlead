# Issue 12: Как проверяем

## Acceptance Criteria

1. `zellij.layout` поддерживается как опциональное строковое поле в
   `./.ai-teamlead/settings.yml`.
2. Старые конфиги без `zellij.layout` продолжают успешно десериализоваться и
   валидироваться.
3. Если session уже существует, launcher по-прежнему добавляет analysis tab
   через generated layout без изменения текущего поведения.
4. Если session не существует и `zellij.layout` задан, новая session создается
   через пользовательский layout, после чего analysis tab добавляется отдельно.
5. Если session не существует и `zellij.layout` не задан, новая session
   стартует не через bare generated layout `launch-layout.kdl`, а analysis tab
   добавляется отдельным действием после старта session.
6. Ошибки создания session или добавления analysis tab не скрываются и дают
   диагностируемое сообщение.

## Ready Criteria

- issue-спека, feature 0003 и связанный ADR не расходятся по семантике
  `zellij.layout`;
- implementation plan ссылается на config contract, launcher contract и
  verification-сценарии;
- путь `session exists` остается обратно совместимым;
- для `session missing` определен один проверяемый fallback-path без
  generated layout на старте session.

## Invariants

- `zellij.layout` остается опциональным repo-level полем в `settings.yml`;
- generated `launch-layout.kdl` продолжает отвечать только за analysis tab, а
  не за базовую session при `layout = None`;
- existing session по-прежнему использует path добавления analysis tab без
  пересоздания session;
- ошибки `zellij` локализуются по шагам `create session` и `add analysis tab`.

## Happy Path

1. Конфиг с `zellij.layout = "my-custom-layout"` загружается без специальных
   миграций.
2. Launcher видит, что session отсутствует.
3. Launcher создает session с пользовательским layout.
4. Launcher добавляет analysis tab из `launch-layout.kdl`.
5. `launch-agent.sh` стартует внутри analysis pane как и раньше.

## Edge Cases

- `zellij.layout` отсутствует полностью.
- `zellij.layout` задан пустой или несуществующей строкой.
- Session между проверкой `list-sessions` и запуском успевает появиться.
- Базовая session создается успешно, но добавление analysis tab завершается
  ошибкой.

## Test Plan

Unit tests:

- парсинг `Config` с `zellij.layout` и без него;
- отсутствие регрессии валидации для старого YAML;
- launcher для `session missing` + custom layout собирает ожидаемую команду
  создания session;
- launcher для `session missing` + no layout собирает fallback-команду без bare
  generated layout;
- launcher для `session exists` сохраняет прежнюю команду добавления tab;
- если логика будет разложена на несколько шагов, проверить порядок вызовов:
  сначала base session, потом analysis tab.

Integration / smoke:

- живой прогон на `zellij` с `layout` в тестовом конфиге и проверкой, что
  analysis tab появился в session;
- живой прогон без `layout` и проверкой, что session стартует с нормальным
  fallback-path без `-n <generated layout>` на старте session;
- регрессия существующего integration flow вокруг `internal launch-zellij-fixture`
  и binding `pane_id/tab_id`.

## Verification Checklist

- шаблон `templates/init/settings.yml` содержит закомментированный пример
  `layout`;
- `cargo test` проходит для unit-тестов `config` и `zellij`;
- при запуске без `layout` команда создания session не использует
  `-n <generated layout>`;
- при ручном запуске с `layout` пользователь видит свой layout и отдельный
  analysis tab;
- runtime-артефакты `pane-entrypoint.sh` и `launch-layout.kdl` продолжают
  создаваться в session directory.

## Failure Scenarios

- Неизвестный layout: launcher завершается ошибкой, не делая вид, что session
  создана успешно.
- Session поднялась, но analysis tab не добавился: ошибка должна быть явной,
  чтобы оператор мог повторить запуск и не потерять диагностику.
- Сломан fallback без `layout`: smoke-проверка должна выявить возврат к bare UX.

## Observability

- В unit-тестах нужно проверять конкретные команды, переданные в `Shell`.
- Ошибки `zellij` должны оборачиваться контекстом шага: создание session или
  добавление analysis tab.
- В логах launcher должно быть различимо, по какому path пошел запуск:
  `existing session`, `custom layout`, `default fallback`.
- Для ручной отладки остаются runtime-артефакты в `.git/.ai-teamlead/sessions`
  и manifest binding с `session_id`, `tab_id`, `pane_id`.
