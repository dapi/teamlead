# Issue 12: Как строим

## Approach

Решение строится как точечное расширение текущего launcher-контракта без
перестройки orchestration flow:

1. Расширить `ZellijConfig` новым полем `layout: Option<String>`.
2. Сохранить отдельный runtime-артефакт для analysis tab, но перестать считать
   hardcoded минимальный `launch-layout.kdl` достаточным продуктовым
   контрактом.
3. Разделить логику запуска новой session и логику добавления analysis tab:
   новая session создается либо пользовательским layout, либо обычным default
   UX `zellij`, а analysis tab подключается отдельным действием.
4. Для уже существующей session оставить текущую модель "добавить tab через
   generated layout".
5. Зафиксировать source of truth для внешнего вида analysis tab: tab должен
   восприниматься как родной для текущей session, включая versioned tab-level
   UX-элементы вроде `compact bar` и плагинов, если они являются частью
   выбранного контракта.

## Affected Areas

- `src/config.rs`
  модель `ZellijConfig`, десериализация и unit-тесты конфига;
- `src/zellij.rs`
  ветвление по состоянию session и сборка команд запуска;
- `templates/init/settings.yml`
  документирование нового опционального поля;
- unit-тесты launcher'а и, при необходимости, integration smoke на реальном
  `zellij`.

## Interfaces And Data

Новый конфигурационный контракт:

```yaml
zellij:
  session_name: "ai-teamlead"
  tab_name: "issue-analysis"
  layout: "my-custom-layout"  # optional
```

Семантика поля:

- `None`: launcher не использует bare generated layout для создания новой
  session и должен сохранить обычный default UX `zellij`;
- `Some(name)`: launcher создает новую session через именованный layout
  `zellij`;
- analysis tab во всех случаях продолжает добавляться отдельным действием, но
  ее layout должен рендериться из versioned tab-layout контракта, а не из
  неявного минимального bare-layout по умолчанию.

Дополнительный UX-контракт:

- analysis tab должна выглядеть как родной tab текущей session;
- отсутствие stable API для чтения layout живой session не позволяет надежно
  "унаследовать" runtime layout постфактум;
- поэтому внешний вид analysis tab должен определяться явным versioned
  источником, например project-local template, а не попыткой сериализовать
  текущее состояние session обратно в KDL.

Ожидаемая матрица поведения:

1. `session exists`
   `zellij --session <name> --layout <generated.kdl>`
2. `session missing` + `zellij.layout = Some(name)`
   сначала создать session через пользовательский layout, затем добавить новый
   analysis tab через generated layout
3. `session missing` + `zellij.layout = None`
   сначала создать session с нормальным built-in UX `zellij`, затем добавить
   analysis tab через generated layout

Предпочтительное направление для analysis tab:

- базовая session наследует user/default layout обычным путем;
- analysis tab использует versioned template с подстановкой `tab_name` и пути к
  `pane-entrypoint.sh`;
- в текущем решении source of truth хранится в
  `./.ai-teamlead/zellij/analysis-tab.kdl`;
- если проекту нужны `compact bar`, плагины и другие tab-level элементы, они
  описываются в этом template явно.

Точная CLI-форма для шага "добавить analysis tab в уже запущенную session"
должна быть подтверждена на версии `zellij`, используемой в проекте. Из issue
следует направление `zellij action new-tab --layout <generated.kdl>`, но в
реализации нужно подтвердить способ адресации нужной session.

## External Interfaces

Внешний интерфейс только один: `zellij` CLI.

Команды, которые участвуют в дизайне:

- `zellij list-sessions --short`
- `zellij --session <name> --layout <layout-name-or-generated-kdl>`
- создание новой session без bare generated layout, чтобы сохранить default UX
- `zellij action new-tab --layout <generated.kdl>` или эквивалентная команда
  для живой session

## Risks

- Поведение `zellij action new-tab` может зависеть от версии CLI и требовать
  дополнительного способа адресации session/tab.
- Между созданием session и добавлением analysis tab возможен короткий race,
  если `zellij` еще не готов принять `action`.
- Ошибка в выборе fallback-команды для `layout = None` может снова вернуть bare
  UX вместо штатного default UX.
- Невалидное имя пользовательского layout должно приводить к явной ошибке, а не
  к тихому запуску в неправильной конфигурации.
- Минимальный generated layout с одной pane не выполняет требование "analysis
  tab выглядит как родной tab session", если tab-level UX нигде явно не задан.

## Architecture Notes

Лучше не смешивать три разные обязанности в одной строке shell-команды:

- определение, существует ли session;
- создание базовой session;
- добавление analysis tab.

Практически это означает, что `launch_issue_analysis()` стоит разложить на
небольшие внутренние шаги или builder'ы команд, чтобы unit-тесты проверяли
ветки `existing session`, `custom layout`, `default fallback` независимо.

Отдельно не стоит проектировать реализацию так, будто можно надежно получить
"текущий layout session" из runtime-состояния `zellij` и затем восстановить его
как KDL. Для первой версии это слишком хрупкий и version-sensitive путь.

## ADR Impact

Нужен отдельный ADR.

Причина: задача меняет сразу три устойчивых контракта уровня подсистемы:

- versioned config contract через новое поле `zellij.layout`;
- launcher contract для создания новой `zellij` session;
- verification contract для fallback без `layout`.

ADR должен зафиксировать:

- что `zellij.layout` принимает только строковое имя layout;
- что отсутствие поля означает создание новой session без bare generated layout;
- что analysis tab не только добавляется отдельно, но и должна выглядеть как
  родной tab session согласно отдельному versioned tab-layout контракту;
- что поддержка пути к `.kdl` и других форматов значения в первую версию не
  входит.

## Alternatives Considered

### Поддержать сразу и имя layout, и путь к `.kdl`

Не брать в первую версию.

Это расширяет формат конфига и вносит новый слой валидации, хотя issue требует
только опциональный именованный layout и корректный fallback.

### Оставить создание новой session через generated layout

Отклонено.

Это противоречит dogfooding finding из issue и не решает потерю default UX.

### Пытаться наследовать analysis tab из уже живой session

Отклонено для первой версии.

У `zellij` нет стабильного contract-level API, которое позволяло бы считать
текущее runtime-состояние session и безопасно нормализовать его обратно в KDL
так, чтобы это стало источником истины для нового tab.

## Migration Or Rollout Notes

- Существующие `settings.yml` должны продолжить десериализоваться без изменений.
- `templates/init/settings.yml` получает только закомментированный пример, без
  изменения обязательного шаблона.
- Rollout безопасен как backward-compatible изменение, если ветка
  `session exists` остается без функциональной регрессии.
