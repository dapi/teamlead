# Issue 12: Что строим

## Problem

Launcher сейчас всегда создает новую `zellij` session через сгенерированный
`launch-layout.kdl` с одним tab и одним pane. Из-за этого:

- нельзя использовать заранее настроенный именованный layout пользователя;
- новая session стартует без привычного default UX `zellij`;
- analysis pane становится единственной точкой входа, а вспомогательные pane
  для логов, мониторинга и операционной работы приходится добавлять вручную.

## Who Is It For

Фича нужна оператору и разработчику, который запускает `ai-teamlead` в
собственной `zellij` session и ожидает один из двух сценариев:

- session стартует с его project-local или user-local layout;
- при отсутствии кастомного layout session выглядит как обычный запуск
  `zellij`, а не как минимальная техническая заготовка.

## Feature Story

Как пользователь `ai-teamlead`, я хочу задать `zellij.layout` в
`./.ai-teamlead/settings.yml`, чтобы новая session создавалась с моим layout, а
analysis tab добавлялся автоматически, не ломая текущий workflow и обратную
совместимость конфига.

## Use Cases

1. Пользователь указывает `zellij.layout: "my-custom-layout"` и получает новую
   session с несколькими pane или tab из своего layout, после чего туда
   автоматически добавляется analysis tab `ai-teamlead`.
2. Пользователь не указывает `zellij.layout`, но при первом запуске все равно
   получает нормальную default session `zellij` c привычным UX, после чего
   analysis tab появляется как дополнительная рабочая вкладка.
3. Если session уже существует, `ai-teamlead` не пересоздает ее и продолжает
   добавлять analysis tab в существующий session context.

## Scope

В первую версию входят:

- новое опциональное поле `zellij.layout` в repo-local конфиге;
- сохранение совместимости старых `settings.yml`, где это поле отсутствует;
- новый launch path для случая, когда session еще не существует;
- fallback на нормальный default UX `zellij`, если `zellij.layout` не задан;
- отдельное добавление analysis tab после старта новой session;
- обновление шаблона `templates/init/settings.yml`;
- тесты на оба сценария запуска: с layout и без layout.

## Non-Goals

В эту задачу не входят:

- поддержка отдельного типа значения для пути к `.kdl` файлу;
- редизайн поведения для уже существующей session;
- управление содержимым пользовательского layout или его валидация до запуска;
- восстановление session/tab после падения `zellij`;
- расширение project-local конфига другими режимами запуска.

## Constraints

- Источник конфигурации остается прежним: `./.ai-teamlead/settings.yml`.
- Поле `zellij.layout` должно быть действительно опциональным на уровне YAML и
  Rust-модели.
- Analysis tab должен по-прежнему запускать `./.ai-teamlead/launch-agent.sh`
  через сгенерированный `launch-layout.kdl`.
- Для существующей session нельзя сломать текущий сценарий добавления tab через
  сгенерированный layout.

## Dependencies

- CLI-контракт `zellij` для создания новой session с именованным layout.
- CLI-контракт `zellij` для добавления нового tab в уже запущенную session.
- Текущее shell-abstraction и unit-тесты в `src/zellij.rs`.
